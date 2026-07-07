use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelError, ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct AttributeValue {
    id: i32,
    pid: Uuid,
    attribute_id: i32,
    value: String,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl AttributeValue {
    /// Creates an attribute value or returns the existing row for the same attribute.
    ///
    /// The supplied value is trimmed before lookup and insertion.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the lookup and insert.
    /// - `attribute_id`: Internal attribute row ID the value belongs to.
    /// - `value`: Attribute value to trim and persist.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup or insert fails.
    pub async fn create<'e, E>(db: &E, attribute_id: i32, value: &str) -> ModelResult<Self>
    where
        for<'a> &'a E: Executor<'e, Database = Postgres>,
    {
        let value = value.trim();

        if let Some(exists) = sqlx::query_as::<_, Self>(
            r"
                SELECT * FROM attribute_values WHERE attribute_id = $1 AND value = $2
        ",
        )
        .bind(attribute_id)
        .bind(value)
        .fetch_optional(db)
        .await?
        {
            return Ok(exists);
        }

        let attribute_value = sqlx::query_as::<_, Self>(
            r"
                INSERT INTO attribute_values (attribute_id, value)
                VALUES ($1, $2)
                RETURNING *
        ",
        )
        .bind(attribute_id)
        .bind(value)
        .fetch_one(db)
        .await?;

        Ok(attribute_value)
    }

    /// Finds an attribute value by public ID.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the lookup.
    /// - `pid`: Public ID of the attribute value to find.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no attribute value has the
    /// given public ID. Returns a database error if the lookup fails.
    pub async fn find_by_pid<'e, E>(db: E, pid: Uuid) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let attribute_value = sqlx::query_as::<_, Self>(
            r"
                SELECT * FROM attribute_values WHERE pid = $1
        ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?;

        attribute_value.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Finds an attribute value by value.
    ///
    /// The supplied value is trimmed before lookup.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the lookup.
    /// - `value`: Attribute value to trim and find.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no attribute value has the
    /// given value. Returns a database error if the lookup fails.
    pub async fn find_by_value<'e, E>(db: E, value: &str) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let attribute_value = sqlx::query_as::<_, Self>(
            r"
                SELECT * FROM attribute_values WHERE value = $1
        ",
        )
        .bind(value.trim())
        .fetch_optional(db)
        .await?;

        attribute_value.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Loads attribute values from a JSON file in `src/data` and seeds them.
    ///
    /// # Parameters
    ///
    /// - `db`: Database pool used for inserts and updates.
    /// - `file`: File name relative to `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialization, or database error if loading,
    /// decoding, inserting, or updating the attribute values fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }

    /// Returns the internal database row ID.
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    /// Returns the public attribute value ID.
    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    /// Returns the attribute row ID this value belongs to.
    #[must_use]
    pub const fn attribute_id(&self) -> i32 {
        self.attribute_id
    }

    /// Returns the stored attribute value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns when the attribute value was created.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    /// Returns when the attribute value was last updated.
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }
}

impl Seedable for AttributeValue {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for value in data {
            sqlx::query(
                r"
                INSERT INTO attribute_values (
                    id,
                    pid,
                    attribute_id,
                    value,
                    created_at,
                    updated_at
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6
                ) ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    attribute_id = EXCLUDED.attribute_id,
                    value = EXCLUDED.value,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at
                ",
            )
            .bind(value.id())
            .bind(value.pid())
            .bind(value.attribute_id())
            .bind(value.value())
            .bind(value.created_at())
            .bind(value.updated_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
