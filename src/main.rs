mod cache;
mod merge;
mod ontology;
mod types;
mod search;
mod semantic;
mod providers;
mod parsers;
mod exporters;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use crate::ontology::{ONTOLOGY, OntologyEntrySerializable};
use std::{collections::HashSet, sync::Arc};
use axum::extract::Query;
use tower_http::services::ServeDir;
use serde_json::json;
use types::{Biobrick, CacheSearchParams, CacheStats, SearchHit, SearchResponse};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub cache: cache::SqliteCache,
    pub refresh_in_flight: Arc<tokio::sync::Mutex<HashSet<String>>>,
}

#[tokio::main]
async fn main() {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    let cache = cache::SqliteCache::new("cache/bricksbio.db").unwrap();
    let state = AppState {
        client,
        cache,
        refresh_in_flight: Arc::new(tokio::sync::Mutex::new(HashSet::new())),
    };

    let app = Router::new()
        .route("/", get(serve_redoc))
        .route("/openapi.yaml", get(serve_openapi))
        .route("/cache/stats", get(get_cache_stats))
        .route("/cache/search", get(get_cache_search))
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/parts/:id", get(get_part))
        .route("/parts/:id/sbol", get(get_part_sbol))
        .route("/ontology", get(get_ontology))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_redoc() -> Html<&'static str> {
    Html(include_str!("../docs/index.html"))
}

async fn serve_openapi() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/yaml")], include_str!("../docs/openapi.yaml"))
}

#[axum::debug_handler]
async fn get_cache_stats(
    State(state): State<AppState>,
) -> Result<Json<CacheStats>, (StatusCode, Json<serde_json::Value>)> {
    match state.cache.stats_entries() {
        Ok(entries) => Ok(Json(CacheStats { entries })),
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": format!("Failed to read cache stats: {}", error) })),
        )),
    }
}

#[axum::debug_handler]
async fn get_cache_search(
    State(state): State<AppState>,
    Query(params): Query<CacheSearchParams>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<serde_json::Value>)> {
    let query = params.q.trim().to_string();
    if query.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "message": "Missing or empty query parameter: q" })),
        ));
    }

    let requested = params.n.unwrap_or(10).min(50);

    match semantic::cache_search(&state.cache, &query, requested) {
        Ok(items) => {
            let results = items
                .into_iter()
                .map(|(biobrick, score)| SearchHit {
                    r#match: score,
                    biobrick,
                })
                .collect::<Vec<_>>();

            Ok(Json(SearchResponse {
                query,
                requested,
                count: results.len(),
                results,
            }))
        }
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": format!("Failed to search cache: {}", error) })),
        )),
    }
}

#[axum::debug_handler]
async fn get_part(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Biobrick>, (StatusCode, Json<serde_json::Value>)> {
    let biobrick = search::meta_search(&state, &id).await;

    match biobrick {
        Some(b) => {
            if b.metadata.size == 0 {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({ "message": "Part not found" })),
                ));
            }
            Ok(Json(b))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Part not found" })),
        )),
    }
}

#[axum::debug_handler]
async fn get_part_sbol(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let biobrick = search::meta_search(&state, &id).await;

    match biobrick {
        Some(b) => {
            if b.metadata.size == 0 {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({ "message": "Part not found" })),
                ));
            }
            let sbol = exporters::sbol::to_sbol_xml(&b);
            Ok((
                [(header::CONTENT_TYPE, "application/rdf+xml")],
                sbol,
            ))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Part not found" })),
        )),
    }
}

#[axum::debug_handler]
async fn get_ontology() -> Json<Vec<OntologyEntrySerializable>> {
    let serializable: Vec<OntologyEntrySerializable> = ONTOLOGY.iter().map(OntologyEntrySerializable::from).collect();
    Json(serializable)
}