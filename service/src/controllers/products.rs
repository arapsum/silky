use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::{
    AppState, Result,
    access_control::permissions,
    middlewares::{AuthLayer, RbacLayer},
    models::{Attribute, Product},
    schemas::{CreateProduct, Validator},
    utils::AppJson,
};

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn create(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<CreateProduct<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let product = Product::create(ctx.db(), validated).await?;

    Ok((StatusCode::CREATED, Json(product)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn attributes(State(ctx): State<AppState>) -> Result<Response> {
    let attributes = Attribute::find_all_with_values(ctx.db()).await?;

    Ok((StatusCode::OK, Json(attributes)).into_response())
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route(
            "/attributes",
            get(attributes).layer(RbacLayer::new(ctx.clone(), permissions::products::READ)),
        )
        .route(
            "/",
            post(create).layer(RbacLayer::new(ctx.clone(), permissions::products::CREATE)),
        )
        .with_state(ctx.clone())
        .layer(AuthLayer::new(ctx.clone()))
}
