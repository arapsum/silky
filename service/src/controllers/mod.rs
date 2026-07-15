use axum::{
    Json, Router, debug_handler,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::get,
};

use crate::{AppState, middlewares::auth::AuthLayer};

mod addresses;
mod auth;
mod categories;
mod media;
mod orders;
mod payments;
mod permissions;
mod products;
mod roles;
mod users;

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

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth::router(ctx))
        .nest("/addresses", addresses::router(ctx))
        .nest(
            "/permissions",
            permissions::router(ctx).layer(AuthLayer::new(ctx.clone())),
        )
        .nest(
            "/roles",
            roles::router(ctx).layer(AuthLayer::new(ctx.clone())),
        )
        .nest(
            "/users",
            users::router(ctx).layer(AuthLayer::new(ctx.clone())),
        )
        .nest("/categories", categories::router(ctx))
        .nest("/products", products::router(ctx))
        .nest("/orders", orders::router(ctx))
        .nest("/payments", payments::router(ctx))
        .nest("/media", media::router(ctx))
        .fallback(not_found)
}
