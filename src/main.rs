mod cache;
mod merge;
mod ontology;
mod limit;
mod types;
mod search;
mod semantic;
mod providers;
mod parsers;
mod exporters;

use axum::{
    extract::{ConnectInfo, Path, Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use crate::ontology::{ONTOLOGY, OntologyEntrySerializable};
use crate::limit::{RateLimiter, MAX_REQUESTS, WINDOW_SECS};
use std::{collections::HashSet, net::{IpAddr, SocketAddr}, sync::Arc};
use axum::extract::Query;
use tower_http::services::ServeDir;
use serde_json::json;
use types::{ApiStats, Biobrick, CacheSearchParams, CacheStats, SearchHit, SearchResponse};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub cache: cache::SqliteCache,
    pub refresh_in_flight: Arc<tokio::sync::Mutex<HashSet<String>>>,
    pub rate_limiter: RateLimiter,
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
        rate_limiter: RateLimiter::new(),
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
        .route("/stats", get(get_api_stats))
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

fn extract_client_ip(request: &Request, peer_addr: IpAddr) -> IpAddr {
    request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .or_else(|| {
            request
                .headers()
                .get("X-Real-IP")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.trim().parse().ok())
        })
        .unwrap_or(peer_addr)
}

async fn rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let ip = extract_client_ip(&request, addr.ip());
    let _ = state.cache.record_request(&ip.to_string());

    let (allowed, remaining, retry_after) = state.rate_limiter.check(ip);

    if !allowed {
        let mut resp = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "message": "Rate limit exceeded",
                "retry_after": retry_after
            })),
        )
            .into_response();
        let headers = resp.headers_mut();
        headers.insert("X-RateLimit-Limit", HeaderValue::from_str(&MAX_REQUESTS.to_string()).unwrap());
        headers.insert("X-RateLimit-Remaining", HeaderValue::from_static("0"));
        headers.insert("Retry-After", HeaderValue::from_str(&retry_after.to_string()).unwrap());
        return resp;
    }

    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert("X-RateLimit-Limit", HeaderValue::from_str(&MAX_REQUESTS.to_string()).unwrap());
    headers.insert("X-RateLimit-Remaining", HeaderValue::from_str(&remaining.to_string()).unwrap());
    headers.insert("X-RateLimit-Window", HeaderValue::from_str(&format!("{}s", WINDOW_SECS)).unwrap());
    response
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

#[axum::debug_handler]
async fn get_api_stats(
    State(state): State<AppState>,
) -> Result<Json<ApiStats>, (StatusCode, Json<serde_json::Value>)> {
    match state.cache.get_api_stats() {
        Ok(stats) => Ok(Json(stats)),
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": format!("Failed to read stats: {}", error) })),
        )),
    }
}
