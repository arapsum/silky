use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelResult, Seedable};

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
    /// Seeds pictures from a file in `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialisation, or database error if loading or
    /// inserting the loaded pictures fails.
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
    pub const fn variant_id(&self) -> Option<i32> {
        self.variant_id
    }

    #[must_use]
    pub fn image_link(&self) -> &str {
        &self.image_link
    }

    #[must_use]
    pub const fn display_order(&self) -> Option<i32> {
        self.display_order
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

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
