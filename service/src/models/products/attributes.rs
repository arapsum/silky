use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelError, ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    id: i32,
    pid: Uuid,
    name: String,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl Attribute {
    /// Finds the [`Attribute`] by its PID
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - No [`Attribute`] with that PID is found.
    /// - The database connection cannot be established.
    /// - The table has not been created.
    pub async fn find_by_pid<'e, E>(db: E, pid: Uuid) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let attribute = sqlx::query_as::<_, Self>(
            r"
                SELECT * FROM attributes WHERE pid = $1
        ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?;

        attribute.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Finds the [`Attribute`] by its name
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - No [`Attribute`] with that name is found.
    /// - The database connection cannot be established.
    /// - The table has not been created.
    pub async fn find_by_name<'e, E>(db: E, name: &str) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let attribute = sqlx::query_as::<_, Self>(
            r"
                SELECT * FROM attributes WHERE name = $1
        ",
        )
        .bind(name)
        .fetch_optional(db)
        .await?;

        attribute.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Seeds attributes from a file in `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialisation, or database error if loading or
    /// inserting the loaded attributes fails.
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
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }
}

impl Seedable for Attribute {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for attribute in data {
            sqlx::query(
                r"
                INSERT INTO attributes (
                    id,
                    pid,
                    name,
                    created_at,
                    updated_at
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5
                ) ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    name = EXCLUDED.name,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at
                ",
            )
            .bind(attribute.id())
            .bind(attribute.pid())
            .bind(attribute.name())
            .bind(attribute.created_at())
            .bind(attribute.updated_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
