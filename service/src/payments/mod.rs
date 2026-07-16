use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::Serialize;
use stripe::{
    CheckoutSession, CheckoutSessionId, CheckoutSessionMode, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateCheckoutSessionLineItemsPriceData,
    CreateCheckoutSessionLineItemsPriceDataProductData, Currency, Event, EventObject, EventType,
    RequestStrategy,
};

/// Expires an active Stripe Checkout Session before Silk releases inventory.
///
/// # Errors
/// Returns an invalid-provider-response error for a malformed persisted ID, or
/// a classified Stripe error when the provider cannot expire the Session.
pub async fn expire_hosted_checkout_session(
    stripe: &StripeContext,
    session_id: &str,
) -> Result<(), PaymentError> {
    let session_id = session_id
        .parse::<CheckoutSessionId>()
        .map_err(|_| PaymentError::InvalidProviderResponse)?;
    CheckoutSession::expire(stripe.client(), &session_id)
        .await
        .map_err(PaymentError::from_stripe)?;
    Ok(())
}

use crate::{
    config::StripeConfig,
    context::StripeContext,
    error::PaymentError,
    models::{OrderWithItems, PaymentAttempt, StripeWebhookEvent},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostedCheckoutSession {
    session_id: String,
    checkout_url: String,
    expires_at: DateTime<FixedOffset>,
}

impl HostedCheckoutSession {
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    #[must_use]
    pub fn checkout_url(&self) -> &str {
        &self.checkout_url
    }

    #[must_use]
    pub const fn expires_at(&self) -> DateTime<FixedOffset> {
        self.expires_at
    }
}

/// Converts a decimal amount into the integer minor unit expected by Stripe.
///
/// Silk checkout currently supports USD, whose minor unit is one cent. Values
/// with fractions of a cent are rejected instead of silently rounded.
///
/// # Errors
/// Returns an unsupported-currency or invalid-amount error when the value
/// cannot be represented exactly as a non-negative 64-bit minor-unit amount.
pub fn to_minor_units(amount: Decimal, currency: &str) -> Result<i64, PaymentError> {
    if !currency.eq_ignore_ascii_case("USD") {
        return Err(PaymentError::UnsupportedCurrency(currency.to_owned()));
    }
    if amount.is_sign_negative() {
        return Err(PaymentError::InvalidAmount);
    }

    let minor_units = amount * Decimal::ONE_HUNDRED;
    if !minor_units.fract().is_zero() {
        return Err(PaymentError::InvalidAmount);
    }

    minor_units.to_i64().ok_or(PaymentError::InvalidAmount)
}

/// Creates a Stripe-hosted Checkout Session for a persisted order attempt.
///
/// Product names, quantities, and prices come exclusively from the order
/// snapshots. The payment attempt PID is used as Stripe's idempotency key so a
/// retry cannot create a second Session for the same attempt.
///
/// # Errors
/// Returns a validation error for unsupported monetary values, a classified
/// Stripe communication error, or an invalid-response error when Stripe omits
/// the hosted checkout URL.
pub async fn create_hosted_checkout_session(
    stripe: &StripeContext,
    config: &StripeConfig,
    checkout: &OrderWithItems,
    attempt: &PaymentAttempt,
) -> Result<HostedCheckoutSession, PaymentError> {
    let currency = stripe_currency(checkout.order.currency())?;
    let mut line_items = Vec::with_capacity(checkout.items.len());
    for item in &checkout.items {
        let quantity = u64::try_from(item.quantity()).map_err(|_| PaymentError::InvalidAmount)?;
        line_items.push(CreateCheckoutSessionLineItems {
            price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                currency,
                product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
                    name: item.product_name().to_owned(),
                    metadata: Some(HashMap::from([
                        ("variant_pid".to_string(), item.variant_pid().to_string()),
                        ("sku".to_string(), item.sku().to_owned()),
                    ])),
                    ..Default::default()
                }),
                unit_amount: Some(to_minor_units(
                    item.unit_price(),
                    checkout.order.currency(),
                )?),
                ..Default::default()
            }),
            quantity: Some(quantity),
            ..Default::default()
        });
    }

    let order_pid = checkout.order.pid().to_string();
    let attempt_pid = attempt.pid().to_string();
    let success_url = config
        .checkout_success_url()
        .replace("{ORDER_PID}", &order_pid);
    let cancel_url = config
        .checkout_cancel_url()
        .replace("{ORDER_PID}", &order_pid);
    let expires_at = Utc::now().timestamp() + config.checkout_ttl_seconds();
    let mut params = CreateCheckoutSession::new();
    params.cancel_url = Some(&cancel_url);
    params.client_reference_id = Some(&order_pid);
    params.customer_email = Some(checkout.order.customer_email());
    params.expires_at = Some(expires_at);
    params.line_items = Some(line_items);
    params.metadata = Some(HashMap::from([
        ("order_pid".to_string(), order_pid.clone()),
        ("payment_attempt_pid".to_string(), attempt_pid.clone()),
    ]));
    params.mode = Some(CheckoutSessionMode::Payment);
    params.success_url = Some(&success_url);

    let client = stripe
        .client()
        .clone()
        .with_strategy(RequestStrategy::Idempotent(format!(
            "silk-checkout-{attempt_pid}"
        )));
    let session = CheckoutSession::create(&client, params)
        .await
        .map_err(PaymentError::from_stripe)?;
    hosted_session(session)
}

fn stripe_currency(currency: &str) -> Result<Currency, PaymentError> {
    if currency.eq_ignore_ascii_case("USD") {
        Ok(Currency::USD)
    } else {
        Err(PaymentError::UnsupportedCurrency(currency.to_owned()))
    }
}

fn hosted_session(session: CheckoutSession) -> Result<HostedCheckoutSession, PaymentError> {
    let checkout_url = session.url.ok_or(PaymentError::InvalidProviderResponse)?;
    let expires_at = DateTime::from_timestamp(session.expires_at, 0)
        .ok_or(PaymentError::InvalidProviderResponse)?
        .fixed_offset();

    Ok(HostedCheckoutSession {
        session_id: session.id.to_string(),
        checkout_url,
        expires_at,
    })
}

/// Applies a verified Stripe event once inside a database transaction.
///
/// Supported Checkout Session events are reconciled against Silk's persisted
/// attempt, order, amount, currency, and runtime mode before state changes.
///
/// # Errors
/// Returns a retryable webhook error when persistence or reconciliation fails.
pub async fn process_stripe_event(
    db: &sqlx::PgPool,
    config: &StripeConfig,
    event: Event,
    payload: &serde_json::Value,
) -> Result<(), PaymentError> {
    if event.livemode != config.live_mode() {
        return Err(PaymentError::WebhookReconciliation("event mode mismatch"));
    }

    let mut txn = db
        .begin()
        .await
        .map_err(|error| PaymentError::WebhookProcessing(error.to_string()))?;
    let event_id = event.id.to_string();
    let event_type = event.type_.to_string();
    let is_new = StripeWebhookEvent::record_once(
        &mut txn,
        &event_id,
        &event_type,
        event.api_version.as_deref(),
        payload,
    )
    .await
    .map_err(|error| PaymentError::WebhookProcessing(error.to_string()))?;
    if !is_new {
        txn.commit()
            .await
            .map_err(|error| PaymentError::WebhookProcessing(error.to_string()))?;
        return Ok(());
    }

    match (event.type_, event.data.object) {
        (
            EventType::CheckoutSessionCompleted | EventType::CheckoutSessionAsyncPaymentSucceeded,
            EventObject::CheckoutSession(session),
        ) => process_completed(&mut txn, config, &session).await?,
        (EventType::CheckoutSessionExpired, EventObject::CheckoutSession(session)) => {
            reconcile_session(&mut txn, config, &session).await?;
            PaymentAttempt::expire_and_release_inventory(&mut txn, session.id.as_ref())
                .await
                .map_err(|error| PaymentError::WebhookProcessing(error.to_string()))?;
        }
        (EventType::CheckoutSessionAsyncPaymentFailed, EventObject::CheckoutSession(session)) => {
            reconcile_session(&mut txn, config, &session).await?;
            PaymentAttempt::fail_and_release_inventory(
                &mut txn,
                session.id.as_ref(),
                Some("async_payment_failed"),
                None,
            )
            .await
            .map_err(|error| PaymentError::WebhookProcessing(error.to_string()))?;
        }
        _ => {}
    }

    txn.commit()
        .await
        .map_err(|error| PaymentError::WebhookProcessing(error.to_string()))
}

async fn process_completed(
    txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &StripeConfig,
    session: &CheckoutSession,
) -> Result<(), PaymentError> {
    reconcile_session(txn, config, session).await?;
    let session_id = session.id.to_string();
    if session.payment_status == stripe::CheckoutSessionPaymentStatus::Paid {
        let payment_intent_id = session
            .payment_intent
            .as_ref()
            .map(stripe::Expandable::id)
            .map(|id| id.to_string());
        PaymentAttempt::mark_succeeded(txn, &session_id, payment_intent_id.as_deref())
            .await
            .map_err(|error| PaymentError::WebhookProcessing(error.to_string()))?;
    } else {
        PaymentAttempt::mark_processing(txn, &session_id)
            .await
            .map_err(|error| PaymentError::WebhookProcessing(error.to_string()))?;
    }
    Ok(())
}

async fn reconcile_session(
    txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &StripeConfig,
    session: &CheckoutSession,
) -> Result<(), PaymentError> {
    if session.livemode != config.live_mode() {
        return Err(PaymentError::WebhookReconciliation("session mode mismatch"));
    }
    if session.mode != CheckoutSessionMode::Payment {
        return Err(PaymentError::WebhookReconciliation(
            "checkout mode mismatch",
        ));
    }

    let metadata = session
        .metadata
        .as_ref()
        .ok_or(PaymentError::WebhookReconciliation("missing metadata"))?;
    let attempt_pid = parse_metadata_pid(metadata, "payment_attempt_pid")?;
    let order_pid = parse_metadata_pid(metadata, "order_pid")?;
    let expires_at = DateTime::from_timestamp(session.expires_at, 0)
        .ok_or(PaymentError::WebhookReconciliation(
            "invalid session expiry",
        ))?
        .fixed_offset();
    let session_id = session.id.to_string();
    PaymentAttempt::recover_checkout_session(
        txn,
        attempt_pid,
        &session_id,
        session.url.as_deref(),
        expires_at,
    )
    .await
    .map_err(|error| PaymentError::WebhookProcessing(error.to_string()))?;

    let stored = PaymentAttempt::reconciliation_for_session(txn, &session_id)
        .await
        .map_err(|error| PaymentError::WebhookProcessing(error.to_string()))?;
    if stored.attempt_pid != attempt_pid || stored.order_pid != order_pid {
        return Err(PaymentError::WebhookReconciliation("metadata mismatch"));
    }
    let order_reference = order_pid.to_string();
    if session.client_reference_id.as_deref() != Some(order_reference.as_str()) {
        return Err(PaymentError::WebhookReconciliation(
            "client reference mismatch",
        ));
    }
    if session.amount_total != Some(to_minor_units(stored.amount, &stored.currency)?) {
        return Err(PaymentError::WebhookReconciliation("amount mismatch"));
    }
    let session_currency = session.currency.map(|currency| currency.to_string());
    if session_currency
        .as_deref()
        .map(str::to_uppercase)
        .as_deref()
        != Some(stored.currency.as_str())
    {
        return Err(PaymentError::WebhookReconciliation("currency mismatch"));
    }
    Ok(())
}

fn parse_metadata_pid(
    metadata: &HashMap<String, String>,
    key: &'static str,
) -> Result<uuid::Uuid, PaymentError> {
    metadata
        .get(key)
        .and_then(|value| uuid::Uuid::parse_str(value).ok())
        .ok_or(PaymentError::WebhookReconciliation(key))
}

/// Verifies a Stripe signature against the exact raw request payload.
///
/// # Errors
/// Returns a stable invalid-signature error for malformed, expired, or
/// mismatched signatures.
pub fn verify_stripe_event(
    payload: &str,
    signature: &str,
    webhook_secret: &str,
) -> Result<Event, PaymentError> {
    stripe::Webhook::construct_event(payload, signature, webhook_secret)
        .map_err(|_| PaymentError::InvalidWebhookSignature)
}
