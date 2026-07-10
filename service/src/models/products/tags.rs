use std::collections::HashSet;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::models::{ModelError, ModelResult};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    id: i32,
    pid: Uuid,
    name: String,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl Tag {
    /// Creates a reusable product tag.
    ///
    /// The name is trimmed before it is stored and is unique without regard
    /// to letter case.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityAlreadyExists`] when the normalized name is
    /// already in use, [`ModelError::InvalidInput`] when it is empty or longer
    /// than 64 characters, or a database error when the insert fails.
    pub async fn create<'e, E>(db: E, name: &str) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        Ok(
            sqlx::query_as::<_, Self>("INSERT INTO tags (name) VALUES ($1) RETURNING *")
                .bind(name.trim())
                .fetch_one(db)
                .await?,
        )
    }

    /// Updates a tag name by public ID.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when the tag does not exist,
    /// [`ModelError::EntityAlreadyExists`] when the normalized name is already
    /// in use, or a database error when the update fails.
    pub async fn update<'e, E>(db: E, pid: Uuid, name: &str) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, Self>("UPDATE tags SET name = $2 WHERE pid = $1 RETURNING *")
            .bind(pid)
            .bind(name.trim())
            .fetch_optional(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    /// Finds a tag by public ID.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no tag matches the public
    /// ID, or a database error when the lookup fails.
    pub async fn find_by_pid<'e, E>(db: E, pid: Uuid) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, Self>("SELECT * FROM tags WHERE pid = $1")
            .bind(pid)
            .fetch_optional(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    /// Finds a tag by its case-insensitive normalized name.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no tag matches the name, or
    /// a database error when the lookup fails.
    pub async fn find_by_name<'e, E>(db: E, name: &str) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, Self>("SELECT * FROM tags WHERE LOWER(name) = LOWER($1)")
            .bind(name.trim())
            .fetch_optional(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    /// Lists all tags in case-insensitive name order.
    ///
    /// # Errors
    ///
    /// Returns a database error when the tags cannot be queried.
    pub async fn find_all<'e, E>(db: E) -> ModelResult<Vec<Self>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        Ok(
            sqlx::query_as::<_, Self>("SELECT * FROM tags ORDER BY LOWER(name), id")
                .fetch_all(db)
                .await?,
        )
    }

    /// Lists the tags assigned to a product identified by public ID.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when the product does not exist,
    /// or a database error when its tags cannot be queried.
    pub async fn find_by_product(db: &PgPool, product_pid: Uuid) -> ModelResult<Vec<Self>> {
        let product_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM products WHERE pid = $1 AND deleted_at IS NULL)",
        )
        .bind(product_pid)
        .fetch_one(db)
        .await?;

        if !product_exists {
            return Err(ModelError::EntityNotFound);
        }

        Ok(sqlx::query_as::<_, Self>(
            r"
            SELECT t.*
            FROM tags t
            INNER JOIN product_tags pt ON pt.tag_id = t.id
            INNER JOIN products p ON p.id = pt.product_id
            WHERE p.pid = $1
            ORDER BY LOWER(t.name), t.id
            ",
        )
        .bind(product_pid)
        .fetch_all(db)
        .await?)
    }

    /// Assigns a tag to a product without duplicating an existing link.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when either record does not
    /// exist, or a database error when the link cannot be persisted.
    pub async fn add_to_product(
        db: &PgPool,
        product_pid: Uuid,
        tag_pid: Uuid,
    ) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"
            WITH selected AS (
                SELECT p.id AS product_id, t.id AS tag_id
                FROM products p
                CROSS JOIN tags t
                WHERE p.pid = $1 AND p.deleted_at IS NULL AND t.pid = $2
            ), inserted AS (
                INSERT INTO product_tags (product_id, tag_id)
                SELECT product_id, tag_id FROM selected
                ON CONFLICT DO NOTHING
            )
            SELECT t.*
            FROM tags t
            INNER JOIN selected s ON s.tag_id = t.id
            ",
        )
        .bind(product_pid)
        .bind(tag_pid)
        .fetch_optional(db)
        .await?
        .ok_or(ModelError::EntityNotFound)
    }

    /// Removes a tag from a product.
    ///
    /// The returned value reports whether an assignment was removed.
    ///
    /// # Errors
    ///
    /// Returns a database error when the link cannot be removed.
    pub async fn remove_from_product(
        db: &PgPool,
        product_pid: Uuid,
        tag_pid: Uuid,
    ) -> ModelResult<bool> {
        let result = sqlx::query(
            r"
            DELETE FROM product_tags
            WHERE product_id = (SELECT id FROM products WHERE pid = $1)
              AND tag_id = (SELECT id FROM tags WHERE pid = $2)
            ",
        )
        .bind(product_pid)
        .bind(tag_pid)
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Replaces every tag assignment for a product atomically.
    ///
    /// Duplicate public IDs in `tag_pids` are ignored.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when the product or any supplied
    /// tag does not exist, or a database error when the assignments cannot be
    /// replaced.
    pub async fn set_for_product(
        db: &PgPool,
        product_pid: Uuid,
        tag_pids: &[Uuid],
    ) -> ModelResult<Vec<Self>> {
        let unique_tag_pids = tag_pids.iter().copied().collect::<HashSet<_>>();
        let unique_tag_pids = unique_tag_pids.into_iter().collect::<Vec<_>>();
        let mut txn = db.begin().await?;
        let catalog_product_id = sqlx::query_scalar::<_, i32>(
            "SELECT id FROM products WHERE pid = $1 AND deleted_at IS NULL",
        )
        .bind(product_pid)
        .fetch_optional(&mut *txn)
        .await?
        .ok_or(ModelError::EntityNotFound)?;

        let tags = if unique_tag_pids.is_empty() {
            Vec::new()
        } else {
            sqlx::query_as::<_, Self>("SELECT * FROM tags WHERE pid = ANY($1::uuid[])")
                .bind(&unique_tag_pids)
                .fetch_all(&mut *txn)
                .await?
        };

        if tags.len() != unique_tag_pids.len() {
            return Err(ModelError::EntityNotFound);
        }

        sqlx::query("DELETE FROM product_tags WHERE product_id = $1")
            .bind(catalog_product_id)
            .execute(&mut *txn)
            .await?;

        if !tags.is_empty() {
            let internal_tag_ids = tags.iter().map(Self::id).collect::<Vec<_>>();
            sqlx::query(
                r"
                INSERT INTO product_tags (product_id, tag_id)
                SELECT $1, UNNEST($2::integer[])
                ",
            )
            .bind(catalog_product_id)
            .bind(internal_tag_ids)
            .execute(&mut *txn)
            .await?;
        }

        txn.commit().await?;

        Self::find_by_product(db, product_pid).await
    }

    /// Deletes a tag and all of its product assignments.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when the tag does not exist, or
    /// a database error when it cannot be deleted.
    pub async fn delete<'e, E>(db: E, pid: Uuid) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, Self>("DELETE FROM tags WHERE pid = $1 RETURNING *")
            .bind(pid)
            .fetch_optional(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    /// Returns the internal database row ID.
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    /// Returns the public tag ID.
    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    /// Returns the display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns when the tag was created.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    /// Returns when the tag was last updated.
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }
}
