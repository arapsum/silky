use axum::{
    Router,
    body::Bytes,
    debug_handler,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};

use crate::{
    AppState,
    error::PaymentError,
    payments::{process_stripe_event, verify_stripe_event},
};

#[tracing::instrument(skip(ctx, headers, body))]
#[debug_handler]
async fn stripe_webhook(
    State(ctx): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> crate::Result<StatusCode> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|value| value.to_str().ok())
        .ok_or(PaymentError::MissingWebhookSignature)?;
    let payload = std::str::from_utf8(&body).map_err(|_| PaymentError::InvalidWebhookPayload)?;
    let stripe = ctx.stripe().ok_or(PaymentError::NotConfigured)?;
    let config = ctx.config().stripe().ok_or(PaymentError::NotConfigured)?;
    let event = verify_stripe_event(payload, signature, stripe.webhook_secret())?;
    let parsed_payload =
        serde_json::from_str(payload).map_err(|_| PaymentError::InvalidWebhookPayload)?;

    Box::pin(process_stripe_event(
        ctx.db(),
        config,
        event,
        &parsed_payload,
    ))
    .await?;
    Ok(StatusCode::OK)
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route("/stripe/webhook", post(stripe_webhook))
        .with_state(ctx.clone())
}
