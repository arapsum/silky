use axum::http::StatusCode;

/// Failures raised while preparing or communicating with the payment provider.
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Stripe payment processing is not configured")]
    NotConfigured,
    #[error("unsupported checkout currency: {0}")]
    UnsupportedCurrency(String),
    #[error("checkout amount cannot be represented in the currency's minor unit")]
    InvalidAmount,
    #[error("Stripe rejected the checkout session request: {0}")]
    ProviderRejected(#[source] stripe::StripeError),
    #[error("Stripe could not be reached: {0}")]
    ProviderUnavailable(#[source] stripe::StripeError),
    #[error("Stripe returned a checkout session without a redirect URL")]
    InvalidProviderResponse,
    #[error("the Stripe signature header is missing")]
    MissingWebhookSignature,
    #[error("the Stripe webhook signature is invalid")]
    InvalidWebhookSignature,
    #[error("the Stripe webhook payload is invalid")]
    InvalidWebhookPayload,
    #[error("Stripe webhook reconciliation failed: {0}")]
    WebhookReconciliation(&'static str),
    #[error("Stripe webhook processing failed: {0}")]
    WebhookProcessing(String),
}

impl PaymentError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::NotConfigured => "stripe_not_configured",
            Self::UnsupportedCurrency(_) => "unsupported_checkout_currency",
            Self::InvalidAmount => "invalid_checkout_amount",
            Self::ProviderRejected(_) => "checkout_session_rejected",
            Self::ProviderUnavailable(_) => "payment_provider_unavailable",
            Self::InvalidProviderResponse => "invalid_payment_provider_response",
            Self::MissingWebhookSignature | Self::InvalidWebhookSignature => {
                "invalid_stripe_signature"
            }
            Self::InvalidWebhookPayload => "invalid_stripe_payload",
            Self::WebhookReconciliation(_) => "webhook_reconciliation_failed",
            Self::WebhookProcessing(_) => "webhook_processing_failed",
        }
    }

    #[must_use]
    pub const fn response_body(&self) -> (StatusCode, &'static str) {
        match self {
            Self::NotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Payments are temporarily unavailable. Please try again later.",
            ),
            Self::UnsupportedCurrency(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "This order's currency is not currently supported for checkout.",
            ),
            Self::InvalidAmount => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "One or more order prices cannot be processed in this currency.",
            ),
            Self::ProviderRejected(_) | Self::InvalidProviderResponse => (
                StatusCode::BAD_GATEWAY,
                "The payment provider could not create this checkout session.",
            ),
            Self::ProviderUnavailable(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "The payment provider is temporarily unavailable. Please try again.",
            ),
            Self::MissingWebhookSignature | Self::InvalidWebhookSignature => (
                StatusCode::BAD_REQUEST,
                "The Stripe webhook signature is missing or invalid.",
            ),
            Self::InvalidWebhookPayload => (
                StatusCode::BAD_REQUEST,
                "The Stripe webhook payload could not be verified.",
            ),
            Self::WebhookReconciliation(_) | Self::WebhookProcessing(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "The Stripe event could not be processed and will be retried.",
            ),
        }
    }

    /// Classifies a Stripe API failure without exposing provider internals to clients.
    #[must_use]
    pub const fn from_stripe(error: stripe::StripeError) -> Self {
        match &error {
            stripe::StripeError::Timeout | stripe::StripeError::ClientError(_) => {
                Self::ProviderUnavailable(error)
            }
            stripe::StripeError::Stripe(request) if request.http_status == 429 => {
                Self::ProviderUnavailable(error)
            }
            _ => Self::ProviderRejected(error),
        }
    }

    #[must_use]
    pub const fn should_compensate_checkout(&self) -> bool {
        matches!(
            self,
            Self::UnsupportedCurrency(_) | Self::InvalidAmount | Self::ProviderRejected(_)
        )
    }
}
