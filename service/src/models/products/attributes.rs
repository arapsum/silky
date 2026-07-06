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
    /// Creates an attribute or returns the existing row with the same normalized name.
    ///
    /// The supplied name is trimmed and stored in lowercase before lookup and
    /// insertion.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the lookup and insert.
    /// - `name`: Attribute name to normalize and persist.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup or insert fails.
    pub async fn create<'e, E>(db: &E, name: &str) -> ModelResult<Self>
    where
        for<'a> &'a E: Executor<'e, Database = Postgres>,
    {
        if let Some(exists) = sqlx::query_as::<_, Self>(r"SELECT * FROM attributes WHERE name = $1")
            .bind(name.to_lowercase().trim())
            .fetch_optional(db)
            .await?
        {
            return Ok(exists);
        }

        let attr = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO attributes (name) VALUES ($1) RETURNING *
        ",
        )
        .bind(name.to_lowercase().trim())
        .fetch_one(db)
        .await?;

        Ok(attr)
    }

    /// Updates an attribute name by public ID.
    ///
    /// The supplied name is trimmed and stored in lowercase.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the duplicate lookup and update.
    /// - `pid`: Public ID of the attribute to update.
    /// - `name`: Replacement attribute name.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityAlreadyExists`] when another attribute
    /// already uses the normalized name. Returns a database error if the lookup
    /// or update fails.
    pub async fn update<'e, E>(db: &E, pid: Uuid, name: &str) -> ModelResult<Self>
    where
        for<'a> &'a E: Executor<'e, Database = Postgres>,
    {
        if let Some(exists) = sqlx::query_as::<_, Self>(r"SELECT * FROM attributes WHERE name = $1")
            .bind(name.to_lowercase().trim())
            .fetch_optional(db)
            .await?
        {
            return Err(ModelError::EntityAlreadyExists(format!(
                "Attribute {} already exists!",
                exists.name()
            )));
        }

        let attr = sqlx::query_as::<_, Self>(
            r"
            UPDATE attributes
            SET name = COALESCE($1, name)
            WHERE pid = $2
            RETURNING *
        ",
        )
        .bind(name.to_lowercase().trim())
        .bind(pid)
        .fetch_one(db)
        .await?;

        Ok(attr)
    }

    /// Finds an attribute by public ID.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the lookup.
    /// - `pid`: Public ID of the attribute to find.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no attribute has the given
    /// public ID. Returns a database error if the lookup fails.
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

    /// Finds an attribute by normalized name.
    ///
    /// The supplied name is trimmed and lowercased before lookup.
    ///
    /// # Parameters
    ///
    /// - `db`: Database executor used for the lookup.
    /// - `name`: Attribute name to normalize and find.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no attribute has the
    /// normalized name. Returns a database error if the lookup fails.
    pub async fn find_by_name<'e, E>(db: E, name: &str) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let attribute = sqlx::query_as::<_, Self>(
            r"
                SELECT * FROM attributes WHERE name = $1
        ",
        )
        .bind(name.to_lowercase().trim())
        .fetch_optional(db)
        .await?;

        attribute.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Loads attributes from a JSON file in `src/data` and seeds them.
    ///
    /// # Parameters
    ///
    /// - `db`: Database pool used for inserts and updates.
    /// - `file`: File name relative to `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialization, or database error if loading,
    /// decoding, inserting, or updating the attributes fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }

    /// Returns the internal database row ID.
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    /// Returns the public attribute ID.
    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    /// Returns the normalized attribute name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns when the attribute was created.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    /// Returns when the attribute was last updated.
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
