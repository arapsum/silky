use argon2::password_hash::Error as ArgonError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("entity already exists")]
    EntityAlreadyExists(String),
    #[error("entity not found")]
    EntityNotFound,
    #[error("File not found")]
    FileNotFound,
    #[error("Invalid claims key")]
    InvalidClaimsKey,
    #[error("Invalid credentials provided")]
    InvalidCredentials,
    #[error("Invalid verification token")]
    InvalidVerificationToken,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Password hashing error: {0}")]
    PasswordHash(ArgonError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeSaphyr(#[from] serde_saphyr::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("Only JSON and YAML file types supported")]
    UnsupportedFileType,
}

pub type ModelResult<T> = std::result::Result<T, ModelError>;

impl From<ArgonError> for ModelError {
    fn from(err: ArgonError) -> Self {
        match err {
            ArgonError::Password => Self::InvalidCredentials,
            other => Self::PasswordHash(other),
        }
    }
}

impl ModelError {
    #[must_use]
    pub fn response_body(&self) -> (StatusCode, String) {
        let (status, message) = match self {
            Self::EntityAlreadyExists(_) => {
                (StatusCode::CONFLICT, "Entity already exists".to_string())
            }
            Self::EntityNotFound => (StatusCode::NOT_FOUND, "Entity not found".to_string()),
            Self::InvalidClaimsKey => (StatusCode::UNAUTHORIZED, "Invalid claims key".to_string()),
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid email or password".to_string(),
            ),
            Self::InvalidVerificationToken => (
                StatusCode::UNAUTHORIZED,
                "Invalid verification token".to_string(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, message)
    }

    #[must_use]
    pub fn response(&self) -> Response {
        let (status, message) = self.response_body();
        (status, Json(json!({ "error": message }))).into_response()
    }
}
