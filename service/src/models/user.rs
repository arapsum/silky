use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    #[must_use]
    pub const fn claims_key(&self) -> Uuid {
        self.pid
    }
}
