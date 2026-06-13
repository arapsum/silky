use axum::{
    Json, Router, debug_handler,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::get,
};

#[debug_handler]
async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({"message": "Server is up and running"})),
    )
        .into_response()
}

pub(crate) async fn not_found(uri: Uri) -> impl IntoResponse {
    let path = uri.path();
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"message": format!("Page not found {path}")})),
    )
}

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .fallback(not_found)
}
