use axum::{
    Json, Router, debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
};
use uuid::Uuid;

use crate::{
    AppState, Result,
    models::Role,
    schemas::{NewRole, UpdateRole, Validator},
    utils::AppJson,
};

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn create(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<NewRole<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let new_role = Role::create(ctx.db(), validated).await?;

    Ok((StatusCode::CREATED, Json(new_role)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn list(State(ctx): State<AppState>) -> Result<Response> {
    let roles = Role::find_list(ctx.db()).await?;

    Ok((StatusCode::OK, Json(roles)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn update(
    State(ctx): State<AppState>,
    Path(pid): Path<Uuid>,
    AppJson(params): AppJson<UpdateRole<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let updated_role = Role::update(ctx.db(), pid, validated).await?;

    Ok((StatusCode::CREATED, Json(updated_role)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn one(State(ctx): State<AppState>, Path(pid): Path<Uuid>) -> Result<Response> {
    let role = Role::find_by_pid(ctx.db(), pid).await?;

    Ok((StatusCode::OK, Json(role)).into_response())
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route("/", post(create))
        .route("/", get(list))
        .route("/{pid}", patch(update))
        .route("/{pid}", get(one))
        .with_state(ctx.clone())
}
