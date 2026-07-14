use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::Serialize;
use stripe::{
    CheckoutSession, CheckoutSessionMode, CreateCheckoutSession, CreateCheckoutSessionLineItems,
    CreateCheckoutSessionLineItemsPriceData, CreateCheckoutSessionLineItemsPriceDataProductData,
    Currency, RequestStrategy,
};

use crate::{
    config::StripeConfig,
    context::StripeContext,
    error::PaymentError,
    models::{OrderWithItems, PaymentAttempt},
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
    let expires_at = Utc::now().timestamp() + config.checkout_ttl_seconds();
    let mut params = CreateCheckoutSession::new();
    params.cancel_url = Some(config.checkout_cancel_url());
    params.client_reference_id = Some(&order_pid);
    params.customer_email = Some(checkout.order.customer_email());
    params.expires_at = Some(expires_at);
    params.line_items = Some(line_items);
    params.metadata = Some(HashMap::from([
        ("order_pid".to_string(), order_pid.clone()),
        ("payment_attempt_pid".to_string(), attempt_pid.clone()),
    ]));
    params.mode = Some(CheckoutSessionMode::Payment);
    params.success_url = Some(config.checkout_success_url());

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
