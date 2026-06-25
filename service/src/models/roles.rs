use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::{
    models::{ModelError, ModelResult, Seedable},
    schemas::NewRole,
};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    id: i32,
    pid: Uuid,
    name: String,
    description: Option<String>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl Role {
    /// Creates a new role.
    ///
    /// The role name is trimmed and stored in lowercase before insertion. Role
    /// names must be unique after this normalization.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityAlreadyExists`] when a role with the same
    /// normalized name already exists. Returns a database error if the lookup,
    /// insert, or transaction commit fails.
    pub async fn create(db: &PgPool, params: &NewRole<'_>) -> ModelResult<Self> {
        let mut txn = db.begin().await?;
        let name = params.name().trim().to_lowercase();
        let description = params.description().map(std::convert::AsRef::as_ref);

        if let Some(role) = sqlx::query_as::<_, Self>(r"SELECT * FROM roles WHERE name = $1")
            .bind(&name)
            .fetch_optional(&mut *txn)
            .await?
        {
            return Err(ModelError::EntityAlreadyExists(format!(
                "Role {} already exists",
                &role.name
            )));
        }

        let new_role = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO roles (
                name,
                description
            ) VALUES (
                $1,
                $2
            ) RETURNING *
        ",
        )
        .bind(name)
        .bind(description)
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(new_role)
    }

    /// Seeds roles from a file in `src/data`.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::FileNotFound`] when the file is missing,
    /// [`ModelError::UnsupportedFileType`] when the extension is not supported,
    /// deserialization errors for invalid seed data, or database errors when
    /// inserting the loaded roles fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let roles = Self::load(file).await?;

        Self::seed(db, &roles).await
    }
}

impl Seedable for Role {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for role in data {
            sqlx::query(
                r"
                INSERT INTO roles (
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
            .bind(role.id)
            .bind(role.pid)
            .bind(role.name.to_lowercase())
            .bind(role.description.as_ref())
            .bind(role.created_at)
            .bind(role.updated_at)
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
