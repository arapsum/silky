use std::sync::{Arc, OnceLock};

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use redis::{ExistenceCheck, SetExpiry, SetOptions};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    Config, Error,
    config::{AuthConfig, JwtConfig, RedisConfig},
    workers::MailQueue,
};

pub type AppState = Arc<AppContext>;

#[derive(Clone)]
pub struct AppContext {
    auth: AuthContext,
    config: Config,
    db: PgPool,
    queue: Arc<OnceLock<MailQueue>>,
    redis: redis::Client,
}

impl AppContext {
    const REFRESH_TOKEN_KEY_PREFIX: &str = "auth:refresh";

    /// Initialises application services and infrastructure.
    ///
    /// This method performs application startup tasks, including configuring
    /// logging and executing any database initialisation required by the current
    /// configuration.
    ///
    /// Startup operations are performed in the following order:
    ///
    /// 1. Configure and initialise the application's logger.
    /// 2. Initialise the database and run any configured migration actions.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The logger cannot be initialised.
    /// - Database initialisation fails.
    /// - Any configured migration operation fails.
    pub async fn init(&self) -> crate::Result<()> {
        self.config.logger().setup()?;
        self.config.database().init().await?;

        Ok(())
    }

    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub const fn db(&self) -> &PgPool {
        &self.db
    }

    #[must_use]
    pub const fn auth(&self) -> &AuthContext {
        &self.auth
    }

    #[must_use]
    pub const fn queue(&self) -> &Arc<OnceLock<MailQueue>> {
        &self.queue
    }

    #[must_use]
    pub const fn redis(&self) -> &redis::Client {
        &self.redis
    }

    /// Stores a newly issued refresh token identifier until the token expires.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The token is already expired.
    /// - Redis cannot store the token identifier.
    pub async fn store_refresh_token(&self, claims: &Claims) -> crate::Result<()> {
        let ttl = claims.remaining_ttl()?;
        let mut conn = self.redis().get_multiplexed_async_connection().await?;
        let key = Self::refresh_token_key(claims.id());
        let options = SetOptions::default()
            .conditional_set(ExistenceCheck::NX)
            .with_expiration(SetExpiry::EX(ttl));

        let stored: Option<String> = redis::cmd("SET")
            .arg(&key)
            .arg(claims.sub())
            .arg(options)
            .query_async(&mut conn)
            .await?;

        if stored.is_some() {
            Ok(())
        } else {
            Err(Error::InvalidToken.into())
        }
    }

    /// Consumes a refresh token identifier, preventing any later reuse.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The token is expired.
    /// - The token identifier is missing from Redis.
    /// - The token identifier does not belong to the expected subject.
    /// - Redis cannot consume the token identifier.
    pub async fn consume_refresh_token(&self, claims: &Claims) -> crate::Result<()> {
        claims.remaining_ttl()?;

        let mut conn = self.redis().get_multiplexed_async_connection().await?;
        let key = Self::refresh_token_key(claims.id());
        let stored_sub: Option<String> = redis::cmd("GETDEL")
            .arg(&key)
            .query_async(&mut conn)
            .await?;

        match stored_sub {
            Some(sub) if sub == claims.sub() => Ok(()),
            _ => Err(Error::InvalidToken.into()),
        }
    }

    /// Removes a refresh token identifier from Redis if it is still present.
    ///
    /// # Errors
    ///
    /// This function will return an error if Redis cannot delete the token
    /// identifier.
    pub async fn revoke_refresh_token(&self, claims: &Claims) -> crate::Result<()> {
        let mut conn = self.redis().get_multiplexed_async_connection().await?;
        let key = Self::refresh_token_key(claims.id());

        redis::cmd("DEL")
            .arg(&key)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }

    pub fn set_queue(&self, queue: MailQueue) {
        self.queue.get_or_init(|| queue);
    }

    fn refresh_token_key(token_id: &str) -> String {
        format!("{}:{token_id}", Self::REFRESH_TOKEN_KEY_PREFIX)
    }
}

impl TryFrom<&Config> for AppContext {
    type Error = crate::Report;

    fn try_from(cfg: &Config) -> Result<Self, Self::Error> {
        Ok(Self {
            auth: cfg.auth().try_into()?,
            db: cfg.database().pool()?,
            config: cfg.clone(),
            queue: Arc::new(OnceLock::new()),
            redis: RedisConfig::connection(cfg.redis())?,
        })
    }
}

#[derive(Clone)]
pub struct AuthContext {
    access: JwtContext,
    refresh: JwtContext,
}

impl AuthContext {
    #[must_use]
    pub const fn access(&self) -> &JwtContext {
        &self.access
    }

    #[must_use]
    pub const fn refresh(&self) -> &JwtContext {
        &self.refresh
    }

    /// Generates a new access token for the given subject.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The token could not be encoded.
    pub fn generate_access_token(&self, sub: &str) -> crate::Result<String> {
        self.access.generate_token(sub)
    }

    /// Generates a new refresh token for the given subject.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The token could not be encoded.
    pub fn generate_refresh_token(&self, sub: &str) -> crate::Result<String> {
        self.refresh.generate_token(sub)
    }

    /// Verifies the given access token.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The token could not be decoded.
    /// - The token is invalid.
    pub fn verify_access_token(&self, token: &str) -> crate::Result<Claims> {
        self.access.verify_token(token)
    }

    /// Verifies the given refresh token.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The token could not be decoded.
    /// - The token is invalid.
    pub fn verify_refresh_token(&self, token: &str) -> crate::Result<Claims> {
        self.refresh.verify_token(token)
    }
}

impl TryFrom<&AuthConfig> for AuthContext {
    type Error = crate::Report;

    fn try_from(cfg: &AuthConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            access: cfg.access().try_into()?,
            refresh: cfg.refresh().try_into()?,
        })
    }
}

#[derive(Clone)]
pub struct JwtContext {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expires_in: i64,
}

impl JwtContext {
    #[must_use]
    pub const fn encoding_key(&self) -> &EncodingKey {
        &self.encoding_key
    }

    #[must_use]
    pub const fn decoding_key(&self) -> &DecodingKey {
        &self.decoding_key
    }

    #[must_use]
    pub const fn expires_in(&self) -> i64 {
        self.expires_in
    }

    /// Generates a JWT token for the given subject.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The encoding key is invalid.
    /// - The claims could not be encoded.
    pub fn generate_token(&self, sub: &str) -> crate::Result<String> {
        let (token, _) = self.generate_token_with_claims(sub)?;

        Ok(token)
    }

    /// Generates a JWT token and returns the claims used to encode it.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The encoding key is invalid.
    /// - The claims could not be encoded.
    pub fn generate_token_with_claims(&self, sub: &str) -> crate::Result<(String, Claims)> {
        let claims = Claims::new(sub, self.expires_in());
        let header = Header::new(Algorithm::RS256);

        let token = jsonwebtoken::encode(&header, &claims, self.encoding_key())?;

        Ok((token, claims))
    }

    /// Verifies a JWT token and returns the claims if valid.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The decoding key is invalid.
    /// - The token could not be decoded.
    pub fn verify_token(&self, token: &str) -> crate::Result<Claims> {
        let claims = jsonwebtoken::decode::<Claims>(
            token,
            self.decoding_key(),
            &Validation::new(Algorithm::RS256),
        )
        .map_err(Error::from)?;
        Ok(claims.claims)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Claims {
    id: String,
    sub: String,
    exp: i64,
    iat: i64,
    nbf: i64,
}

impl Claims {
    #[must_use]
    pub fn new(sub: &str, expiry: i64) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::seconds(expiry)).timestamp();

        Self {
            id: Uuid::new_v4().to_string(),
            sub: sub.to_string(),
            exp,
            iat: now.timestamp(),
            nbf: now.timestamp(),
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn sub(&self) -> &str {
        &self.sub
    }

    #[must_use]
    pub const fn exp(&self) -> i64 {
        self.exp
    }

    #[must_use]
    pub const fn iat(&self) -> i64 {
        self.iat
    }

    #[must_use]
    pub const fn nbf(&self) -> i64 {
        self.nbf
    }

    fn remaining_ttl(&self) -> crate::Result<u64> {
        let ttl = self.exp() - Utc::now().timestamp();

        if ttl <= 0 {
            return Err(Error::ExpiredSession.into());
        }

        u64::try_from(ttl).map_err(Into::into)
    }
}

impl TryFrom<&JwtConfig> for JwtContext {
    type Error = crate::Report;

    fn try_from(config: &JwtConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            encoding_key: config.encoding_key()?,
            decoding_key: config.decoding_key()?,
            expires_in: config.maxage(),
        })
    }
}
