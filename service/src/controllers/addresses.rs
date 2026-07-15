use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use uuid::Uuid;

use crate::{
    AppState, Error, Result,
    context::Claims,
    middlewares::{AuthLayer, RbacLayer},
    models::Address,
    schemas::{NewAddress, Validator},
    utils::{AppExtension, AppJson, AppPath},
};

fn customer_pid(claims: &Claims) -> Result<Uuid> {
    Uuid::parse_str(claims.sub()).map_err(|_| Error::Forbidden.into())
}

#[tracing::instrument(skip(ctx, claims))]
#[debug_handler]
async fn list(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
) -> Result<Response> {
    let addresses = Address::find_by_customer(ctx.db(), customer_pid(&claims)?).await?;

    Ok((StatusCode::OK, Json(addresses)).into_response())
}

#[tracing::instrument(skip(ctx, claims))]
#[debug_handler]
async fn one(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
    AppPath(pid): AppPath<Uuid>,
) -> Result<Response> {
    let address = Address::find_by_pid_for_customer(ctx.db(), pid, customer_pid(&claims)?).await?;

    Ok((StatusCode::OK, Json(address)).into_response())
}

#[tracing::instrument(skip(ctx, claims, params))]
#[debug_handler]
async fn create(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
    AppJson(params): AppJson<NewAddress>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;
    let address = Address::create(ctx.db(), customer_pid(&claims)?, validated).await?;

    Ok((StatusCode::CREATED, Json(address)).into_response())
}

#[tracing::instrument(skip(ctx, claims, params))]
#[debug_handler]
async fn update(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
    AppPath(pid): AppPath<Uuid>,
    AppJson(params): AppJson<NewAddress>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;
    let address =
        Address::update_for_customer(ctx.db(), pid, customer_pid(&claims)?, validated).await?;

    Ok((StatusCode::OK, Json(address)).into_response())
}

#[tracing::instrument(skip(ctx, claims))]
#[debug_handler]
async fn remove(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
    AppPath(pid): AppPath<Uuid>,
) -> Result<Response> {
    Address::delete_for_customer(ctx.db(), pid, customer_pid(&claims)?).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{pid}", get(one).put(update).delete(remove))
        .with_state(ctx.clone())
        .layer(RbacLayer::customers_only(ctx.clone()))
        .layer(AuthLayer::new(ctx.clone()))
}
