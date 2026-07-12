use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch},
};
use uuid::Uuid;

use crate::{
    AppState, Error, Result,
    access_control::permissions,
    context::Claims,
    middlewares::{AuthLayer, RbacLayer},
    models::{Order, User},
    schemas::{OrderListQuery, UpdateOrder, Validator},
    utils::{AppExtension, AppJson, AppPath, AppQuery},
};

async fn actor(ctx: &AppState, claims: &Claims) -> Result<(Uuid, bool)> {
    let pid = Uuid::parse_str(claims.sub()).map_err(|_| Error::Forbidden)?;
    let is_customer = User::has_role(ctx.db(), pid, "customer").await?;
    Ok((pid, is_customer))
}

#[tracing::instrument(skip(ctx, claims))]
#[debug_handler]
async fn list(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
    AppQuery(mut query): AppQuery<OrderListQuery>,
) -> Result<Response> {
    let (actor_pid, is_customer) = actor(&ctx, &claims).await?;
    if is_customer {
        query = query.with_customer_pid(actor_pid);
    }

    let validator = Validator::new(query);
    let validated = validator.validate()?;
    let orders = Order::find_all(ctx.db(), validated).await?;
    Ok((StatusCode::OK, Json(orders)).into_response())
}

#[tracing::instrument(skip(ctx, claims))]
#[debug_handler]
async fn one(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
    AppPath(pid): AppPath<Uuid>,
) -> Result<Response> {
    let (actor_pid, is_customer) = actor(&ctx, &claims).await?;
    let order = if is_customer {
        Order::find_detail_for_customer(ctx.db(), pid, actor_pid).await?
    } else {
        Order::find_detail_by_pid(ctx.db(), pid).await?
    };

    Ok((StatusCode::OK, Json(order)).into_response())
}

#[tracing::instrument(skip(ctx, params))]
#[debug_handler]
async fn update(
    State(ctx): State<AppState>,
    AppPath(pid): AppPath<Uuid>,
    AppJson(params): AppJson<UpdateOrder>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;
    let order = Order::update(ctx.db(), pid, validated).await?;
    Ok((StatusCode::OK, Json(order)).into_response())
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(list)
                .layer(RbacLayer::new(ctx.clone(), permissions::orders::READ).allow_customers()),
        )
        .route(
            "/{pid}",
            get(one)
                .layer(RbacLayer::new(ctx.clone(), permissions::orders::READ).allow_customers()),
        )
        .route(
            "/{pid}",
            patch(update)
                .layer(RbacLayer::new(ctx.clone(), permissions::orders::UPDATE).deny_customers()),
        )
        .with_state(ctx.clone())
        .layer(AuthLayer::new(ctx.clone()))
}
