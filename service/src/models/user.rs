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
    /// An email verification token is also generated and stored as a hash.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Password hashing fails.
    /// - Verification token hashing fails.
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
        let verification_token_hash = Self::hash_password(Uuid::new_v4().to_string().as_str())?;
        let now = Utc::now();

        let user = sqlx::query_as::<_, Self>(r"
            INSERT INTO users (name, email, password_hash, verification_token_hash, verification_token_expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
        ")
        .bind(params.username())
        .bind(params.email())
        .bind(password_hash)
        .bind(verification_token_hash)
        .bind(now + Duration::hours(24))
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(user)
    }

    fn hash_password(password: &str) -> ModelResult<String> {
        let argon: Argon2<'_> = Argon2::default();
        let salt: SaltString = SaltString::generate(&mut OsRng);

        Ok(argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(ModelError::PasswordHash)?
            .to_string())
    }

    fn verify_password(&self, plain_password: &str) -> ModelResult<()> {
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

    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let users = Self::load(file).await?;

        Self::seed(db, &users).await
    }
}

impl Seedable for User {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        let now = Utc::now();
        let expires = now + Duration::hours(24);

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
            .bind(expires)
            .bind(user.created_at)
            .bind(user.updated_at)
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
