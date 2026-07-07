use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewVariant {
    #[serde(rename = "productId")]
    product: i32,
    sku: String,
    price: Decimal,
    stock_quantity: i32,
    is_default: bool,
}

impl NewVariant {
    /// Creates parameters for adding a product variant.
    ///
    /// # Parameters
    ///
    /// - `product`: Internal product row ID this variant belongs to.
    /// - `sku`: Stock keeping unit for the variant.
    /// - `price`: Variant price.
    /// - `stock_quantity`: Available stock quantity.
    /// - `is_default`: Whether this is the default variant for the product.
    #[must_use]
    pub const fn new(
        product: i32,
        sku: String,
        price: Decimal,
        stock_quantity: i32,
        is_default: bool,
    ) -> Self {
        Self {
            product,
            sku,
            price,
            stock_quantity,
            is_default,
        }
    }

    /// Returns the product row ID this variant belongs to.
    #[must_use]
    pub const fn product_id(&self) -> i32 {
        self.product
    }

    /// Returns the stock keeping unit.
    #[must_use]
    pub fn sku(&self) -> &str {
        &self.sku
    }

    /// Returns the variant price.
    #[must_use]
    pub const fn price(&self) -> Decimal {
        self.price
    }

    /// Returns the available stock quantity.
    #[must_use]
    pub const fn stock_quantity(&self) -> i32 {
        self.stock_quantity
    }

    /// Returns whether this is the default variant for the product.
    #[must_use]
    pub const fn is_default(&self) -> bool {
        self.is_default
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct ProductVariant {
    id: i32,
    pid: Uuid,
    product_id: i32,
    sku: String,
    price: Decimal,
    stock_quantity: i32,
    is_default: bool,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

impl ProductVariant {
    /// Creates a product variant.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the insert.
    /// - `params`: Product variant data to persist.
    ///
    /// # Errors
    ///
    /// Returns [`crate::models::ModelError::EntityAlreadyExists`] for unique
    /// constraint violations, [`crate::models::ModelError::InvalidReference`]
    /// for missing referenced records, and
    /// [`crate::models::ModelError::InvalidInput`] for check constraint
    /// violations. Returns a database error if the insert fails for another
    /// reason.
    pub async fn create<'e, E>(db: E, params: &NewVariant) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let variant = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO product_variants (
                    product_id,
                    sku,
                    price,
                    stock_quantity,
                    is_default
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5
                ) RETURNING *
        ",
        )
        .bind(params.product_id())
        .bind(params.sku())
        .bind(params.price())
        .bind(params.stock_quantity())
        .bind(params.is_default())
        .fetch_one(db)
        .await?;

        Ok(variant)
    }

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
    pub const fn price(&self) -> Decimal {
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
