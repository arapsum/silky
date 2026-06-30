use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::schemas::{AssignPermission, PermissionRoleQuery};

use super::{ModelError, ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct RolePermission {
    id: i32,
    pid: Uuid,
    role_id: i32,
    permission_id: i32,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl RolePermission {
    /// Assigns a permission to a role.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityAlreadyExists`] when the role already has
    /// the permission. Returns a database error if the duplicate lookup,
    /// insertion, or transaction commit fails.
    pub async fn assign_permission(db: &PgPool, params: &AssignPermission) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        if let Some(exists) = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM roles_permissions
            WHERE
                role_id = $1
            AND
                permission_id = $2
        ",
        )
        .bind(params.role_id())
        .bind(params.permission_id())
        .fetch_optional(&mut *txn)
        .await?
        {
            tracing::error!("[role-permission] role already exists. {:?}", exists);
            return Err(ModelError::EntityAlreadyExists(
                "Role already has permission assigned".into(),
            ));
        }

        let role_permission = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO roles_permissions (
                role_id, permission_id
            ) VALUES ( $1, $2 )
            RETURNING *
        ",
        )
        .bind(params.role_id())
        .bind(params.permission_id())
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(role_permission)
    }

    /// Finds role-permission assignments, optionally filtered by role or permission.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup fails.
    pub async fn find_all<'e, E>(db: E, query: PermissionRoleQuery) -> ModelResult<Vec<Self>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let roles_permissions = sqlx::query_as::<_, Self>(
            r"
            SELECT rp.*
            FROM roles_permissions rp
            WHERE ($1::INT IS NULL OR rp.role_id = $1)
                AND ($2::INT IS NULL OR rp.permission_id = $2)
            ORDER BY rp.created_at DESC
        ",
        )
        .bind(query.role_id())
        .bind(query.permission_id())
        .fetch_all(db)
        .await?;

        Ok(roles_permissions)
    }

    /// Finds a role-permission assignment by public ID.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no role-permission
    /// assignment exists for `pid`. Returns a database error if the lookup
    /// fails.
    pub async fn find_by_pid<'e, E>(db: E, pid: Uuid) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let role_permission = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM roles_permissions
            WHERE pid = $1
        ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?;

        role_permission.ok_or_else(|| ModelError::EntityNotFound)
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
    pub const fn role_id(&self) -> i32 {
        self.role_id
    }

    #[must_use]
    pub const fn permission_id(&self) -> i32 {
        self.permission_id
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }

    /// Seeds role-permission assignments from a file in `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialisation, or database error if loading or
    /// inserting the loaded assignments fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }
}

impl Seedable for RolePermission {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for rp in data {
            sqlx::query(
                r"
                INSERT INTO roles_permissions (
                    id,
                    pid,
                    role_id,
                    permission_id,
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
                    role_id = EXCLUDED.role_id,
                    permission_id = EXCLUDED.permission_id,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at

            ",
            )
            .bind(rp.id())
            .bind(rp.pid())
            .bind(rp.role_id())
            .bind(rp.permission_id())
            .bind(rp.created_at())
            .bind(rp.updated_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
