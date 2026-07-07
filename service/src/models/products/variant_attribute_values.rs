use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewVariantAttributeValue {
    #[serde(rename = "variantId")]
    variant: i32,
    #[serde(rename = "attributeId")]
    attribute: i32,
    #[serde(rename = "attributeValueId")]
    value: i32,
}

impl NewVariantAttributeValue {
    /// Creates parameters for linking a variant to an attribute value.
    ///
    /// # Parameters
    ///
    /// - `variant_id`: Internal product variant row ID.
    /// - `attribute_id`: Internal attribute row ID represented by the value.
    /// - `attribute_value_id`: Internal attribute value row ID selected for
    ///   the variant.
    #[must_use]
    pub const fn new(variant_id: i32, attribute_id: i32, attribute_value_id: i32) -> Self {
        Self {
            variant: variant_id,
            attribute: attribute_id,
            value: attribute_value_id,
        }
    }

    /// Returns the product variant row ID.
    #[must_use]
    pub const fn variant_id(&self) -> i32 {
        self.variant
    }

    /// Returns the attribute row ID.
    #[must_use]
    pub const fn attribute_id(&self) -> i32 {
        self.attribute
    }

    /// Returns the selected attribute value row ID.
    #[must_use]
    pub const fn attribute_value_id(&self) -> i32 {
        self.value
    }
}

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
    /// Creates a variant attribute value link.
    ///
    /// A variant can have only one value for a given attribute.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the insert.
    /// - `params`: Variant, attribute, and attribute value row IDs to link.
    ///
    /// # Errors
    ///
    /// Returns [`crate::models::ModelError::EntityAlreadyExists`] when the
    /// variant already has a value for the attribute. Returns
    /// [`crate::models::ModelError::InvalidReference`] when the variant,
    /// attribute, or attribute value does not exist. Returns a database error
    /// if the insert fails for another reason.
    pub async fn create<'e, E>(db: &E, params: &NewVariantAttributeValue) -> ModelResult<Self>
    where
        for<'a> &'a E: Executor<'e, Database = Postgres>,
    {
        let value = sqlx::query_as::<_, Self>(
            r"
                INSERT INTO variant_attribute_values (
                    variant_id,
                    attribute_id,
                    attribute_value_id
                ) VALUES (
                    $1,
                    $2,
                    $3
                )
                RETURNING *
        ",
        )
        .bind(params.variant_id())
        .bind(params.attribute_id())
        .bind(params.attribute_value_id())
        .fetch_one(db)
        .await?;

        Ok(value)
    }

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
