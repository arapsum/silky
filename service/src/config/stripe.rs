use std::fmt;

use serde::Deserialize;

const DEFAULT_CHECKOUT_TTL_SECONDS: i64 = 30 * 60;

/// Runtime settings for Stripe-hosted Checkout and signed webhooks.
///
/// Secret values are loaded from the process environment and deliberately
/// redacted from debug output. Stripe remains disabled when any required value
/// is missing, allowing non-payment commands and tests to run without Stripe.
#[derive(Clone, Deserialize)]
pub struct StripeConfig {
    #[serde(default)]
    secret_key: String,
    #[serde(default)]
    webhook_secret: String,
    #[serde(default)]
    checkout_success_url: String,
    #[serde(default)]
    checkout_cancel_url: String,
    #[serde(default = "default_checkout_ttl_seconds")]
    checkout_ttl_seconds: i64,
    #[serde(default)]
    live_mode: bool,
}

impl StripeConfig {
    pub(super) fn apply_environment(&mut self) {
        if let Ok(value) = std::env::var("APP_STRIPE_SECRET_KEY") {
            self.secret_key = value;
        }
        if let Ok(value) = std::env::var("APP_STRIPE_WEBHOOK_SECRET") {
            self.webhook_secret = value;
        }
        if let Ok(value) = std::env::var("APP_STRIPE_CHECKOUT_SUCCESS_URL") {
            self.checkout_success_url = value;
        }
        if let Ok(value) = std::env::var("APP_STRIPE_CHECKOUT_CANCEL_URL") {
            self.checkout_cancel_url = value;
        }
        if let Ok(value) = std::env::var("APP_STRIPE_CHECKOUT_TTL_SECONDS")
            && let Ok(value) = value.parse()
        {
            self.checkout_ttl_seconds = value;
        }
        if let Ok(value) = std::env::var("APP_STRIPE_LIVE_MODE")
            && let Ok(value) = value.parse()
        {
            self.live_mode = value;
        }
    }

    pub(super) fn is_configured(&self) -> bool {
        !self.secret_key.trim().is_empty()
            && !self.webhook_secret.trim().is_empty()
            && !self.checkout_success_url.trim().is_empty()
            && !self.checkout_cancel_url.trim().is_empty()
    }

    #[must_use]
    pub fn secret_key(&self) -> &str {
        &self.secret_key
    }

    #[must_use]
    pub fn webhook_secret(&self) -> &str {
        &self.webhook_secret
    }

    #[must_use]
    pub fn checkout_success_url(&self) -> &str {
        &self.checkout_success_url
    }

    #[must_use]
    pub fn checkout_cancel_url(&self) -> &str {
        &self.checkout_cancel_url
    }

    #[must_use]
    pub const fn checkout_ttl_seconds(&self) -> i64 {
        self.checkout_ttl_seconds
    }

    #[must_use]
    pub const fn live_mode(&self) -> bool {
        self.live_mode
    }
}

impl fmt::Debug for StripeConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StripeConfig")
            .field("secret_key", &"[REDACTED]")
            .field("webhook_secret", &"[REDACTED]")
            .field("checkout_success_url", &self.checkout_success_url)
            .field("checkout_cancel_url", &self.checkout_cancel_url)
            .field("checkout_ttl_seconds", &self.checkout_ttl_seconds)
            .field("live_mode", &self.live_mode)
            .finish()
    }
}

const fn default_checkout_ttl_seconds() -> i64 {
    DEFAULT_CHECKOUT_TTL_SECONDS
}
