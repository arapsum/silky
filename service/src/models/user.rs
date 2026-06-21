#![allow(unused)]
use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
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
    verified_at: Option<DateTime<Utc>>,
    verification_token_hash: Option<String>,
    verification_token_expires_at: Option<DateTime<Utc>>,
    // password reset
    reset_token_hash: Option<String>,
    reset_token_expires_at: Option<DateTime<Utc>>,
    // Dates
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
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

        let password_hash = Self::hash_text(params.password())?;

        let user = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO users (name, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
        ",
        )
        .bind(params.username())
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
        db: &C,
        expires_at: i64,
    ) -> ModelResult<Self>
    where
        for<'a> &'a C: Executor<'e, Database = Postgres>,
    {
        self.verification_token_hash = Some(Self::hash_text(Uuid::new_v4().to_string().as_str())?);
        self.verification_token_expires_at = Some(Utc::now() + Duration::seconds(expires_at));

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

    /// Finds a user by their claims key.
    ///
    /// # Errors
    ///
    /// This function will return an error:
    /// - `InvalidClaimsKey` if the claims key is not a valid UUID.
    /// - `EntityNotFound` if no user is found with the given claims key.
    /// - `Sqlx` if there is a database error.
    pub async fn find_by_claims_key<'e, C>(db: &C, claims_key: &str) -> ModelResult<Self>
    where
        for<'a> &'a C: Executor<'e, Database = Postgres>,
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

    fn hash_text(password: &str) -> ModelResult<String> {
        let argon: Argon2<'_> = Argon2::default();
        let salt: SaltString = SaltString::generate(&mut OsRng);

        Ok(argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(ModelError::PasswordHash)?
            .to_string())
    }

    fn verify_hash(&self, plain_password: &str) -> ModelResult<()> {
        let parded_hash = PasswordHash::new(&self.password_hash)?;

        Argon2::default().verify_password(plain_password.as_bytes(), &parded_hash)?;

        Ok(())
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
}

impl Seedable for User {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for user in data {
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
            ",
            )
            .bind(user.id)
            .bind(user.pid)
            .bind(user.email.as_str())
            .bind(user.name.as_str())
            .bind(user.password_hash.as_str())
            .bind(user.verified_at)
            .bind(user.verification_token_hash.as_deref())
            .bind(user.verification_token_expires_at)
            .bind(user.created_at)
            .bind(user.updated_at)
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
