use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct ProductVariant {
    id: i32,
    pid: Uuid,
    product_id: i32,
    sku: String,
    price: f64,
    stock_quantity: i32,
    is_default: bool,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

impl ProductVariant {
    /// Loads product variants from a JSON file in `src/data` and seeds them.
    ///
    /// # Parameters
    ///
    /// - `db`: Database pool used for inserts and updates.
    /// - `file`: File name relative to `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialization, or database error if loading,
    /// decoding, inserting, or updating the variants fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }

    /// Returns the internal database row ID.
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    /// Returns the public product variant ID.
    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    /// Returns the product row ID this variant belongs to.
    #[must_use]
    pub const fn product_id(&self) -> i32 {
        self.product_id
    }

    /// Returns the stock keeping unit for this variant.
    #[must_use]
    pub fn sku(&self) -> &str {
        &self.sku
    }

    /// Returns the variant price.
    #[must_use]
    pub const fn price(&self) -> f64 {
        self.price
    }

    /// Returns the available stock quantity.
    #[must_use]
    pub const fn stock_quantity(&self) -> i32 {
        self.stock_quantity
    }

    /// Returns whether this is the default variant for its product.
    #[must_use]
    pub const fn is_default(&self) -> bool {
        self.is_default
    }

    /// Returns when the variant was created.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    /// Returns when the variant was last updated.
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }

    /// Returns when the variant was soft-deleted, if it has been deleted.
    #[must_use]
    pub const fn deleted_at(&self) -> Option<DateTime<FixedOffset>> {
        self.deleted_at
    }
}

impl Seedable for ProductVariant {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for variant in data {
            sqlx::query(
                r"
                INSERT INTO product_variants (
                    id,
                    pid,
                    product_id,
                    sku,
                    price,
                    stock_quantity,
                    is_default,
                    created_at,
                    updated_at,
                    deleted_at
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8,
                    $9,
                    $10
                ) ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    product_id = EXCLUDED.product_id,
                    sku = EXCLUDED.sku,
                    price = EXCLUDED.price,
                    stock_quantity = EXCLUDED.stock_quantity,
                    is_default = EXCLUDED.is_default,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at,
                    deleted_at = EXCLUDED.deleted_at
                ",
            )
            .bind(variant.id())
            .bind(variant.pid())
            .bind(variant.product_id())
            .bind(variant.sku())
            .bind(variant.price())
            .bind(variant.stock_quantity())
            .bind(variant.is_default())
            .bind(variant.created_at())
            .bind(variant.updated_at())
            .bind(variant.deleted_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
