#![allow(unused_imports)]
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::schemas::{NewCategory, PaginationQuery, UpdateCategory};

use super::{ModelError, ModelResult, PaginatedModel, Pagination, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    id: i32,
    pid: Uuid,
    name: String,
    image_link: String,
    description: Option<String>,
    parent_id: Option<i32>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

impl Category {
    /// Creates a new category.
    ///
    /// The category name is trimmed and stored in lowercase before insertion.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityAlreadyExists`] when a category with the
    /// same normalized name exists. Returns a database error if the duplicate
    /// lookup, insertion, or transaction commit fails.
    pub async fn create(db: &PgPool, params: &NewCategory<'_>) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        if let Some(category) = Self::find_by_name(&mut *txn, params.name()).await? {
            return Err(ModelError::EntityAlreadyExists(format!(
                "Category {} already exists!",
                category.name()
            )));
        }

        let created = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO categories (
                name,
                image_link,
                parent_id,
                description
            ) VALUES (
                $1,
                $2,
                $3,
                $4
            ) RETURNING *
        ",
        )
        .bind(params.name().to_lowercase().trim())
        .bind(params.image_link().trim())
        .bind(params.parent_id())
        .bind(params.description().map(|s| s.trim()))
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(created)
    }

    /// Updates an existing category by public ID.
    ///
    /// Provided string fields are trimmed, and the category name is stored in
    /// lowercase when changed.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no category exists for
    /// `pid`. Returns [`ModelError::EntityAlreadyExists`] when the requested
    /// name belongs to another category. Returns a database error if the lookup,
    /// update, or transaction commit fails.
    pub async fn update(db: &PgPool, pid: Uuid, params: &UpdateCategory<'_>) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        let exists = Self::find_by_pid(&mut *txn, pid).await?;

        if let Some(name) = params.name()
            && let Some(category) = Self::find_by_name(&mut *txn, name).await?
            && category.pid() != exists.pid()
        {
            return Err(ModelError::EntityAlreadyExists(format!(
                "Category {} already exists!",
                category.name()
            )));
        }

        let updated = sqlx::query_as::<_, Self>(
            r"
                UPDATE categories
                SET
                    name = COALESCE($1, name),
                    image_link = COALESCE($2, image_link),
                    parent_id = COALESCE($3, parent_id),
                    description = COALESCE($4, description)
                WHERE pid = $5
                RETURNING *
        ",
        )
        .bind(params.name().map(|s| s.trim().to_lowercase()))
        .bind(params.image_link().map(|s| s.trim()))
        .bind(params.parent_id())
        .bind(params.description().map(|s| s.trim()))
        .bind(exists.pid())
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(updated)
    }

    /// Lists categories with pagination metadata.
    ///
    /// Defaults to page `1` and limit `20` when query values are missing. The
    /// limit is clamped to the range `1..=40`, and the page is clamped to a
    /// minimum of `1`.
    ///
    /// # Errors
    ///
    /// Returns a database error if counting or fetching categories fails, or if
    /// the transaction commit fails.
    pub async fn find_all(
        db: &PgPool,
        query: &PaginationQuery,
    ) -> ModelResult<PaginatedModel<Self>> {
        let mut txn = db.begin().await?;

        let limit = query.limit().unwrap_or(20).clamp(1, 40);
        let page = query.page().unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let total_items: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM categories")
            .fetch_one(&mut *txn)
            .await?;

        let categories = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM categories
            ORDER BY created_at DESC, id DESC
            LIMIT $1 OFFSET $2
        ",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(PaginatedModel::new(
            categories,
            Pagination::new(page, limit, total_items),
        ))
    }

    /// Finds a category by public ID.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no category exists for
    /// `pid`. Returns a database error if the lookup fails.
    pub async fn find_by_pid<'e, E>(db: E, pid: Uuid) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM categories WHERE pid = $1
        ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Finds a category by normalized name.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup fails.
    pub async fn find_by_name<'e, E>(db: E, name: &str) -> ModelResult<Option<Self>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM categories WHERE name = $1
        ",
        )
        .bind(name.to_lowercase().trim())
        .fetch_optional(db)
        .await
        .map_err(Into::into)
    }

    /// Soft deletes a category by public ID.
    ///
    /// The row is retained and `deleted_at` is set to the current database
    /// timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no category exists for
    /// `pid`. Returns a database error if the update fails.
    pub async fn delete(db: &PgPool, pid: Uuid) -> ModelResult<Self> {
        let category = sqlx::query_as::<_, Self>(
            r"
            UPDATE categories
            SET
                deleted_at = NOW()
            WHERE pid = $1
            RETURNING *
        ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)?;

        Ok(category)
    }

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

    #[must_use]
    pub const fn parent_id(&self) -> Option<i32> {
        self.parent_id
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
                    parent_id,
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
                   $8,
                   $9
                ) ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    name = EXCLUDED.name,
                    image_link = EXCLUDED.image_link,
                    description = EXCLUDED.description,
                    parent_id = EXCLUDED.parent_id,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at,
                    deleted_at = EXCLUDED.deleted_at
            ",
            )
            .bind(category.id())
            .bind(category.pid())
            .bind(category.name().to_lowercase().trim())
            .bind(category.image_link())
            .bind(category.description())
            .bind(category.parent_id())
            .bind(category.created_at())
            .bind(category.updated_at())
            .bind(category.deleted_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
