use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use uuid::Uuid;

use crate::{
    AppState, Result,
    access_control::permissions,
    middlewares::{AuthLayer, RbacLayer},
    models::{Attribute, Product, Tag},
    schemas::{
        CreateProduct, CreateProductPicture, CreateProductTag, CreateProductVariant,
        ProductListQuery, UpdateProduct, UpdateProductPicture, UpdateProductVariant, Validator,
    },
    utils::{AppJson, AppPath, AppQuery},
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
async fn update(
    State(ctx): State<AppState>,
    AppPath(pid): AppPath<Uuid>,
    AppJson(params): AppJson<UpdateProduct>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let product = Product::update(ctx.db(), pid, validated).await?;

    Ok((StatusCode::OK, Json(product)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn list(
    State(ctx): State<AppState>,
    AppQuery(query): AppQuery<ProductListQuery>,
) -> Result<Response> {
    let validator = Validator::new(query);
    let validated = validator.validate()?;

    let products = Product::find_list(ctx.db(), validated).await?;

    Ok((StatusCode::OK, Json(products)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn one(State(ctx): State<AppState>, AppPath(pid): AppPath<Uuid>) -> Result<Response> {
    let product = Product::find_detail_by_pid(ctx.db(), pid).await?;

    Ok((StatusCode::OK, Json(product)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn remove(State(ctx): State<AppState>, AppPath(pid): AppPath<Uuid>) -> Result<Response> {
    let product = Product::delete(ctx.db(), pid).await?;

    Ok((StatusCode::NO_CONTENT, Json(product)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn create_variant(
    State(ctx): State<AppState>,
    AppPath(pid): AppPath<Uuid>,
    AppJson(params): AppJson<CreateProductVariant<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let product = Product::create_variant(ctx.db(), pid, validated).await?;

    Ok((StatusCode::CREATED, Json(product)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn update_variant(
    State(ctx): State<AppState>,
    AppPath((pid, variant_pid)): AppPath<(Uuid, Uuid)>,
    AppJson(params): AppJson<UpdateProductVariant>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let product = Product::update_variant(ctx.db(), pid, variant_pid, validated).await?;

    Ok((StatusCode::OK, Json(product)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn remove_variant(
    State(ctx): State<AppState>,
    AppPath((pid, variant_pid)): AppPath<(Uuid, Uuid)>,
) -> Result<Response> {
    let product = Product::delete_variant(ctx.db(), pid, variant_pid).await?;

    Ok((StatusCode::OK, Json(product)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn set_default_variant(
    State(ctx): State<AppState>,
    AppPath((pid, variant_pid)): AppPath<(Uuid, Uuid)>,
) -> Result<Response> {
    let product = Product::set_default_variant(ctx.db(), pid, variant_pid).await?;

    Ok((StatusCode::OK, Json(product)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn add_picture(
    State(ctx): State<AppState>,
    AppPath(pid): AppPath<Uuid>,
    AppJson(params): AppJson<CreateProductPicture<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let picture = Product::add_picture(ctx.db(), pid, validated).await?;

    Ok((StatusCode::CREATED, Json(picture)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn update_picture(
    State(ctx): State<AppState>,
    AppPath((pid, picture_pid)): AppPath<(Uuid, Uuid)>,
    AppJson(params): AppJson<UpdateProductPicture>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let picture = Product::update_picture_order(ctx.db(), pid, picture_pid, validated).await?;

    Ok((StatusCode::OK, Json(picture)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn remove_picture(
    State(ctx): State<AppState>,
    AppPath((pid, picture_pid)): AppPath<(Uuid, Uuid)>,
) -> Result<Response> {
    let picture = Product::delete_picture(ctx.db(), pid, picture_pid).await?;

    Ok((StatusCode::NO_CONTENT, Json(picture)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn add_variant_picture(
    State(ctx): State<AppState>,
    AppPath((pid, variant_pid)): AppPath<(Uuid, Uuid)>,
    AppJson(params): AppJson<CreateProductPicture<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let picture = Product::add_variant_picture(ctx.db(), pid, variant_pid, validated).await?;

    Ok((StatusCode::CREATED, Json(picture)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn update_variant_picture(
    State(ctx): State<AppState>,
    AppPath((pid, variant_pid, picture_pid)): AppPath<(Uuid, Uuid, Uuid)>,
    AppJson(params): AppJson<UpdateProductPicture>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let picture =
        Product::update_variant_picture_order(ctx.db(), pid, variant_pid, picture_pid, validated)
            .await?;

    Ok((StatusCode::OK, Json(picture)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn remove_variant_picture(
    State(ctx): State<AppState>,
    AppPath((pid, variant_pid, picture_pid)): AppPath<(Uuid, Uuid, Uuid)>,
) -> Result<Response> {
    let picture = Product::delete_variant_picture(ctx.db(), pid, variant_pid, picture_pid).await?;

    Ok((StatusCode::NO_CONTENT, Json(picture)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn attributes(State(ctx): State<AppState>) -> Result<Response> {
    let attributes = Attribute::find_all_with_values(ctx.db()).await?;

    Ok((StatusCode::OK, Json(attributes)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn tags(State(ctx): State<AppState>) -> Result<Response> {
    let tags = Tag::find_all(ctx.db()).await?;

    Ok((StatusCode::OK, Json(tags)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn create_tag(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<CreateProductTag>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;
    let tag = Tag::create(ctx.db(), validated.name()).await?;

    Ok((StatusCode::CREATED, Json(tag)).into_response())
}

fn protected(ctx: &AppState) -> Router {
    Router::new()
        .route(
            "/attributes",
            get(attributes).layer(RbacLayer::new(ctx.clone(), permissions::products::READ)),
        )
        .route(
            "/tags",
            get(tags).layer(RbacLayer::new(ctx.clone(), permissions::products::READ)),
        )
        .route(
            "/tags",
            post(create_tag).layer(RbacLayer::new(ctx.clone(), permissions::products::CREATE)),
        )
        .route(
            "/",
            post(create).layer(RbacLayer::new(ctx.clone(), permissions::products::CREATE)),
        )
        .route(
            "/{pid}",
            patch(update).layer(RbacLayer::new(ctx.clone(), permissions::products::UPDATE)),
        )
        .route(
            "/{pid}",
            delete(remove).layer(RbacLayer::new(ctx.clone(), permissions::products::DELETE)),
        )
        .route(
            "/{pid}/variants",
            post(create_variant).layer(RbacLayer::new(ctx.clone(), permissions::products::UPDATE)),
        )
        .route(
            "/{pid}/variants/{variant_pid}",
            patch(update_variant).layer(RbacLayer::new(ctx.clone(), permissions::products::UPDATE)),
        )
        .route(
            "/{pid}/variants/{variant_pid}",
            delete(remove_variant)
                .layer(RbacLayer::new(ctx.clone(), permissions::products::DELETE)),
        )
        .route(
            "/{pid}/variants/{variant_pid}/default",
            post(set_default_variant)
                .layer(RbacLayer::new(ctx.clone(), permissions::products::UPDATE)),
        )
        .route(
            "/{pid}/pictures",
            post(add_picture).layer(RbacLayer::new(ctx.clone(), permissions::products::UPDATE)),
        )
        .route(
            "/{pid}/pictures/{picture_pid}",
            patch(update_picture).layer(RbacLayer::new(ctx.clone(), permissions::products::UPDATE)),
        )
        .route(
            "/{pid}/pictures/{picture_pid}",
            delete(remove_picture)
                .layer(RbacLayer::new(ctx.clone(), permissions::products::DELETE)),
        )
        .route(
            "/{pid}/variants/{variant_pid}/pictures",
            post(add_variant_picture)
                .layer(RbacLayer::new(ctx.clone(), permissions::products::UPDATE)),
        )
        .route(
            "/{pid}/variants/{variant_pid}/pictures/{picture_pid}",
            patch(update_variant_picture)
                .layer(RbacLayer::new(ctx.clone(), permissions::products::UPDATE)),
        )
        .route(
            "/{pid}/variants/{variant_pid}/pictures/{picture_pid}",
            delete(remove_variant_picture)
                .layer(RbacLayer::new(ctx.clone(), permissions::products::DELETE)),
        )
        .with_state(ctx.clone())
}

fn general(ctx: &AppState) -> Router {
    Router::new()
        .route("/", get(list))
        .route("/{pid}", get(one))
        .with_state(ctx.clone())
}

pub fn router(ctx: &AppState) -> Router {
    general(ctx).merge(protected(ctx).layer(AuthLayer::new(ctx.clone())))
}
