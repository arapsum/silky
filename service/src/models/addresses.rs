use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::schemas::NewAddress;

use super::{ModelError, ModelResult};

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
#[expect(
    dead_code,
    reason = "internal database keys are retained for repository operations"
)]
#[allow(clippy::struct_field_names)]
pub struct Address {
    #[serde(skip)]
    id: i32,
    pid: Uuid,
    #[serde(skip)]
    customer_id: i32,
    address_type: String,
    label: Option<String>,
    recipient_name: String,
    company: Option<String>,
    line_one: String,
    line_two: Option<String>,
    city: String,
    region: Option<String>,
    postal_code: Option<String>,
    country_code: String,
    email: Option<String>,
    phone: Option<String>,
    is_default: bool,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

impl Address {
    /// Creates a customer-owned address from a public customer ID.
    ///
    /// When `is_default` is true, the existing default for the same customer
    /// and address type is cleared in the same transaction.
    ///
    /// # Errors
    /// Returns [`ModelError::InvalidReference`] when the customer is absent,
    /// or [`ModelError::Sqlx`] when the address cannot be persisted.
    pub async fn create(db: &PgPool, params: &NewAddress) -> ModelResult<Self> {
        let mut txn = db.begin().await?;
        let customer_id = sqlx::query_scalar::<_, i32>(
            r"SELECT id
              FROM users
              WHERE pid = $1
                AND deleted_at IS NULL",
        )
        .bind(params.customer_pid())
        .fetch_optional(&mut *txn)
        .await?
        .ok_or_else(|| ModelError::InvalidReference("Customer does not exist.".to_string()))?;

        if params.is_default() {
            sqlx::query(
                r"UPDATE addresses
                  SET is_default = FALSE
                  WHERE customer_id = $1
                    AND address_type = $2
                    AND deleted_at IS NULL",
            )
            .bind(customer_id)
            .bind(params.address_type())
            .execute(&mut *txn)
            .await?;
        }

        let address = sqlx::query_as::<_, Self>(
            r"INSERT INTO addresses (
                customer_id,
                address_type,
                label,
                recipient_name,
                company,
                line_one,
                line_two,
                city,
                region,
                postal_code,
                country_code,
                email,
                phone,
                is_default
              )
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
              RETURNING *",
        )
        .bind(customer_id)
        .bind(params.address_type())
        .bind(params.label().map(str::trim))
        .bind(params.recipient_name().trim())
        .bind(params.company().map(str::trim))
        .bind(params.line_one().trim())
        .bind(params.line_two().map(str::trim))
        .bind(params.city().trim())
        .bind(params.region().map(str::trim))
        .bind(params.postal_code().map(str::trim))
        .bind(params.country_code().to_uppercase())
        .bind(params.email().map(str::trim))
        .bind(params.phone().map(str::trim))
        .bind(params.is_default())
        .fetch_one(&mut *txn)
        .await?;
        txn.commit().await?;
        Ok(address)
    }

    /// Finds one non-deleted address by its public ID.
    ///
    /// # Errors
    /// Returns [`ModelError::EntityNotFound`] when the address is missing or deleted.
    pub async fn find_by_pid(db: &PgPool, pid: Uuid) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"SELECT *
              FROM addresses
              WHERE pid = $1
                AND deleted_at IS NULL",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?
        .ok_or(ModelError::EntityNotFound)
    }

    /// Lists a customer's non-deleted addresses, with defaults first.
    ///
    /// # Errors
    /// Returns a database error when the customer address list cannot be queried.
    pub async fn find_by_customer(db: &PgPool, customer_pid: Uuid) -> ModelResult<Vec<Self>> {
        Ok(sqlx::query_as::<_, Self>(
            r"SELECT a.*
              FROM addresses a
              JOIN users u ON u.id = a.customer_id
              WHERE u.pid = $1
                AND a.deleted_at IS NULL
              ORDER BY a.is_default DESC, a.created_at DESC, a.id DESC",
        )
        .bind(customer_pid)
        .fetch_all(db)
        .await?)
    }

    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }
    #[must_use]
    pub fn address_type(&self) -> &str {
        &self.address_type
    }
    #[must_use]
    pub fn city(&self) -> &str {
        &self.city
    }
}
