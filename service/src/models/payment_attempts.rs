use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use super::{ModelError, ModelResult};

const ACTIVE_STATUSES: &[&str] = &["initiated", "session_created", "processing"];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentAttempt {
    id: i32,
    pid: Uuid,
    order_id: i32,
    provider: String,
    status: String,
    amount: Decimal,
    currency: String,
    stripe_checkout_session_id: Option<String>,
    stripe_payment_intent_id: Option<String>,
    checkout_url: Option<String>,
    expires_at: Option<DateTime<FixedOffset>>,
    failure_code: Option<String>,
    failure_message: Option<String>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    completed_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone)]
pub struct CheckoutSessionDetails<'a> {
    pub session_id: &'a str,
    pub checkout_url: &'a str,
    pub expires_at: DateTime<FixedOffset>,
}

impl PaymentAttempt {
    /// Creates the sole active payment attempt for an order.
    ///
    /// The order amount and currency are copied into the attempt so webhook
    /// processing can reconcile Stripe's values without trusting event metadata.
    ///
    /// # Errors
    /// Returns [`ModelError::PaymentInProgress`] when the order already has an
    /// active attempt, or a database error when the attempt cannot be persisted.
    pub async fn create_for_order(
        txn: &mut Transaction<'_, Postgres>,
        order_id: i32,
        amount: Decimal,
        currency: &str,
    ) -> ModelResult<Self> {
        Ok(sqlx::query_as::<_, Self>(
            r"INSERT INTO payment_attempts (order_id, amount, currency)
              VALUES ($1, $2, $3)
              RETURNING *",
        )
        .bind(order_id)
        .bind(amount)
        .bind(currency)
        .fetch_one(&mut **txn)
        .await?)
    }

    /// Stores the Stripe-hosted Checkout Session created for this attempt.
    ///
    /// Reattaching the same Session is an idempotent no-op. Any different or
    /// terminal-state transition is rejected.
    ///
    /// # Errors
    /// Returns an attempt-not-found or invalid-transition error, or a database
    /// error when the row cannot be locked or updated.
    pub async fn attach_checkout_session(
        txn: &mut Transaction<'_, Postgres>,
        attempt_pid: Uuid,
        session: &CheckoutSessionDetails<'_>,
    ) -> ModelResult<Self> {
        let attempt = Self::lock_by_pid(txn, attempt_pid).await?;
        if attempt.status == "session_created"
            && attempt.stripe_checkout_session_id.as_deref() == Some(session.session_id)
        {
            return Ok(attempt);
        }
        attempt.require_status(&["initiated"], "session_created")?;

        Ok(sqlx::query_as::<_, Self>(
            r"UPDATE payment_attempts
              SET status = 'session_created',
                  stripe_checkout_session_id = $2,
                  checkout_url = $3,
                  expires_at = $4
              WHERE pid = $1
              RETURNING *",
        )
        .bind(attempt_pid)
        .bind(session.session_id)
        .bind(session.checkout_url)
        .bind(session.expires_at)
        .fetch_one(&mut **txn)
        .await?)
    }

    /// Marks a Checkout Session as waiting for delayed payment confirmation.
    ///
    /// # Errors
    /// Returns an attempt-not-found or invalid-transition error, or a database
    /// error when the attempt cannot be updated.
    pub async fn mark_processing(
        txn: &mut Transaction<'_, Postgres>,
        session_id: &str,
    ) -> ModelResult<Self> {
        let attempt = Self::lock_by_session(txn, session_id).await?;
        if attempt.status == "processing" {
            return Ok(attempt);
        }
        attempt.require_status(&["session_created"], "processing")?;
        Self::set_status(txn, attempt.pid, "processing").await
    }

    /// Marks an attempt successful and confirms its linked order atomically.
    ///
    /// Replaying success for an already successful attempt is idempotent.
    /// Fulfilment is deliberately left unchanged.
    ///
    /// # Errors
    /// Returns an attempt-not-found or invalid-transition error, or a database
    /// error when the attempt or order cannot be updated.
    pub async fn mark_succeeded(
        txn: &mut Transaction<'_, Postgres>,
        session_id: &str,
        payment_intent_id: Option<&str>,
    ) -> ModelResult<Self> {
        let attempt = Self::lock_by_session(txn, session_id).await?;
        if attempt.status == "succeeded" {
            return Ok(attempt);
        }
        attempt.require_status(&["session_created", "processing"], "succeeded")?;

        let updated = sqlx::query_as::<_, Self>(
            r"UPDATE payment_attempts
              SET status = 'succeeded',
                  stripe_payment_intent_id = COALESCE($2, stripe_payment_intent_id),
                  failure_code = NULL,
                  failure_message = NULL,
                  completed_at = now()
              WHERE pid = $1
              RETURNING *",
        )
        .bind(attempt.pid)
        .bind(payment_intent_id)
        .fetch_one(&mut **txn)
        .await?;

        sqlx::query(
            r"UPDATE orders
              SET status = 'confirmed', payment_status = 'paid'
              WHERE id = $1",
        )
        .bind(attempt.order_id)
        .execute(&mut **txn)
        .await?;

        Ok(updated)
    }

    /// Records a definitive provider failure without releasing inventory.
    ///
    /// Inventory remains reserved because a failed asynchronous payment can be
    /// retried through a fresh attempt after the order is reconciled.
    ///
    /// # Errors
    /// Returns an attempt-not-found or invalid-transition error, or a database
    /// error when the attempt cannot be updated.
    pub async fn mark_failed(
        txn: &mut Transaction<'_, Postgres>,
        session_id: &str,
        code: Option<&str>,
        message: Option<&str>,
    ) -> ModelResult<Self> {
        let attempt = Self::lock_by_session(txn, session_id).await?;
        if attempt.status == "failed" {
            return Ok(attempt);
        }
        attempt.require_status(&["session_created", "processing"], "failed")?;

        let updated = sqlx::query_as::<_, Self>(
            r"UPDATE payment_attempts
              SET status = 'failed', failure_code = $2, failure_message = $3
              WHERE pid = $1
              RETURNING *",
        )
        .bind(attempt.pid)
        .bind(code)
        .bind(message)
        .fetch_one(&mut **txn)
        .await?;

        sqlx::query("UPDATE orders SET payment_status = 'failed' WHERE id = $1")
            .bind(attempt.order_id)
            .execute(&mut **txn)
            .await?;

        Ok(updated)
    }

    /// Expires an unpaid attempt, cancels its order, and restores stock once.
    ///
    /// Replayed expiry events are idempotent. A successful payment is never
    /// reverted by a late or out-of-order expiry event.
    ///
    /// # Errors
    /// Returns an attempt-not-found or invalid-transition error, or a database
    /// error when locking, inventory restoration, or state updates fail.
    pub async fn expire_and_release_inventory(
        txn: &mut Transaction<'_, Postgres>,
        session_id: &str,
    ) -> ModelResult<Self> {
        let attempt = Self::lock_by_session(txn, session_id).await?;
        if matches!(attempt.status.as_str(), "expired" | "succeeded") {
            return Ok(attempt);
        }
        attempt.require_status(ACTIVE_STATUSES, "expired")?;

        restore_inventory(txn, attempt.order_id).await?;
        cancel_order(txn, attempt.order_id).await?;

        sqlx::query_as::<_, Self>(
            r"UPDATE payment_attempts
              SET status = 'expired', completed_at = now()
              WHERE pid = $1
              RETURNING *",
        )
        .bind(attempt.pid)
        .fetch_one(&mut **txn)
        .await
        .map_err(Into::into)
    }

    /// Cancels an initiated attempt after Stripe definitively rejects Session creation.
    ///
    /// # Errors
    /// Returns an attempt-not-found or invalid-transition error, or a database
    /// error when compensation cannot be completed.
    pub async fn cancel_and_release_inventory(
        txn: &mut Transaction<'_, Postgres>,
        attempt_pid: Uuid,
        code: Option<&str>,
        message: Option<&str>,
    ) -> ModelResult<Self> {
        let attempt = Self::lock_by_pid(txn, attempt_pid).await?;
        if attempt.status == "cancelled" {
            return Ok(attempt);
        }
        attempt.require_status(&["initiated"], "cancelled")?;

        restore_inventory(txn, attempt.order_id).await?;
        cancel_order(txn, attempt.order_id).await?;

        Ok(sqlx::query_as::<_, Self>(
            r"UPDATE payment_attempts
              SET status = 'cancelled',
                  failure_code = $2,
                  failure_message = $3,
                  completed_at = now()
              WHERE pid = $1
              RETURNING *",
        )
        .bind(attempt.pid)
        .bind(code)
        .bind(message)
        .fetch_one(&mut **txn)
        .await?)
    }

    /// Finds an attempt by its public identifier.
    ///
    /// # Errors
    /// Returns [`ModelError::PaymentAttemptNotFound`] when no row exists, or a
    /// database error when the query fails.
    pub async fn find_by_pid(db: &sqlx::PgPool, attempt_pid: Uuid) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>("SELECT * FROM payment_attempts WHERE pid = $1")
            .bind(attempt_pid)
            .fetch_optional(db)
            .await?
            .ok_or(ModelError::PaymentAttemptNotFound)
    }

    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    #[must_use]
    pub const fn order_id(&self) -> i32 {
        self.order_id
    }

    #[must_use]
    pub const fn amount(&self) -> Decimal {
        self.amount
    }

    #[must_use]
    pub fn currency(&self) -> &str {
        &self.currency
    }

    async fn lock_by_pid(
        txn: &mut Transaction<'_, Postgres>,
        attempt_pid: Uuid,
    ) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"SELECT *
              FROM payment_attempts
              WHERE pid = $1
              FOR UPDATE",
        )
        .bind(attempt_pid)
        .fetch_optional(&mut **txn)
        .await?
        .ok_or(ModelError::PaymentAttemptNotFound)
    }

    async fn lock_by_session(
        txn: &mut Transaction<'_, Postgres>,
        session_id: &str,
    ) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"SELECT *
              FROM payment_attempts
              WHERE stripe_checkout_session_id = $1
              FOR UPDATE",
        )
        .bind(session_id)
        .fetch_optional(&mut **txn)
        .await?
        .ok_or(ModelError::PaymentAttemptNotFound)
    }

    fn require_status(&self, allowed: &[&str], target: &'static str) -> ModelResult<()> {
        if allowed.contains(&self.status.as_str()) {
            Ok(())
        } else {
            Err(ModelError::InvalidPaymentTransition {
                current: self.status.clone(),
                target,
            })
        }
    }

    async fn set_status(
        txn: &mut Transaction<'_, Postgres>,
        attempt_pid: Uuid,
        status: &str,
    ) -> ModelResult<Self> {
        Ok(sqlx::query_as::<_, Self>(
            r"UPDATE payment_attempts
              SET status = $2
              WHERE pid = $1
              RETURNING *",
        )
        .bind(attempt_pid)
        .bind(status)
        .fetch_one(&mut **txn)
        .await?)
    }
}

async fn restore_inventory(txn: &mut Transaction<'_, Postgres>, order_id: i32) -> ModelResult<()> {
    sqlx::query(
        r"UPDATE product_variants variant
          SET stock_quantity = variant.stock_quantity + restored.quantity
          FROM (
              SELECT variant_id, SUM(quantity)::INTEGER AS quantity
              FROM order_items
              WHERE order_id = $1 AND variant_id IS NOT NULL
              GROUP BY variant_id
          ) restored
          WHERE variant.id = restored.variant_id",
    )
    .bind(order_id)
    .execute(&mut **txn)
    .await?;
    Ok(())
}

async fn cancel_order(txn: &mut Transaction<'_, Postgres>, order_id: i32) -> ModelResult<()> {
    sqlx::query(
        r"UPDATE orders
          SET status = 'cancelled',
              payment_status = 'failed',
              fulfillment_status = 'cancelled',
              cancelled_at = COALESCE(cancelled_at, now())
          WHERE id = $1",
    )
    .bind(order_id)
    .execute(&mut **txn)
    .await?;
    Ok(())
}
