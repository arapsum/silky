use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use super::{ModelError, ModelResult};
use crate::views::CheckoutSessionResponse;

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

#[derive(Debug, Clone)]
pub struct PaymentReconciliation {
    pub attempt_pid: Uuid,
    pub order_pid: Uuid,
    pub amount: Decimal,
    pub currency: String,
}

impl PaymentAttempt {
    pub(super) async fn find_active_for_order(
        txn: &mut Transaction<'_, Postgres>,
        order_id: i32,
    ) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"SELECT *
              FROM payment_attempts
              WHERE order_id = $1
                AND status IN ('initiated', 'session_created', 'processing')
              ORDER BY created_at DESC, id DESC
              LIMIT 1
              FOR UPDATE",
        )
        .bind(order_id)
        .fetch_optional(&mut **txn)
        .await?
        .ok_or(ModelError::PaymentAttemptNotFound)
    }

    /// Finds the active payment attempt for a customer-owned order.
    ///
    /// # Errors
    /// Returns an attempt-not-found error when ownership or active state does
    /// not match, or a database error when the query fails.
    pub async fn find_active_for_customer_order(
        db: &sqlx::PgPool,
        order_pid: Uuid,
        customer_pid: Uuid,
    ) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"SELECT attempt.*
              FROM payment_attempts AS attempt
              JOIN orders AS orders ON orders.id = attempt.order_id
              JOIN users AS customer ON customer.id = orders.customer_id
              WHERE orders.pid = $1
                AND customer.pid = $2
                AND attempt.status IN ('initiated', 'session_created', 'processing')
              ORDER BY attempt.created_at DESC, attempt.id DESC
              LIMIT 1",
        )
        .bind(order_pid)
        .bind(customer_pid)
        .fetch_optional(db)
        .await?
        .ok_or(ModelError::PaymentAttemptNotFound)
    }

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

    /// Fails an unpaid attempt, cancels its order, and restores stock once.
    ///
    /// Replayed failures are idempotent. A successful payment is acknowledged
    /// without changing the attempt, order, or inventory when a stale failure
    /// event arrives after payment confirmation.
    ///
    /// # Errors
    /// Returns an attempt-not-found or invalid-transition error, or a database
    /// error when the attempt cannot be updated.
    pub async fn fail_and_release_inventory(
        txn: &mut Transaction<'_, Postgres>,
        session_id: &str,
        code: Option<&str>,
        message: Option<&str>,
    ) -> ModelResult<Self> {
        let attempt = Self::lock_by_session(txn, session_id).await?;
        if matches!(attempt.status.as_str(), "failed" | "succeeded") {
            return Ok(attempt);
        }
        attempt.require_status(&["session_created", "processing"], "failed")?;

        restore_inventory(txn, attempt.order_id).await?;
        cancel_order(txn, attempt.order_id).await?;

        let updated = sqlx::query_as::<_, Self>(
            r"UPDATE payment_attempts
              SET status = 'failed',
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

    /// Finds the latest hosted Checkout Session for a customer-owned order.
    ///
    /// The query includes customer ownership so another customer's order and
    /// an order without an attempt are indistinguishable. Expired or terminal
    /// attempts retain their status but no longer expose a redirect URL.
    ///
    /// # Errors
    /// Returns [`ModelError::PaymentAttemptNotFound`] when the order is not
    /// owned by the customer or has no payment attempt. Returns a database
    /// error when the lookup fails.
    pub async fn find_checkout_session_for_customer(
        db: &sqlx::PgPool,
        order_pid: Uuid,
        customer_pid: Uuid,
    ) -> ModelResult<CheckoutSessionResponse> {
        sqlx::query_as::<_, CheckoutSessionResponse>(
            r"
            SELECT orders.pid AS order_pid,
                orders.status AS order_status,
                orders.payment_status,
                attempt.status AS attempt_status,
                CASE
                    WHEN attempt.status IN ('initiated', 'session_created', 'processing')
                        AND (attempt.expires_at IS NULL OR attempt.expires_at > NOW())
                    THEN attempt.checkout_url
                    ELSE NULL
                END AS checkout_url,
                attempt.expires_at
            FROM orders
            JOIN users AS customer ON customer.id = orders.customer_id
            JOIN payment_attempts AS attempt ON attempt.order_id = orders.id
            WHERE orders.pid = $1
                AND customer.pid = $2
            ORDER BY attempt.created_at DESC, attempt.id DESC
            LIMIT 1
            ",
        )
        .bind(order_pid)
        .bind(customer_pid)
        .fetch_optional(db)
        .await?
        .ok_or(ModelError::PaymentAttemptNotFound)
    }

    /// Recovers the Session link when Stripe succeeded after the application
    /// committed its attempt but before it stored Stripe's response.
    ///
    /// Replaying the same link is idempotent. A different Session or a terminal
    /// attempt is rejected before any order state changes.
    ///
    /// # Errors
    /// Returns an attempt-not-found or invalid-transition error, or a database
    /// error when the attempt cannot be locked or updated.
    pub async fn recover_checkout_session(
        txn: &mut Transaction<'_, Postgres>,
        attempt_pid: Uuid,
        session_id: &str,
        checkout_url: Option<&str>,
        expires_at: DateTime<FixedOffset>,
    ) -> ModelResult<Self> {
        let attempt = Self::lock_by_pid(txn, attempt_pid).await?;
        if attempt.stripe_checkout_session_id.as_deref() == Some(session_id) {
            return Ok(attempt);
        }
        attempt.require_status(&["initiated"], "session_created")?;

        Ok(sqlx::query_as::<_, Self>(
            r"UPDATE payment_attempts
              SET status = 'session_created',
                  stripe_checkout_session_id = $2,
                  checkout_url = COALESCE($3, checkout_url),
                  expires_at = $4
              WHERE pid = $1
              RETURNING *",
        )
        .bind(attempt_pid)
        .bind(session_id)
        .bind(checkout_url)
        .bind(expires_at)
        .fetch_one(&mut **txn)
        .await?)
    }

    /// Locks and returns the identifiers and monetary values needed to
    /// reconcile a verified Stripe Session against Silk's persisted records.
    ///
    /// # Errors
    /// Returns an attempt-not-found error when the Session is unknown, or a
    /// database error when the linked rows cannot be locked.
    pub async fn reconciliation_for_session(
        txn: &mut Transaction<'_, Postgres>,
        session_id: &str,
    ) -> ModelResult<PaymentReconciliation> {
        sqlx::query_as::<_, (Uuid, Uuid, Decimal, String)>(
            r"SELECT attempt.pid, orders.pid, attempt.amount, attempt.currency
              FROM payment_attempts attempt
              JOIN orders ON orders.id = attempt.order_id
              WHERE attempt.stripe_checkout_session_id = $1
              FOR UPDATE OF attempt, orders",
        )
        .bind(session_id)
        .fetch_optional(&mut **txn)
        .await?
        .map(
            |(attempt_pid, order_pid, amount, currency)| PaymentReconciliation {
                attempt_pid,
                order_pid,
                amount,
                currency,
            },
        )
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

    #[must_use]
    pub fn checkout_url(&self) -> Option<&str> {
        self.checkout_url.as_deref()
    }

    #[must_use]
    pub fn checkout_session_id(&self) -> Option<&str> {
        self.stripe_checkout_session_id.as_deref()
    }

    #[must_use]
    pub const fn expires_at(&self) -> Option<DateTime<FixedOffset>> {
        self.expires_at
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
