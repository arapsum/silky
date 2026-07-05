use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, Row, prelude::FromRow};
use uuid::Uuid;

use crate::{
    models::{ModelError, ModelResult, Seedable},
    schemas::{NewRole, UpdateRole},
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoleUser {
    pid: Uuid,
    name: String,
    email: String,
    image: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoleWithUsers {
    id: i32,
    pid: Uuid,
    name: String,
    description: Option<String>,
    users: Vec<RoleUser>,
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

    fn role_with_users_from_row(row: &sqlx::postgres::PgRow) -> Result<RoleWithUsers, sqlx::Error> {
        let users_json = row.try_get::<serde_json::Value, _>("users")?;
        let users =
            serde_json::from_value(users_json).map_err(|err| sqlx::Error::ColumnDecode {
                index: "users".into(),
                source: Box::new(err),
            })?;

        Ok(RoleWithUsers {
            id: row.try_get("id")?,
            pid: row.try_get("pid")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            users,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    /// Updates an existing role by public ID.
    ///
    /// The role name is trimmed and stored in lowercase when provided. A name
    /// can be reused by the same role after normalization, but cannot collide
    /// with another role's normalized name.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no role exists for `pid`.
    /// Returns [`ModelError::EntityAlreadyExists`] when another role already
    /// uses the requested normalized name. Returns a database error if the
    /// lookup, update, or transaction commit fails.
    pub async fn update(db: &PgPool, pid: Uuid, params: &UpdateRole<'_>) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        let exists = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM roles WHERE pid = $1
        ",
        )
        .bind(pid)
        .fetch_optional(&mut *txn)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)?;

        if let Some(name) = params.name() {
            let name_exists = sqlx::query_as::<_, Self>(
                r"
                SELECT * FROM roles WHERE name = $1
            ",
            )
            .bind(name.to_lowercase().trim())
            .fetch_optional(&mut *txn)
            .await?;

            if let Some(role) = name_exists
                && role.pid != exists.pid
            {
                return Err(ModelError::EntityAlreadyExists(format!(
                    "Role {} already exists",
                    &role.name
                )));
            }
        }

        let updated_role = sqlx::query_as::<_, Self>(
            r"
            UPDATE roles
            SET
                name = COALESCE($1, name),
                description = COALESCE($2, description)
            WHERE pid = $3
            RETURNING *
        ",
        )
        .bind(params.name().map(|s| s.trim().to_lowercase()))
        .bind(params.description())
        .bind(pid)
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(updated_role)
    }

    /// Finds a role by public ID.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no role exists for `pid`.
    /// Returns a database error if the lookup fails.
    pub async fn find_by_pid<'e, C>(db: C, pid: Uuid) -> ModelResult<Self>
    where
        C: Executor<'e, Database = Postgres>,
    {
        let role = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM roles WHERE pid = $1
        ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)?;

        Ok(role)
    }

    /// Finds a role by public ID with assigned users.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no role exists for `pid`.
    /// Returns a database error if the lookup fails.
    pub async fn find_with_users_by_pid(db: &PgPool, pid: Uuid) -> ModelResult<RoleWithUsers> {
        let row = sqlx::query(
            r"
            SELECT
                roles.id,
                roles.pid,
                roles.name,
                roles.description,
                roles.created_at,
                roles.updated_at,
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object(
                            'pid', users.pid,
                            'name', users.name,
                            'email', users.email,
                            'image', users.image
                        )
                        ORDER BY users.name
                    ) FILTER (WHERE users.id IS NOT NULL),
                    '[]'::jsonb
                ) AS users
            FROM roles
            LEFT JOIN users_roles ON users_roles.role_id = roles.id
            LEFT JOIN users ON users.id = users_roles.user_id
            WHERE roles.pid = $1
            GROUP BY roles.id
        ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)?;

        Ok(Self::role_with_users_from_row(&row)?)
    }

    /// Lists all roles ordered by newest creation time first.
    ///
    /// # Errors
    ///
    /// Returns a database error if the role query fails.
    pub async fn find_list<'e, C>(db: C) -> ModelResult<Vec<Self>>
    where
        C: Executor<'e, Database = Postgres>,
    {
        let roles = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM roles
            ORDER BY created_at DESC
        ",
        )
        .fetch_all(db)
        .await?;

        Ok(roles)
    }

    /// Lists all roles ordered by newest creation time first, with assigned
    /// users aggregated on each role.
    ///
    /// # Errors
    ///
    /// Returns a database error if the role query fails.
    pub async fn find_list_with_users(db: &PgPool) -> ModelResult<Vec<RoleWithUsers>> {
        let rows = sqlx::query(
            r"
            SELECT
                roles.id,
                roles.pid,
                roles.name,
                roles.description,
                roles.created_at,
                roles.updated_at,
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object(
                            'pid', users.pid,
                            'name', users.name,
                            'email', users.email,
                            'image', users.image
                        )
                        ORDER BY users.name
                    ) FILTER (WHERE users.id IS NOT NULL),
                    '[]'::jsonb
                ) AS users
            FROM roles
            LEFT JOIN users_roles ON users_roles.role_id = roles.id
            LEFT JOIN users ON users.id = users_roles.user_id
            GROUP BY roles.id
            ORDER BY roles.created_at DESC
        ",
        )
        .fetch_all(db)
        .await?;

        let roles = rows
            .iter()
            .map(Self::role_with_users_from_row)
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        Ok(roles)
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

    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
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
