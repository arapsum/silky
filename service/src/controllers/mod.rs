use std::sync::Arc;

use axum::{
    Json, Router, debug_handler,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::get,
};

use crate::AppContext;

mod auth;

#[debug_handler]
async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({"message": "Server is up and running"})),
    )
        .into_response()
}

pub async fn not_found(uri: Uri) -> impl IntoResponse {
    let path = uri.path();
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"message": format!("Page not found {path}")})),
    )
}

pub fn router(ctx: &Arc<AppContext>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth::router(ctx))
        .fallback(not_found)
}
