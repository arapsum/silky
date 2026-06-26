#![allow(unused_imports)]
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelError, ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    id: i32,
    pid: Uuid,
    name: String,
    image_link: String,
    description: Option<String>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

impl Category {
    /// Seeds categories from a file in `src/data`.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::FileNotFound`] when the file is missing,
    /// [`ModelError::UnsupportedFileType`] when the extension is not supported,
    /// deserialization errors for invalid seed data, or database errors when
    /// inserting the loaded permissions fails.
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
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn image_link(&self) -> &str {
        &self.image_link
    }

    #[must_use]
    pub const fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }

    #[must_use]
    pub const fn deleted_at(&self) -> Option<DateTime<FixedOffset>> {
        self.deleted_at
    }
}

impl Seedable for Category {
    async fn seed(db: &sqlx::PgPool, data: &[Self]) -> super::ModelResult<()> {
        for category in data {
            sqlx::query(
                r"
                INSERT INTO categories (
                    id,
                    pid,
                    name,
                    image_link,
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
                    name = EXCLUDED.name,
                    image_link = EXCLUDED.image_link,
                    description = EXCLUDED.description,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at,
                    deleted_at = EXCLUDED.deleted_at
            ",
            )
            .bind(category.id())
            .bind(category.pid())
            .bind(category.name())
            .bind(category.image_link())
            .bind(category.description())
            .bind(category.created_at())
            .bind(category.updated_at())
            .bind(category.deleted_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
