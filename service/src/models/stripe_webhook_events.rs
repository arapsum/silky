use serde::Serialize;
use serde_json::Value as JsonValue;
use sqlx::{FromRow, Postgres, Transaction, types::Json};

use super::ModelResult;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct StripeWebhookEvent {
    stripe_event_id: String,
    event_type: String,
    api_version: Option<String>,
    payload: Json<JsonValue>,
    processed_at: chrono::DateTime<chrono::FixedOffset>,
    created_at: chrono::DateTime<chrono::FixedOffset>,
}

impl StripeWebhookEvent {
    /// Records a verified Stripe event as the transaction's idempotency guard.
    ///
    /// The return value is `true` only for the first delivery. Duplicate event
    /// IDs return `false` and should be acknowledged without reprocessing.
    ///
    /// # Errors
    /// Returns a database error when the event cannot be inserted.
    pub async fn record_once(
        txn: &mut Transaction<'_, Postgres>,
        event_id: &str,
        event_type: &str,
        api_version: Option<&str>,
        payload: &JsonValue,
    ) -> ModelResult<bool> {
        let inserted = sqlx::query_scalar::<_, String>(
            r"INSERT INTO stripe_webhook_events (
                  stripe_event_id,
                  event_type,
                  api_version,
                  payload
              )
              VALUES ($1, $2, $3, $4)
              ON CONFLICT (stripe_event_id) DO NOTHING
              RETURNING stripe_event_id",
        )
        .bind(event_id)
        .bind(event_type)
        .bind(api_version)
        .bind(Json(payload))
        .fetch_optional(&mut **txn)
        .await?;

        Ok(inserted.is_some())
    }

    /// Finds a previously recorded event by its Stripe identifier.
    ///
    /// # Errors
    /// Returns a database error when the lookup fails.
    pub async fn find_by_id(db: &sqlx::PgPool, event_id: &str) -> ModelResult<Option<Self>> {
        Ok(sqlx::query_as::<_, Self>(
            "SELECT * FROM stripe_webhook_events WHERE stripe_event_id = $1",
        )
        .bind(event_id)
        .fetch_optional(db)
        .await?)
    }
}
