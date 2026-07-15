use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// Customer-facing state for the latest hosted Checkout Session on an order.
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutSessionResponse {
    order_pid: Uuid,
    order_status: String,
    payment_status: String,
    attempt_status: String,
    checkout_url: Option<String>,
    expires_at: Option<DateTime<FixedOffset>>,
}
