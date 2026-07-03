use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct ProductOption {
    id: i32,
    pid: Uuid,
    product_id: i32,
    attribute_id: i32,
    display_order: Option<i32>,
    created_at: DateTime<FixedOffset>,
}

impl ProductOption {
    /// Seeds product options from a file in `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialisation, or database error if loading or
    /// inserting the loaded product options fails.
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
    pub const fn product_id(&self) -> i32 {
        self.product_id
    }

    #[must_use]
    pub const fn attribute_id(&self) -> i32 {
        self.attribute_id
    }

    #[must_use]
    pub const fn display_order(&self) -> Option<i32> {
        self.display_order
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }
}

impl Seedable for ProductOption {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for option in data {
            sqlx::query(
                r"
                INSERT INTO product_options (
                    id,
                    pid,
                    product_id,
                    attribute_id,
                    display_order,
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
                    product_id = EXCLUDED.product_id,
                    attribute_id = EXCLUDED.attribute_id,
                    display_order = EXCLUDED.display_order,
                    created_at = EXCLUDED.created_at
                ",
            )
            .bind(option.id())
            .bind(option.pid())
            .bind(option.product_id())
            .bind(option.attribute_id())
            .bind(option.display_order())
            .bind(option.created_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
