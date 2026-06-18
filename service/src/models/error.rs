use argon2::password_hash::Error as ArgonError;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("entity already exists")]
    EntityAlreadyExists,
    #[error("entity not found")]
    EntityNotFound,
    #[error("Invalid claims key")]
    InvalidClaimsKey,
    #[error("Invalid credentials provided")]
    InvalidCredentials,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Password hashing error: {0}")]
    PasswordHash(ArgonError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
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
