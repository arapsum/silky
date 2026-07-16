#![allow(unused)]
use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Encode, Executor, PgPool, Postgres, Row, prelude::FromRow};
use uuid::Uuid;

use crate::schemas::{ChangePassword, CreateStaffUser, LoginUser, RegisterUser, UpdateProfile};

use super::{ModelError, ModelResult, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: i32,
    // public facing identifier
    pid: Uuid,
    // user provided details
    name: String,
    email: String,
    image: Option<String>,
    password_hash: String,
    // email verification
    verified_at: Option<DateTime<FixedOffset>>,
    verification_token_hash: Option<String>,
    verification_token_expires_at: Option<DateTime<FixedOffset>>,
    // password reset
    reset_token_hash: Option<String>,
    reset_token_expires_at: Option<DateTime<FixedOffset>>,
    // Dates
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserRoleSummary {
    id: i32,
    pid: Uuid,
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserWithRoles {
    id: i32,
    pid: Uuid,
    name: String,
    email: String,
    image: Option<String>,
    verified: bool,
    roles: Vec<UserRoleSummary>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserAccess {
    roles: Vec<String>,
    permissions: Vec<String>,
}

impl User {
    /// Returns the effective roles and permissions assigned to a user.
    ///
    /// Permissions are combined across every assigned role and returned once,
    /// in a stable alphabetical order suitable for authorization clients.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when the user does not exist, or
    /// a database error when the access lookup fails.
    pub async fn find_access<'e, E>(db: E, pid: Uuid) -> ModelResult<UserAccess>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, UserAccess>(
            r"
            SELECT
                COALESCE(
                    ARRAY_AGG(DISTINCT roles.name ORDER BY roles.name)
                        FILTER (WHERE roles.id IS NOT NULL),
                    ARRAY[]::TEXT[]
                ) AS roles,
                COALESCE(
                    ARRAY_AGG(DISTINCT permissions.name ORDER BY permissions.name)
                        FILTER (WHERE permissions.id IS NOT NULL),
                    ARRAY[]::TEXT[]
                ) AS permissions
            FROM users
            LEFT JOIN users_roles
                ON users_roles.user_id = users.id
            LEFT JOIN roles
                ON roles.id = users_roles.role_id
            LEFT JOIN roles_permissions
                ON roles_permissions.role_id = roles.id
            LEFT JOIN permissions
                ON permissions.id = roles_permissions.permission_id
            WHERE users.pid = $1
            GROUP BY users.id
        ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Returns whether the user is assigned the customer role.
    ///
    /// # Errors
    /// Returns a database error when the role lookup fails.
    pub async fn has_role(db: &PgPool, pid: Uuid, role: &str) -> ModelResult<bool> {
        Ok(sqlx::query_scalar(
            r"SELECT EXISTS (
                SELECT 1
                FROM users
                JOIN users_roles ON users_roles.user_id = users.id
                JOIN roles ON roles.id = users_roles.role_id
                WHERE users.pid = $1 AND roles.name = LOWER(TRIM($2))
            )",
        )
        .bind(pid)
        .bind(role)
        .fetch_one(db)
        .await?)
    }
    /// Creates a new [`User`] and stores it in the database.
    ///
    /// The provided password is hashed using Argon2 before being persisted.
    /// The customer role is assigned in the same transaction so a storefront
    /// account is immediately eligible for customer-owned routes.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Password hashing fails.
    /// - The user record cannot be inserted into the database.
    /// - The required customer role is not configured.
    /// - Any database constraint is violated.
    pub async fn create(db: &PgPool, params: &RegisterUser<'_>) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        let exists = sqlx::query_as::<_, Self>("SELECT * FROM users WHERE email = $1")
            .bind(params.email())
            .fetch_optional(&mut *txn)
            .await?;

        if let Some(user) = exists {
            return Err(ModelError::EntityAlreadyExists(
                "User with email already exists".into(),
            ));
        }

        let password_hash = Self::hash_password(params.password())?;

        let user = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO users (name, email, password_hash, image)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        ",
        )
        .bind(params.name())
        .bind(params.email())
        .bind(password_hash)
        .bind(params.image())
        .fetch_one(&mut *txn)
        .await?;

        let assigned = sqlx::query(
            r"INSERT INTO users_roles (user_id, role_id)
              SELECT $1, roles.id
              FROM roles
              WHERE roles.name = 'customer'",
        )
        .bind(user.id)
        .execute(&mut *txn)
        .await?;
        if assigned.rows_affected() != 1 {
            return Err(ModelError::InvalidReference(
                "Customer role is not configured".into(),
            ));
        }

        txn.commit().await?;

        Ok(user)
    }

    /// Creates a staff user and assigns its initial role in one transaction.
    ///
    /// Customer accounts are created through the storefront registration flow,
    /// so the customer role is deliberately rejected here. This keeps staff
    /// provisioning separate and prevents a partially-created user without an
    /// access role when an assignment cannot be stored.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityAlreadyExists`] when the email is already
    /// registered. Returns [`ModelError::InvalidReference`] when the role does
    /// not exist, [`ModelError::InvalidInput`] when it is the customer role,
    /// or a database/password-hashing error when the transaction cannot be
    /// completed.
    pub async fn create_staff(db: &PgPool, params: &CreateStaffUser<'_>) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        let exists = sqlx::query_scalar::<_, i32>("SELECT id FROM users WHERE email = $1")
            .bind(params.email())
            .fetch_optional(&mut *txn)
            .await?;

        if exists.is_some() {
            return Err(ModelError::EntityAlreadyExists(
                "User with email already exists".into(),
            ));
        }

        let role_name = sqlx::query_scalar::<_, String>("SELECT name FROM roles WHERE id = $1")
            .bind(params.role_id())
            .fetch_optional(&mut *txn)
            .await?
            .ok_or_else(|| ModelError::InvalidReference("Staff role does not exist".into()))?;

        if role_name.eq_ignore_ascii_case("customer") {
            return Err(ModelError::InvalidInput(
                "Customer accounts must be created through the storefront registration flow".into(),
            ));
        }

        let user = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO users (name, email, password_hash, image)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        ",
        )
        .bind(params.name())
        .bind(params.email())
        .bind(Self::hash_password(params.password())?)
        .bind(params.image())
        .fetch_one(&mut *txn)
        .await?;

        sqlx::query(
            r"
            INSERT INTO users_roles (user_id, role_id)
            VALUES ($1, $2)
        ",
        )
        .bind(user.id)
        .bind(params.role_id())
        .execute(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(user)
    }

    /// Generates a new email verification token for the [`User`].
    ///
    /// # Errors
    ///
    /// This function will return an error:
    ///  - if the token could not be hashed.
    ///  - if the update query fails.
    pub async fn set_verification_token<'e, C>(
        &mut self,
        db: C,
        token: &str,
        expires_at: i64,
    ) -> ModelResult<Self>
    where
        C: Executor<'e, Database = Postgres>,
    {
        self.verification_token_hash = Some(Self::hash_text(token));

        let verification_token_expires_at = Utc::now() + Duration::seconds(expires_at);

        self.verification_token_expires_at = Some(verification_token_expires_at.fixed_offset());

        let this = sqlx::query_as::<_, Self>(
            r"
            UPDATE users
            SET
                verification_token_hash = $1,
                verification_token_expires_at = $2
            WHERE id = $3
            RETURNING *
        ",
        )
        .bind(&self.verification_token_hash)
        .bind(self.verification_token_expires_at)
        .bind(self.id)
        .fetch_one(db)
        .await?;

        Ok(this)
    }

    /// Verifies a [`User`]'s email using the provided verification token.
    ///
    /// # Errors
    ///
    /// This function will return an error:
    ///
    /// - `InvalidVerificationToken` if the token is not valid or does not match the user's verification token.
    /// - `EntityNotFound` if no user is found with the provided token.
    /// - `Sqlx` if a database error occurs.
    pub async fn verify_email(db: &PgPool, token: &str) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        let mut user = Self::find_by_verification_token(&mut *txn, token).await?;

        user = sqlx::query_as::<_, Self>(
            r"
            UPDATE users
            SET
                verified_at = NOW(),
                verification_token_hash = NULL,
                verification_token_expires_at = NULL
            WHERE id = $1
            RETURNING *
            ",
        )
        .bind(user.id)
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(user)
    }

    /// Resets the user's password using the provided token and new password.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///  - The reset token is invalid.
    ///  - The database transaction fails.
    pub async fn reset_password(db: &PgPool, token: &str, new_password: &str) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        let valid = Self::find_by_reset_token(&mut *txn, token).await?;

        if valid.reset_token_hash().is_none() {
            return Err(ModelError::InvalidResetToken);
        }

        let user = sqlx::query_as::<_, Self>(
            r"
            UPDATE users
            SET
                password_hash = $2,
                reset_token_hash = NULL,
                reset_token_expires_at = NULL
            WHERE reset_token_hash = $1
            RETURNING *
            ",
        )
        .bind(valid.reset_token_hash().as_ref())
        .bind(Self::hash_password(new_password)?)
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(user)
    }

    /// Changes a user's password after verifying their current password.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The claims key does not resolve to a user.
    /// - The current password does not match the user's stored password.
    /// - The new password cannot be hashed.
    /// - The database transaction fails.
    pub async fn change_password(
        db: &PgPool,
        claims_key: &str,
        params: &ChangePassword<'_>,
    ) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        let user = Self::find_by_claims_key(&mut *txn, claims_key).await?;
        user.verify_password(params.current_password())?;

        let user = sqlx::query_as::<_, Self>(
            r"
            UPDATE users
            SET
                password_hash = $2,
                reset_token_hash = NULL,
                reset_token_expires_at = NULL,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            ",
        )
        .bind(user.id)
        .bind(Self::hash_password(params.password())?)
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(user)
    }

    /// Updates a user's profile details.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The claims key does not resolve to a user.
    /// - The email is already used by another user.
    /// - The database transaction fails.
    pub async fn update_profile(
        db: &PgPool,
        claims_key: &str,
        params: &UpdateProfile<'_>,
    ) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        let user = Self::find_by_claims_key(&mut *txn, claims_key).await?;

        let email_owner = sqlx::query_scalar::<_, i32>("SELECT id FROM users WHERE email = $1")
            .bind(params.email())
            .fetch_optional(&mut *txn)
            .await?;

        if email_owner.is_some_and(|id| id != user.id) {
            return Err(ModelError::EntityAlreadyExists(
                "User with email already exists".into(),
            ));
        }

        let user = sqlx::query_as::<_, Self>(
            r"
            UPDATE users
            SET
                name = $2,
                email = $3,
                image = COALESCE($4, image),
                media_asset_id = COALESCE((
                    SELECT media_assets.id
                    FROM media_assets
                    WHERE media_assets.pid = $5
                      AND media_assets.created_by = (
                          SELECT users.id FROM users WHERE users.pid = $6
                      )
                      AND media_assets.status = 'active'
                ), media_asset_id),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            ",
        )
        .bind(user.id)
        .bind(params.name())
        .bind(params.email())
        .bind(params.image())
        .bind(params.media_asset_pid())
        .bind(user.pid())
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(user)
    }

    /// Finds a user by their claims key.
    ///
    /// # Errors
    ///
    /// This function will return an error:
    /// - `InvalidClaimsKey` if the claims key is not a valid UUID.
    /// - `EntityNotFound` if no user is found with the given claims key.
    /// - `Sqlx` if there is a database error.
    pub async fn find_by_claims_key<'e, C>(db: C, claims_key: &str) -> ModelResult<Self>
    where
        C: Executor<'e, Database = Postgres>,
    {
        let pid = Uuid::parse_str(claims_key).map_err(|_| ModelError::InvalidClaimsKey)?;

        let this = sqlx::query_as::<_, Self>(
            r"
            SELECT *
            FROM users
            WHERE pid = $1
            ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?;

        this.ok_or(ModelError::EntityNotFound)
    }

    /// Finds a user by their verification token.
    ///
    /// # Errors
    ///
    /// This function will return an error:
    /// - `EntityNotFound` if no user is found with the given token.
    /// - `Sqlx` if there is a database error.
    pub async fn find_by_verification_token<'e, C>(db: C, token: &str) -> ModelResult<Self>
    where
        C: Executor<'e, Database = Postgres>,
    {
        let token_hash = Self::hash_text(token);

        let this = sqlx::query_as::<_, Self>(
            r"
            SELECT *
            FROM users
            WHERE verification_token_hash = $1
            AND verification_token_expires_at > NOW()
            ",
        )
        .bind(token_hash)
        .fetch_optional(db)
        .await?;

        this.ok_or(ModelError::EntityNotFound)
    }

    /// Finds a user by their reset token.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - `EntityNotFound` if no user is found with the given token.
    /// - `Sqlx` if there is a database error.
    pub async fn find_by_reset_token<'e, C>(db: C, token: &str) -> ModelResult<Self>
    where
        C: Executor<'e, Database = Postgres>,
    {
        let token_hash = Self::hash_text(token);
        let this = sqlx::query_as::<_, Self>(
            r"
            SELECT *
            FROM users
            WHERE reset_token_hash = $1
            AND reset_token_expires_at > NOW()
            ",
        )
        .bind(token_hash)
        .fetch_optional(db)
        .await?;

        this.ok_or(ModelError::EntityNotFound)
    }

    /// Sets the reset token for the user.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - There is a database error.
    /// - The token is empty.
    pub async fn set_reset_token<'e, C>(
        &mut self,
        db: C,
        token: &str,
        expires_at: i64,
    ) -> ModelResult<Self>
    where
        C: Executor<'e, Database = Postgres>,
    {
        let token_hash = Self::hash_text(token);

        let reset_token_expires_at = Utc::now() + Duration::seconds(expires_at);

        let this = sqlx::query_as::<_, Self>(
            r"
            UPDATE users
            SET
                reset_token_hash = $1,
                reset_token_expires_at = $2
            WHERE id = $3
            RETURNING *
            ",
        )
        .bind(token_hash)
        .bind(reset_token_expires_at)
        .bind(self.id)
        .fetch_one(db)
        .await?;

        Ok(this)
    }

    /// Finds a user by their email.
    ///
    /// # Errors
    ///
    /// This function will return an error:
    /// - `EntityNotFound` if no user is found with the given email.
    /// - `Sqlx` if there is a database error.
    pub async fn find_by_email<'e, C>(db: C, email: &str) -> ModelResult<Self>
    where
        C: Executor<'e, Database = Postgres>,
    {
        let this = sqlx::query_as::<_, Self>(
            r"
            SELECT *
            FROM users
            WHERE email = $1
            ",
        )
        .bind(email)
        .fetch_optional(db)
        .await?;

        this.ok_or(ModelError::EntityNotFound)
    }

    fn hash_password(password: &str) -> ModelResult<String> {
        let argon: Argon2<'_> = Argon2::default();
        let salt: SaltString = SaltString::generate(&mut OsRng);

        Ok(argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(ModelError::PasswordHash)?
            .to_string())
    }

    /// Verifies the provided plaintext password against the user's stored hash.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - `PasswordHash` - The stored password hash is invalid.
    /// - `ArgonError` - The password verification fails.
    pub fn verify_password(&self, plain_password: &str) -> ModelResult<()> {
        let parded_hash = PasswordHash::new(&self.password_hash)?;

        Argon2::default().verify_password(plain_password.as_bytes(), &parded_hash)?;

        Ok(())
    }

    fn hash_text(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());

        hex::encode(hasher.finalize())
    }

    #[must_use]
    pub const fn claims_key(&self) -> Uuid {
        self.pid
    }

    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    /// Loads user seed data from a file and inserts it into the database.
    ///
    /// This method reads and deserialises user records from the specified
    /// seed file before delegating persistence to [`Self::seed`].
    ///
    /// It is primarily intended for development, testing, and environment
    /// bootstrapping where predefined user records need to be created.
    ///
    /// # Parameters
    ///
    /// - `db` - The database connection pool used to persist the records.
    /// - `file` - Path to the seed data file.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The seed file cannot be found or read.
    /// - The seed file contains invalid or malformed data.
    /// - User records cannot be inserted into the database.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let users = Self::load(file).await?;

        Self::seed(db, &users).await
    }

    /// Finds a user by their public identifier (PID).
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The user with the specified PID is not found.
    /// - The database query fails.
    pub async fn find_by_pid(db: &PgPool, pid: Uuid) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE pid = $1")
            .bind(pid)
            .fetch_one(db)
            .await
            .map_err(|e| ModelError::EntityNotFound)
    }

    /// Lists users with their assigned roles.
    ///
    /// When `role` is provided, only users assigned to that role are returned.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup fails or role aggregation cannot
    /// be decoded.
    pub async fn find_list_with_roles(
        db: &PgPool,
        role: Option<&str>,
    ) -> ModelResult<Vec<UserWithRoles>> {
        let rows = sqlx::query(
            r"
            SELECT
                users.id,
                users.pid,
                users.name,
                users.email,
                users.image,
                users.verified_at IS NOT NULL AS verified,
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object(
                            'id', roles.id,
                            'pid', roles.pid,
                            'name', roles.name,
                            'description', roles.description
                        )
                        ORDER BY roles.name
                    ) FILTER (WHERE roles.id IS NOT NULL),
                    '[]'::jsonb
                ) AS roles,
                users.created_at,
                users.updated_at
            FROM users
            LEFT JOIN users_roles ON users_roles.user_id = users.id
            LEFT JOIN roles ON roles.id = users_roles.role_id
            WHERE users.deleted_at IS NULL
                AND (
                    $1::TEXT IS NULL
                    OR EXISTS (
                        SELECT 1
                        FROM users_roles role_filter_users_roles
                        INNER JOIN roles role_filter_roles
                            ON role_filter_roles.id = role_filter_users_roles.role_id
                        WHERE role_filter_users_roles.user_id = users.id
                            AND role_filter_roles.name = LOWER(TRIM($1))
                    )
                )
            GROUP BY users.id
            ORDER BY users.created_at DESC, users.id DESC
        ",
        )
        .bind(role)
        .fetch_all(db)
        .await?;

        rows.into_iter()
            .map(|row| {
                let roles_json = row.try_get::<serde_json::Value, _>("roles")?;
                let roles = serde_json::from_value(roles_json).map_err(|err| {
                    sqlx::Error::ColumnDecode {
                        index: "roles".into(),
                        source: Box::new(err),
                    }
                })?;

                Ok(UserWithRoles {
                    id: row.try_get("id")?,
                    pid: row.try_get("pid")?,
                    name: row.try_get("name")?,
                    email: row.try_get("email")?,
                    image: row.try_get("image")?,
                    verified: row.try_get("verified")?,
                    roles,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()
            .map_err(Into::into)
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
    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    #[must_use]
    pub const fn verified_at(&self) -> Option<DateTime<FixedOffset>> {
        self.verified_at
    }

    #[must_use]
    pub const fn verification_token_hash(&self) -> Option<&String> {
        self.verification_token_hash.as_ref()
    }

    #[must_use]
    pub const fn verification_token_expires_at(&self) -> Option<DateTime<FixedOffset>> {
        self.verification_token_expires_at
    }

    #[must_use]
    pub const fn reset_token_hash(&self) -> Option<&String> {
        self.reset_token_hash.as_ref()
    }

    #[must_use]
    pub const fn reset_token_expires_at(&self) -> Option<DateTime<FixedOffset>> {
        self.reset_token_expires_at
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
    pub const fn image(&self) -> Option<&String> {
        self.image.as_ref()
    }
}

impl UserAccess {
    #[must_use]
    pub fn roles(&self) -> &[String] {
        &self.roles
    }

    #[must_use]
    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }
}

impl Seedable for User {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for user in data {
            let verification_token_hash = user
                .verification_token_hash
                .as_ref()
                .map(|token| Self::hash_text(token));

            sqlx::query(
                r"
                INSERT INTO users (
                    id,
                    pid,
                    email,
                    name,
                    password_hash,
                    image,
                    verified_at,
                    verification_token_hash,
                    verification_token_expires_at,
                    created_at,
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    email = EXCLUDED.email,
                    name = EXCLUDED.name,
                    password_hash = EXCLUDED.password_hash,
                    image = EXCLUDED.image,
                    verified_at = EXCLUDED.verified_at,
                    verification_token_hash = EXCLUDED.verification_token_hash,
                    verification_token_expires_at = EXCLUDED.verification_token_expires_at,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at
            ",
            )
            .bind(user.id)
            .bind(user.pid)
            .bind(user.email.as_str())
            .bind(user.name.as_str())
            .bind(user.password_hash.as_str())
            .bind(user.image.as_deref())
            .bind(user.verified_at)
            .bind(verification_token_hash.as_deref())
            .bind(user.verification_token_expires_at)
            .bind(user.created_at)
            .bind(user.updated_at)
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
