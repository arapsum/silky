use axum::{
    Json, Router, debug_handler,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use uuid::Uuid;

use crate::{
    AppState, Result,
    middlewares::{AuthLayer, RbacLayer},
    models::Category,
    schemas::{NewCategory, PaginationQuery, UpdateCategory, Validator},
    utils::AppJson,
};

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn create(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<NewCategory<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let category = Category::create(ctx.db(), validated).await?;

    Ok((StatusCode::CREATED, Json(category)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn list(
    State(ctx): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Response> {
    let validator = Validator::new(query);
    let validated = validator.validate()?;

    let categories = Category::find_all(ctx.db(), validated).await?;

    Ok((StatusCode::OK, Json(categories)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn one(State(ctx): State<AppState>, Path(pid): Path<Uuid>) -> Result<Response> {
    let category = Category::find_by_pid(ctx.db(), pid).await?;

    Ok((StatusCode::OK, Json(category)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn update(
    State(ctx): State<AppState>,
    Path(pid): Path<Uuid>,
    AppJson(params): AppJson<UpdateCategory<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let category = Category::update(ctx.db(), pid, validated).await?;

    Ok((StatusCode::CREATED, Json(category)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn remove(State(ctx): State<AppState>, Path(pid): Path<Uuid>) -> Result<Response> {
    let category = Category::delete(ctx.db(), pid).await?;

    Ok((StatusCode::NO_CONTENT, Json(category)).into_response())
}

fn auth_router(ctx: &AppState) -> Router {
    Router::new()
        .route(
            "/",
            post(create).layer(RbacLayer::new(ctx.clone(), "categories:create")),
        )
        .route(
            "/{pid}",
            patch(update).layer(RbacLayer::new(ctx.clone(), "categories:update")),
        )
        .route(
            "/{pid}",
            delete(remove).layer(RbacLayer::new(ctx.clone(), "categories:delete")),
        )
        .with_state(ctx.clone())
}

fn unauth_router(ctx: &AppState) -> Router {
    Router::new()
        .route("/", get(list))
        .route("/{pid}", get(one))
        .with_state(ctx.clone())
}

pub fn router(ctx: &AppState) -> Router {
    unauth_router(ctx).merge(auth_router(ctx).layer(AuthLayer::new(ctx.clone())))
}
