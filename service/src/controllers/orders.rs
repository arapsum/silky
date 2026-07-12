use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use uuid::Uuid;

use crate::{
    AppState, Result,
    models::Order,
    schemas::{OrderListQuery, Validator},
    utils::{AppPath, AppQuery},
};

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn list(
    State(ctx): State<AppState>,
    AppQuery(query): AppQuery<OrderListQuery>,
) -> Result<Response> {
    let validator = Validator::new(query);
    let validated = validator.validate()?;
    let orders = Order::find_all(ctx.db(), validated).await?;

    Ok((StatusCode::OK, Json(orders)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn one(State(ctx): State<AppState>, AppPath(pid): AppPath<Uuid>) -> Result<Response> {
    let order = Order::find_detail_by_pid(ctx.db(), pid).await?;

    Ok((StatusCode::OK, Json(order)).into_response())
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route("/", get(list))
        .route("/{pid}", get(one))
        .with_state(ctx.clone())
}
