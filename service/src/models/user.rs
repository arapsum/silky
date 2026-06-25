#![allow(unused)]
use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::schemas::{LoginUser, RegisterUser};

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

impl User {
    /// Creates a new [`User`] and stores it in the database.
    ///
    /// The provided password is hashed using Argon2 before being persisted.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Password hashing fails.
    /// - The user record cannot be inserted into the database.
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
            INSERT INTO users (name, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
        ",
        )
        .bind(params.name())
        .bind(params.email())
        .bind(password_hash)
        .fetch_one(&mut *txn)
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
                    verified_at,
                    verification_token_hash,
                    verification_token_expires_at,
                    created_at,
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    email = EXCLUDED.email,
                    name = EXCLUDED.name,
                    password_hash = EXCLUDED.password_hash,
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
