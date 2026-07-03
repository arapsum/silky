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
    /// Seeds variant attribute values from a file in `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialisation, or database error if loading or
    /// inserting the loaded variant attribute values fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }

    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    #[must_use]
    pub const fn variant_id(&self) -> i32 {
        self.variant_id
    }

    #[must_use]
    pub const fn attribute_id(&self) -> i32 {
        self.attribute_id
    }

    #[must_use]
    pub const fn attribute_value_id(&self) -> i32 {
        self.attribute_value_id
    }

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
