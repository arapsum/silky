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
    /// Checks whether a user has the requested permission through any assigned role.
    ///
    /// The user must be assigned to a role through `users_roles`, and that same
    /// role must be linked to `permission` through
    /// `roles_permissions`.
    ///
    /// # Errors
    ///
    /// Returns a database error if the authorization query fails.
    pub async fn is_granted_to_user_role<'e, C>(
        db: C,
        user_pid: Uuid,
        permission: &str,
    ) -> ModelResult<bool>
    where
        C: Executor<'e, Database = Postgres>,
    {
        let exists = sqlx::query_scalar::<_, bool>(
            r"
            SELECT EXISTS (
                SELECT 1
                FROM users
                INNER JOIN users_roles
                    ON users_roles.user_id = users.id
                INNER JOIN roles
                    ON roles.id = users_roles.role_id
                INNER JOIN roles_permissions
                    ON roles_permissions.role_id = roles.id
                INNER JOIN permissions
                    ON permissions.id = roles_permissions.permission_id
                WHERE users.pid = $1
                    AND permissions.name = $2
            )
        ",
        )
        .bind(user_pid)
        .bind(permission)
        .fetch_one(db)
        .await?;

        Ok(exists)
    }

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

    /// Lists permissions ordered by newest creation time first.
    ///
    /// When `role` is provided, only permissions assigned to that role are
    /// returned.
    ///
    /// # Errors
    ///
    /// Returns a database error if the permissions query fails.
    pub async fn find_list<'e, C>(db: C, role: Option<&str>) -> ModelResult<Vec<Self>>
    where
        C: Executor<'e, Database = Postgres>,
    {
        let role = role
            .map(|role| role.trim().to_lowercase())
            .filter(|role| !role.is_empty());

        let permissions = sqlx::query_as::<_, Self>(
            r"
            SELECT permissions.*
            FROM permissions
            WHERE $1::TEXT IS NULL
                OR EXISTS (
                    SELECT 1
                    FROM roles_permissions
                    INNER JOIN roles
                        ON roles.id = roles_permissions.role_id
                    WHERE roles_permissions.permission_id = permissions.id
                        AND roles.name = $1
                )
            ORDER BY permissions.created_at DESC, permissions.id DESC
        ",
        )
        .bind(role.as_deref())
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
