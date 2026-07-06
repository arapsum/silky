use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct VariantAttributeValue {
    id: i32,
    pid: Uuid,
    variant_id: i32,
    attribute_id: i32,
    attribute_value_id: i32,
    created_at: DateTime<FixedOffset>,
}

impl VariantAttributeValue {
    /// Loads variant attribute value links from a JSON file in `src/data` and seeds them.
    ///
    /// # Parameters
    ///
    /// - `db`: Database pool used for inserts and updates.
    /// - `file`: File name relative to `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialization, or database error if loading,
    /// decoding, inserting, or updating the variant attribute value links
    /// fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }

    /// Returns the internal database row ID.
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    /// Returns the public variant attribute value link ID.
    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    /// Returns the product variant row ID this link belongs to.
    #[must_use]
    pub const fn variant_id(&self) -> i32 {
        self.variant_id
    }

    /// Returns the attribute row ID this link represents.
    #[must_use]
    pub const fn attribute_id(&self) -> i32 {
        self.attribute_id
    }

    /// Returns the attribute value row ID selected for the variant.
    #[must_use]
    pub const fn attribute_value_id(&self) -> i32 {
        self.attribute_value_id
    }

    /// Returns when the link was created.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }
}

impl Seedable for VariantAttributeValue {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for value in data {
            sqlx::query(
                r"
                INSERT INTO variant_attribute_values (
                    id,
                    pid,
                    variant_id,
                    attribute_id,
                    attribute_value_id,
                    created_at
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6
                ) ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    variant_id = EXCLUDED.variant_id,
                    attribute_id = EXCLUDED.attribute_id,
                    attribute_value_id = EXCLUDED.attribute_value_id,
                    created_at = EXCLUDED.created_at
                ",
            )
            .bind(value.id())
            .bind(value.pid())
            .bind(value.variant_id())
            .bind(value.attribute_id())
            .bind(value.attribute_value_id())
            .bind(value.created_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
