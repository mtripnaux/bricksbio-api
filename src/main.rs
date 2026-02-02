mod merge;
mod ontology;
mod types;
mod search;
mod providers;
mod parsers;
mod exporters;

use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde_json::json;
use types::Biobrick;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(serve_redoc))
        .route("/openapi.yaml", get(serve_openapi))
        .route("/parts/:id", get(get_part))
        .route("/parts/:id/sbol", get(get_part_sbol));

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
async fn get_part(Path(id): Path<String>) -> Result<Json<Biobrick>, (StatusCode, Json<serde_json::Value>)> {
    let biobrick = search::meta_search(&id).await;

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
async fn get_part_sbol(Path(id): Path<String>) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let biobrick = search::meta_search(&id).await;

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