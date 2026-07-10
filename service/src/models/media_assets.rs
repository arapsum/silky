use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use super::{ModelError, ModelResult};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MediaAsset {
    id: i32,
    pid: Uuid,
    provider: String,
    resource_type: String,
    public_id: String,
    asset_id: Option<String>,
    secure_url: String,
    folder: String,
    format: Option<String>,
    bytes: Option<i64>,
    width: Option<i32>,
    height: Option<i32>,
    checksum: Option<String>,
    status: String,
    created_by: Option<i32>,
    orphaned_at: Option<DateTime<FixedOffset>>,
    purge_after: Option<DateTime<FixedOffset>>,
    deleted_at: Option<DateTime<FixedOffset>>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalizeMediaAsset {
    pub public_id: String,
    pub asset_id: Option<String>,
    pub secure_url: String,
    pub format: Option<String>,
    pub bytes: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub checksum: Option<String>,
}

impl MediaAsset {
    /// Creates a pending media asset owned by the user identified by `user_pid`.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when the user does not exist, or
    /// a database error when the asset cannot be persisted.
    pub async fn create_pending(
        db: &PgPool,
        user_pid: Uuid,
        public_id: &str,
        folder: &str,
        checksum: Option<&str>,
    ) -> ModelResult<Self> {
        let asset = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO media_assets (created_by, public_id, folder, secure_url, checksum, status)
            SELECT users.id, $2, $3, '', $4, 'pending'
            FROM users
            WHERE users.pid = $1
            RETURNING *
            ",
        )
        .bind(user_pid)
        .bind(public_id)
        .bind(folder)
        .bind(checksum)
        .fetch_optional(db)
        .await?
        .ok_or(ModelError::EntityNotFound)?;

        Ok(asset)
    }

    /// Finalizes a pending media asset using the metadata returned by Cloudinary.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when the asset is missing, has
    /// already been finalized, or does not match the supplied public ID. Returns
    /// a database error when the update cannot be completed.
    pub async fn finalize(db: &PgPool, pid: Uuid, input: &FinalizeMediaAsset) -> ModelResult<Self> {
        let asset = sqlx::query_as::<_, Self>(
            r"
            UPDATE media_assets
            SET
                public_id = $9,
                asset_id = COALESCE($2, asset_id),
                secure_url = $3,
                format = COALESCE($4, format),
                bytes = COALESCE($5, bytes),
                width = COALESCE($6, width),
                height = COALESCE($7, height),
                checksum = COALESCE($8, checksum),
                status = 'active'
            WHERE pid = $1
                AND (public_id = $9 OR CONCAT(folder, '/', public_id) = $9)
                AND status = 'pending'
            RETURNING *
            ",
        )
        .bind(pid)
        .bind(&input.public_id)
        .bind(&input.secure_url)
        .bind(&input.format)
        .bind(input.bytes)
        .bind(input.width)
        .bind(input.height)
        .bind(&input.checksum)
        .bind(&input.public_id)
        .fetch_optional(db)
        .await?
        .ok_or(ModelError::EntityNotFound)?;

        Ok(asset)
    }

    /// Finds a media asset by its public ID.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no asset matches `pid`, or a
    /// database error when the lookup fails.
    pub async fn find_by_pid(db: &PgPool, pid: Uuid) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>("SELECT * FROM media_assets WHERE pid = $1")
            .bind(pid)
            .fetch_optional(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    /// Lists the most recently created non-deleted media assets.
    ///
    /// # Errors
    ///
    /// Returns a database error when the media assets cannot be queried.
    pub async fn list(db: &PgPool) -> ModelResult<Vec<Self>> {
        Ok(sqlx::query_as::<_, Self>(
            "SELECT * FROM media_assets WHERE status <> 'deleted' ORDER BY created_at DESC LIMIT 100",
        )
        .fetch_all(db)
        .await?)
    }

    /// Quarantines active assets that have no user, category, or picture reference.
    ///
    /// # Errors
    ///
    /// Returns a database error when orphaned assets cannot be quarantined.
    pub async fn quarantine_orphans(db: &PgPool) -> ModelResult<Vec<Self>> {
        Ok(sqlx::query_as::<_, Self>(
            r"
            UPDATE media_assets
            SET status = 'quarantined',
                orphaned_at = COALESCE(orphaned_at, NOW()),
                purge_after = COALESCE(purge_after, NOW() + INTERVAL '30 days')
            WHERE status = 'active'
              AND NOT EXISTS (SELECT 1 FROM users WHERE users.media_asset_id = media_assets.id)
              AND NOT EXISTS (SELECT 1 FROM categories WHERE categories.media_asset_id = media_assets.id)
              AND NOT EXISTS (SELECT 1 FROM pictures WHERE pictures.media_asset_id = media_assets.id)
            RETURNING *
            ",
        )
        .fetch_all(db)
        .await?)
    }

    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    #[must_use]
    pub fn public_id(&self) -> &str {
        &self.public_id
    }

    #[must_use]
    pub fn folder(&self) -> &str {
        &self.folder
    }

    #[must_use]
    pub fn secure_url(&self) -> &str {
        &self.secure_url
    }
}
