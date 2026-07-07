use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelResult, Seedable};

mod attribute_values;
mod attributes;
mod options;
mod pictures;
mod variant_attribute_values;
mod variants;

pub use self::{
    attribute_values::AttributeValue,
    attributes::Attribute,
    options::ProductOption,
    pictures::Picture,
    variant_attribute_values::{NewVariantAttributeValue, VariantAttributeValue},
    variants::{NewVariant, ProductVariant},
};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    id: i32,
    pid: Uuid,
    category_id: i32,
    name: String,
    description: Option<String>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

impl Product {
    /// Loads products from a JSON file in `src/data` and seeds them.
    ///
    /// # Parameters
    ///
    /// - `db`: Database pool used for inserts and updates.
    /// - `file`: File name relative to `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialization, or database error if loading,
    /// decoding, inserting, or updating the products fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }

    /// Returns the internal database row ID.
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    /// Returns the public product ID.
    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    /// Returns the category row ID that owns this product.
    #[must_use]
    pub const fn category_id(&self) -> i32 {
        self.category_id
    }

    /// Returns the product name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the optional product description.
    #[must_use]
    pub const fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    /// Returns when the product was created.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    /// Returns when the product was last updated.
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }

    /// Returns when the product was soft-deleted, if it has been deleted.
    #[must_use]
    pub const fn deleted_at(&self) -> Option<DateTime<FixedOffset>> {
        self.deleted_at
    }
}

impl Seedable for Product {
    async fn seed(db: &sqlx::PgPool, data: &[Self]) -> super::ModelResult<()> {
        for product in data {
            sqlx::query(
                r"
                INSERT INTO products (
                    id,
                    pid,
                    category_id,
                    name,
                    description,
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
                    $8
                ) ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    category_id = EXCLUDED.category_id,
                    name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at,
                    deleted_at = EXCLUDED.deleted_at
           ",
            )
            .bind(product.id())
            .bind(product.pid())
            .bind(product.category_id())
            .bind(product.name())
            .bind(product.description())
            .bind(product.created_at())
            .bind(product.updated_at())
            .bind(product.deleted_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
