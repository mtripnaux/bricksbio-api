mod merge;
mod ontology;
mod types;
mod search;
mod providers;
mod parsers;

use axum::{
    extract::Path,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use types::Biobrick;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/parts/:id", get(get_part));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn get_part(Path(id): Path<String>) -> Result<Json<Biobrick>, StatusCode> {
    let biobrick = search::meta_search(&id).await;

    match biobrick {
        Some(b) => {
            if b.metadata.size == 0 {
                return Err(StatusCode::NOT_FOUND);
            }
            Ok(Json(b))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}