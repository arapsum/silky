use axum::{Json, Router, debug_handler, http::StatusCode, response::IntoResponse, routing::get};

#[debug_handler]
async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({"message": "Server is up and running"})),
    )
        .into_response()
}

pub fn router() -> Router {
    Router::new().route("/health", get(health_check))
}
