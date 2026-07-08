use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelError, ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewProductOption {
    #[serde(rename = "productId")]
    product: i32,
    #[serde(rename = "attributeId")]
    attribute: i32,
    display_order: Option<i32>,
}

impl NewProductOption {
    /// Creates parameters for adding a product option.
    ///
    /// # Parameters
    ///
    /// - `product_id`: Internal product row ID this option belongs to.
    /// - `attribute_id`: Internal attribute row ID represented by the option.
    /// - `display_order`: Optional ordering value for rendering options.
    #[must_use]
    pub const fn new(product_id: i32, attribute_id: i32, display_order: Option<i32>) -> Self {
        Self {
            product: product_id,
            attribute: attribute_id,
            display_order,
        }
    }

    /// Returns the product row ID this option belongs to.
    #[must_use]
    pub const fn product_id(&self) -> i32 {
        self.product
    }

    /// Returns the attribute row ID represented by this option.
    #[must_use]
    pub const fn attribute_id(&self) -> i32 {
        self.attribute
    }

    /// Returns the optional display order for this option.
    #[must_use]
    pub const fn display_order(&self) -> Option<i32> {
        self.display_order
    }
}

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
    /// Creates a product option.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the insert.
    /// - `params`: Product option data to persist.
    ///
    /// # Errors
    ///
    /// Returns [`crate::models::ModelError::EntityAlreadyExists`] when the
    /// product already has an option for the attribute, and
    /// [`crate::models::ModelError::InvalidReference`] when the product or
    /// attribute does not exist. Returns a database error if the insert fails
    /// for another reason.
    pub async fn create<'e, E>(db: E, params: &NewProductOption) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let option = sqlx::query_as::<_, Self>(
            r"
                INSERT INTO product_options (
                    product_id,
                    attribute_id,
                    display_order
                ) VALUES (
                    $1,
                    $2,
                    $3
                )
                RETURNING *
        ",
        )
        .bind(params.product_id())
        .bind(params.attribute_id())
        .bind(params.display_order())
        .fetch_one(db)
        .await?;

        Ok(option)
    }

    /// Finds a product option by public ID.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the lookup.
    /// - `pid`: Public ID of the product option to find.
    ///
    /// # Errors
    ///
    /// Returns [`crate::models::ModelError::EntityNotFound`] when no product
    /// option has the given public ID. Returns a database error if the lookup
    /// fails.
    pub async fn find_by_pid<'e, E>(db: E, pid: Uuid) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let option = sqlx::query_as::<_, Self>(
            r"
                SELECT * FROM product_options WHERE pid = $1
            ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?;

        option.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Lists all options for a product.
    ///
    /// Options are ordered by display order and then ID.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the lookup.
    /// - `product_id`: Internal product row ID whose options should be
    ///   returned.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup fails.
    pub async fn find_by_product<'e, E>(db: E, product_id: i32) -> ModelResult<Vec<Self>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let options = sqlx::query_as::<_, Self>(
            r"
                SELECT * FROM product_options
                WHERE product_id = $1
                ORDER BY display_order NULLS LAST, id
            ",
        )
        .bind(product_id)
        .fetch_all(db)
        .await?;

        Ok(options)
    }

    /// Lists all product options.
    ///
    /// Options are ordered by product, display order, and ID.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the lookup.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup fails.
    pub async fn find_all<'e, E>(db: E) -> ModelResult<Vec<Self>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let options = sqlx::query_as::<_, Self>(
            r"
                SELECT * FROM product_options
                ORDER BY product_id, display_order NULLS LAST, id
            ",
        )
        .fetch_all(db)
        .await?;

        Ok(options)
    }

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
