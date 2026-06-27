use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::schemas::AssignRole;

use super::{ModelError, ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct UserRole {
    id: i32,
    pid: Uuid,

    user_id: i32,
    role_id: i32,

    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl UserRole {
    /// Assigns a role to a user.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityAlreadyExists`] when the user already has
    /// the role. Returns a database error if the duplicate lookup, insertion,
    /// or transaction commit fails.
    pub async fn assign_role(db: &PgPool, params: &AssignRole) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        if let Some(_exists) = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM users_roles
            WHERE
                user_id = $1
            AND
                role_id = $2
        ",
        )
        .bind(params.user_id())
        .bind(params.role_id())
        .fetch_optional(&mut *txn)
        .await?
        {
            return Err(ModelError::EntityAlreadyExists(
                "User has already been assigned that role".into(),
            ));
        }

        let role = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO users_roles (
                user_id,
                role_id
            ) VALUES (
               $1,
               $2
            ) RETURNING *
        ",
        )
        .bind(params.user_id())
        .bind(params.role_id())
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(role)
    }

    /// Finds all role assignments for a user.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup fails.
    pub async fn find_by_user<'e, E>(db: E, user_id: i32) -> ModelResult<Vec<Self>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let user_roles = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM users_roles
            WHERE user_id = $1
            ORDER BY id ASC
        ",
        )
        .bind(user_id)
        .fetch_all(db)
        .await?;

        Ok(user_roles)
    }

    /// Finds all user assignments for a role.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup fails.
    pub async fn find_by_role<'e, E>(db: E, role_id: i32) -> ModelResult<Vec<Self>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let user_roles = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM users_roles
            WHERE role_id = $1
            ORDER BY id ASC
        ",
        )
        .bind(role_id)
        .fetch_all(db)
        .await?;

        Ok(user_roles)
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
    pub const fn user_id(&self) -> i32 {
        self.user_id
    }

    #[must_use]
    pub const fn role_id(&self) -> i32 {
        self.role_id
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }

    /// Seeds user-role assignments from a file in `src/data`.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::FileNotFound`] when the file is missing,
    /// [`ModelError::UnsupportedFileType`] when the extension is not supported,
    /// deserialization errors for invalid seed data, or database errors when
    /// inserting the loaded assignments fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }
}

impl Seedable for UserRole {
    async fn seed(db: &sqlx::PgPool, data: &[Self]) -> ModelResult<()> {
        for user_role in data {
            sqlx::query(
                r"
                INSERT INTO users_roles (
                    id,
                    pid,
                    user_id,
                    role_id,
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
                   user_id = EXCLUDED.user_id,
                   role_id = EXCLUDED.role_id,
                   created_at = EXCLUDED.created_at,
                   updated_at = EXCLUDED.updated_at
            ",
            )
            .bind(user_role.id())
            .bind(user_role.pid())
            .bind(user_role.user_id())
            .bind(user_role.role_id())
            .bind(user_role.created_at())
            .bind(user_role.updated_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
