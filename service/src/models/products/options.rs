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
    /// Loads product options from a JSON file in `src/data` and seeds them.
    ///
    /// # Parameters
    ///
    /// - `db`: Database pool used for inserts and updates.
    /// - `file`: File name relative to `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialization, or database error if loading,
    /// decoding, inserting, or updating the product options fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }

    /// Returns the internal database row ID.
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    /// Returns the public product option ID.
    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    /// Returns the product row ID this option belongs to.
    #[must_use]
    pub const fn product_id(&self) -> i32 {
        self.product_id
    }

    /// Returns the attribute row ID represented by this option.
    #[must_use]
    pub const fn attribute_id(&self) -> i32 {
        self.attribute_id
    }

    /// Returns the optional display order for this option.
    #[must_use]
    pub const fn display_order(&self) -> Option<i32> {
        self.display_order
    }

    /// Returns when the product option was created.
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
