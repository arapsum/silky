use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::{
    models::{ModelError, ModelResult, Seedable},
    schemas::UpdateProductPicture,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewPicture {
    product: i32,
    variant: Option<i32>,
    image_link: String,
    display_order: Option<i32>,
}

impl NewPicture {
    /// Creates parameters for adding a product picture.
    ///
    /// # Parameters
    ///
    /// - `product`: Internal product row ID this picture belongs to.
    /// - `image_link`: Public image URL to store.
    /// - `variant`: Optional internal variant row ID for variant-specific
    ///   pictures.
    /// - `display_order`: Optional ordering value for rendering pictures.
    #[must_use]
    pub const fn new(
        product: i32,
        image_link: String,
        variant: Option<i32>,
        display_order: Option<i32>,
    ) -> Self {
        Self {
            product,
            variant,
            image_link,
            display_order,
        }
    }

    #[must_use]
    pub const fn product(&self) -> i32 {
        self.product
    }

    #[must_use]
    pub const fn variant(&self) -> Option<i32> {
        self.variant
    }

    #[must_use]
    pub fn image_link(&self) -> &str {
        &self.image_link
    }

    #[must_use]
    pub const fn display_order(&self) -> Option<i32> {
        self.display_order
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct Picture {
    id: i32,
    pid: Uuid,
    product_id: i32,
    variant_id: Option<i32>,
    image_link: String,
    display_order: Option<i32>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl Picture {
    /// Creates a product picture.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the insert.
    /// - `params`: Product picture data to persist.
    ///
    /// # Errors
    ///
    /// Returns [`crate::models::ModelError::InvalidReference`] when the
    /// product or variant does not exist. Returns a database error if the
    /// insert fails for another reason.
    pub async fn create<'e, E>(db: E, params: &NewPicture) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let picture = sqlx::query_as::<_, Self>(
            r"
                INSERT INTO pictures (
                    product_id,
                    variant_id,
                    image_link,
                    display_order
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4
                )
                RETURNING *
        ",
        )
        .bind(params.product())
        .bind(params.variant())
        .bind(params.image_link())
        .bind(params.display_order())
        .fetch_one(db)
        .await?;

        Ok(picture)
    }

    /// Updates a product or variant picture display order.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when the picture does not belong
    /// to the active product and optional active variant. Returns a database
    /// error if the update fails.
    pub async fn update_order(
        db: &PgPool,
        product_pid: Uuid,
        variant_pid: Option<Uuid>,
        picture_pid: Uuid,
        params: &UpdateProductPicture,
    ) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"
            UPDATE pictures pic
            SET display_order = $4
            WHERE EXISTS (
                    SELECT 1
                    FROM products p
                    WHERE p.id = pic.product_id
                        AND p.pid = $1
                        AND p.deleted_at IS NULL
                )
                AND pic.pid = $3
                AND (
                    ($2::UUID IS NULL AND pic.variant_id IS NULL)
                    OR EXISTS (
                        SELECT 1
                        FROM product_variants pv
                        WHERE pv.id = pic.variant_id
                            AND $2::UUID IS NOT NULL
                            AND pv.pid = $2
                            AND pv.product_id = pic.product_id
                            AND pv.deleted_at IS NULL
                    )
                )
            RETURNING pic.*
        ",
        )
        .bind(product_pid)
        .bind(variant_pid)
        .bind(picture_pid)
        .bind(params.display_order())
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Deletes a product or variant picture.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when the picture does not belong
    /// to the active product and optional active variant. Returns a database
    /// error if the delete fails.
    pub async fn delete(
        db: &PgPool,
        product_pid: Uuid,
        variant_pid: Option<Uuid>,
        picture_pid: Uuid,
    ) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"
            DELETE FROM pictures pic
            WHERE EXISTS (
                    SELECT 1
                    FROM products p
                    WHERE p.id = pic.product_id
                        AND p.pid = $1
                        AND p.deleted_at IS NULL
                )
                AND pic.pid = $3
                AND (
                    ($2::UUID IS NULL AND pic.variant_id IS NULL)
                    OR EXISTS (
                        SELECT 1
                        FROM product_variants pv
                        WHERE pv.id = pic.variant_id
                            AND $2::UUID IS NOT NULL
                            AND pv.pid = $2
                            AND pv.product_id = pic.product_id
                            AND pv.deleted_at IS NULL
                    )
                )
            RETURNING pic.*
        ",
        )
        .bind(product_pid)
        .bind(variant_pid)
        .bind(picture_pid)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Loads product pictures from a JSON file in `src/data` and seeds them.
    ///
    /// # Parameters
    ///
    /// - `db`: Database pool used for inserts and updates.
    /// - `file`: File name relative to `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialization, or database error if loading,
    /// decoding, inserting, or updating the pictures fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }

    /// Returns the internal database row ID.
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    /// Returns the public picture ID.
    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    /// Returns the product row ID this picture belongs to.
    #[must_use]
    pub const fn product_id(&self) -> i32 {
        self.product_id
    }

    /// Returns the variant row ID this picture belongs to, if variant-specific.
    #[must_use]
    pub const fn variant_id(&self) -> Option<i32> {
        self.variant_id
    }

    /// Returns the picture URL.
    #[must_use]
    pub fn image_link(&self) -> &str {
        &self.image_link
    }

    /// Returns the optional display order for this picture.
    #[must_use]
    pub const fn display_order(&self) -> Option<i32> {
        self.display_order
    }

    /// Returns when the picture was created.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    /// Returns when the picture was last updated.
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }
}

impl Seedable for Picture {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for picture in data {
            sqlx::query(
                r"
                INSERT INTO pictures (
                    id,
                    pid,
                    product_id,
                    variant_id,
                    image_link,
                    display_order,
                    created_at,
                    updated_at
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
                    product_id = EXCLUDED.product_id,
                    variant_id = EXCLUDED.variant_id,
                    image_link = EXCLUDED.image_link,
                    display_order = EXCLUDED.display_order,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at
                ",
            )
            .bind(picture.id())
            .bind(picture.pid())
            .bind(picture.product_id())
            .bind(picture.variant_id())
            .bind(picture.image_link())
            .bind(picture.display_order())
            .bind(picture.created_at())
            .bind(picture.updated_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
