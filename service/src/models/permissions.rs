use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelError, ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    id: i32,
    pid: Uuid,
    name: String,
    description: Option<String>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl Permission {
    /// Finds a permission by public ID.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no permission exists for
    /// `pid`. Returns a database error if the lookup fails.
    pub async fn find_by_pid<'e, C>(db: C, pid: Uuid) -> ModelResult<Self>
    where
        C: Executor<'e, Database = Postgres>,
    {
        let permission = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM permissions WHERE pid = $1
        ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)?;

        Ok(permission)
    }

    /// Lists all permissions ordered by newest creation time first.
    ///
    /// # Errors
    ///
    /// Returns a database error if the permissions query fails.
    pub async fn find_list<'e, C>(db: C) -> ModelResult<Vec<Self>>
    where
        C: Executor<'e, Database = Postgres>,
    {
        let permissions = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM permissions
            ORDER BY created_at DESC
        ",
        )
        .fetch_all(db)
        .await?;

        Ok(permissions)
    }

    /// Seeds permissions from a file in `src/data`.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::FileNotFound`] when the file is missing,
    /// [`ModelError::UnsupportedFileType`] when the extension is not supported,
    /// deserialization errors for invalid seed data, or database errors when
    /// inserting the loaded permissions fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let permissions = Self::load(file).await?;

        Self::seed(db, &permissions).await
    }
}

impl Seedable for Permission {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for permission in data {
            sqlx::query(
                r"
                INSERT INTO permissions (
                    id,
                    pid,
                    name,
                    description,
                    created_at,
                    updated_at
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6
                ) ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at

            ",
            )
            .bind(permission.id)
            .bind(permission.pid)
            .bind(permission.name.to_lowercase())
            .bind(permission.description.as_ref())
            .bind(permission.created_at)
            .bind(permission.updated_at)
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
