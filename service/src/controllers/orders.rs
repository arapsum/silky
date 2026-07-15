use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    AppState, Error, Result,
    access_control::permissions,
    context::Claims,
    middlewares::{AuthLayer, RbacLayer},
    models::{CheckoutSessionDetails, Order, PaymentAttempt, User},
    payments::create_hosted_checkout_session,
    schemas::{CheckoutOrder, OrderListQuery, UpdateOrder, Validator},
    utils::{AppExtension, AppJson, AppPath, AppQuery},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckoutResponse {
    order_pid: Uuid,
    checkout_url: String,
    expires_at: chrono::DateTime<chrono::FixedOffset>,
}

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

#[tracing::instrument(skip(ctx, claims, params))]
#[debug_handler]
async fn checkout(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
    AppJson(params): AppJson<CheckoutOrder>,
) -> Result<Response> {
    let stripe = ctx
        .stripe()
        .ok_or(crate::error::PaymentError::NotConfigured)?;
    let stripe_config = ctx
        .config()
        .stripe()
        .ok_or(crate::error::PaymentError::NotConfigured)?;
    let customer_pid = Uuid::parse_str(claims.sub()).map_err(|_| Error::Forbidden)?;
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let mut txn = ctx.db().begin().await?;
    let order = Order::create_checkout_order(&mut txn, customer_pid, validated).await?;
    let attempt = PaymentAttempt::create_for_order(
        &mut txn,
        order.order.row_id(),
        order.order.grand_total(),
        order.order.currency(),
    )
    .await?;
    txn.commit().await?;

    let session =
        match create_hosted_checkout_session(stripe, stripe_config, &order, &attempt).await {
            Ok(session) => session,
            Err(error) => {
                if error.should_compensate_checkout() {
                    let mut txn = ctx.db().begin().await?;
                    PaymentAttempt::cancel_and_release_inventory(
                        &mut txn,
                        attempt.pid(),
                        Some(error.code()),
                        None,
                    )
                    .await?;
                    txn.commit().await?;
                }
                return Err(Error::Payment(error).into());
            }
        };

    let mut txn = ctx.db().begin().await?;
    PaymentAttempt::attach_checkout_session(
        &mut txn,
        attempt.pid(),
        &CheckoutSessionDetails {
            session_id: session.session_id(),
            checkout_url: session.checkout_url(),
            expires_at: session.expires_at(),
        },
    )
    .await?;
    txn.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(CheckoutResponse {
            order_pid: order.order.pid(),
            checkout_url: session.checkout_url().to_owned(),
            expires_at: session.expires_at(),
        }),
    )
        .into_response())
}

#[tracing::instrument(skip(ctx, claims))]
#[debug_handler]
async fn checkout_session(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
    AppPath(pid): AppPath<Uuid>,
) -> Result<Response> {
    let customer_pid = Uuid::parse_str(claims.sub()).map_err(|_| Error::Forbidden)?;
    let session =
        PaymentAttempt::find_checkout_session_for_customer(ctx.db(), pid, customer_pid).await?;

    Ok((StatusCode::OK, Json(session)).into_response())
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route(
            "/checkout",
            post(checkout).layer(RbacLayer::customers_only(ctx.clone())),
        )
        .route(
            "/{pid}/checkout-session",
            get(checkout_session).layer(RbacLayer::customers_only(ctx.clone())),
        )
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
