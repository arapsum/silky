use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::schemas::NewAddress;

use super::{ModelError, ModelResult, Seedable};

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
pub struct Address {
    #[serde(skip_serializing)]
    id: i32,
    pid: Uuid,
    #[serde(skip_serializing)]
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
    pub async fn create(db: &PgPool, customer_pid: Uuid, params: &NewAddress) -> ModelResult<Self> {
        let mut txn = db.begin().await?;
        let owner_id = Self::customer_row_id(&mut txn, customer_pid).await?;

        if params.is_default() {
            sqlx::query(
                r"UPDATE addresses
                  SET is_default = FALSE
                  WHERE customer_id = $1
                    AND address_type = $2
                    AND deleted_at IS NULL",
            )
            .bind(owner_id)
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
        .bind(owner_id)
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

    /// Replaces an address owned by the supplied customer.
    ///
    /// When the replacement becomes the default, the previous default for the
    /// same address type is cleared within the transaction. Ownership is part
    /// of the update predicate so callers cannot edit another customer's data.
    ///
    /// # Errors
    /// Returns [`ModelError::EntityNotFound`] when the address is missing,
    /// deleted, or belongs to another customer. Returns a database error when
    /// the update cannot be persisted.
    pub async fn update_for_customer(
        db: &PgPool,
        pid: Uuid,
        customer_pid: Uuid,
        params: &NewAddress,
    ) -> ModelResult<Self> {
        let mut txn = db.begin().await?;
        let owner_id = Self::customer_row_id(&mut txn, customer_pid).await?;

        if params.is_default() {
            sqlx::query(
                r"
                UPDATE addresses
                SET is_default = FALSE
                WHERE customer_id = $1
                    AND address_type = $2
                    AND pid <> $3
                    AND deleted_at IS NULL
                ",
            )
            .bind(owner_id)
            .bind(params.address_type())
            .bind(pid)
            .execute(&mut *txn)
            .await?;
        }

        let address = sqlx::query_as::<_, Self>(
            r"
            UPDATE addresses
            SET address_type = $1,
                label = $2,
                recipient_name = $3,
                company = $4,
                line_one = $5,
                line_two = $6,
                city = $7,
                region = $8,
                postal_code = $9,
                country_code = $10,
                email = $11,
                phone = $12,
                is_default = $13
            WHERE pid = $14
                AND customer_id = $15
                AND deleted_at IS NULL
            RETURNING *
            ",
        )
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
        .bind(pid)
        .bind(owner_id)
        .fetch_optional(&mut *txn)
        .await?
        .ok_or(ModelError::EntityNotFound)?;

        txn.commit().await?;
        Ok(address)
    }

    /// Soft-deletes an address owned by the supplied customer.
    ///
    /// Historical orders retain their immutable address snapshots and their
    /// optional foreign-key reference to this row.
    ///
    /// # Errors
    /// Returns [`ModelError::EntityNotFound`] when the address is missing,
    /// already deleted, or belongs to another customer. Returns a database
    /// error when the deletion cannot be persisted.
    pub async fn delete_for_customer(
        db: &PgPool,
        pid: Uuid,
        customer_pid: Uuid,
    ) -> ModelResult<()> {
        let result = sqlx::query(
            r"
            UPDATE addresses AS address
            SET deleted_at = NOW(),
                is_default = FALSE
            FROM users AS customer
            WHERE address.pid = $1
                AND customer.pid = $2
                AND customer.id = address.customer_id
                AND address.deleted_at IS NULL
            ",
        )
        .bind(pid)
        .bind(customer_pid)
        .execute(db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ModelError::EntityNotFound);
        }

        Ok(())
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

    /// Finds one non-deleted address owned by the supplied customer.
    ///
    /// # Errors
    /// Returns [`ModelError::EntityNotFound`] when the address is missing,
    /// deleted, or belongs to another customer.
    pub async fn find_by_pid_for_customer(
        db: &PgPool,
        pid: Uuid,
        customer_pid: Uuid,
    ) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"
            SELECT address.*
            FROM addresses AS address
            JOIN users AS customer ON customer.id = address.customer_id
            WHERE address.pid = $1
                AND customer.pid = $2
                AND address.deleted_at IS NULL
            ",
        )
        .bind(pid)
        .bind(customer_pid)
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

    async fn customer_row_id(
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        customer_pid: Uuid,
    ) -> ModelResult<i32> {
        sqlx::query_scalar::<_, i32>(
            r"
            SELECT id
            FROM users
            WHERE pid = $1
                AND deleted_at IS NULL
            ",
        )
        .bind(customer_pid)
        .fetch_optional(&mut **txn)
        .await?
        .ok_or_else(|| ModelError::InvalidReference("Customer does not exist.".to_string()))
    }

    /// Loads addresses from a data file and seeds them into the database.
    ///
    /// The file path is resolved relative to `src/data` and must contain a
    /// JSON or YAML array matching [`Address`]. Existing rows are updated by
    /// their internal ID so the operation can be repeated safely.
    ///
    /// # Errors
    /// Returns a file, deserialization, or database error when the seed cannot
    /// be loaded or persisted.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;
        Self::seed(db, &data).await
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

impl Seedable for Address {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for address in data {
            sqlx::query(
                r"
                INSERT INTO addresses (
                    id,
                    pid,
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
                    is_default,
                    created_at,
                    updated_at,
                    deleted_at
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19
                )
                ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    customer_id = EXCLUDED.customer_id,
                    address_type = EXCLUDED.address_type,
                    label = EXCLUDED.label,
                    recipient_name = EXCLUDED.recipient_name,
                    company = EXCLUDED.company,
                    line_one = EXCLUDED.line_one,
                    line_two = EXCLUDED.line_two,
                    city = EXCLUDED.city,
                    region = EXCLUDED.region,
                    postal_code = EXCLUDED.postal_code,
                    country_code = EXCLUDED.country_code,
                    email = EXCLUDED.email,
                    phone = EXCLUDED.phone,
                    is_default = EXCLUDED.is_default,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at,
                    deleted_at = EXCLUDED.deleted_at
                ",
            )
            .bind(address.id)
            .bind(address.pid)
            .bind(address.customer_id)
            .bind(&address.address_type)
            .bind(address.label.as_deref())
            .bind(&address.recipient_name)
            .bind(address.company.as_deref())
            .bind(&address.line_one)
            .bind(address.line_two.as_deref())
            .bind(&address.city)
            .bind(address.region.as_deref())
            .bind(address.postal_code.as_deref())
            .bind(&address.country_code)
            .bind(address.email.as_deref())
            .bind(address.phone.as_deref())
            .bind(address.is_default)
            .bind(address.created_at)
            .bind(address.updated_at)
            .bind(address.deleted_at)
            .execute(db)
            .await?;
        }

        sqlx::query(
            r"SELECT setval(
                    pg_get_serial_sequence('addresses', 'id'),
                    COALESCE((SELECT MAX(id) FROM addresses), 1),
                    (SELECT COUNT(*) > 0 FROM addresses)
                )",
        )
        .execute(db)
        .await?;

        Ok(())
    }
}
