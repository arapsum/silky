use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::User;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoginResponse {
    pub pid: Uuid,
    pub email: String,
    pub name: String,
    pub token: String,
    pub verified: bool,
}

impl LoginResponse {
    #[must_use]
    pub fn new(user: &User, token: &str) -> Self {
        Self {
            pid: user.pid(),
            email: user.email().to_string(),
            name: user.name().to_string(),
            token: token.to_string(),
            verified: user.verified_at().is_some(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthResponse {
    pub message: String,
}

impl AuthResponse {
    #[must_use]
    pub fn new<T: Into<String>>(message: T) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub pid: Uuid,
    pub email: String,
    pub name: String,
    pub verified: bool,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl UserResponse {
    #[must_use]
    pub fn new(user: &User) -> Self {
        Self {
            pid: user.pid(),
            email: user.email().to_string(),
            name: user.name().to_string(),
            verified: user.verified_at().is_some(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        }
    }
}
