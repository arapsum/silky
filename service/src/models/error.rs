use argon2::password_hash::Error as ArgonError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::error::{DatabaseError, ErrorKind as SqlxErrorKind};

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("entity already exists")]
    EntityAlreadyExists(String),
    #[error("entity not found")]
    EntityNotFound,
    #[error("File not found")]
    FileNotFound,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Invalid reference: {0}")]
    InvalidReference(String),
    #[error("Invalid claims key")]
    InvalidClaimsKey,
    #[error("Invalid credentials provided")]
    InvalidCredentials,
    #[error("Invalid reset token")]
    InvalidResetToken,
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
    Sqlx(sqlx::Error),
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

impl From<sqlx::Error> for ModelError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::EntityNotFound,
            sqlx::Error::Database(db_err) => Self::from_database_error(db_err.as_ref())
                .unwrap_or_else(|| Self::Sqlx(sqlx::Error::Database(db_err))),
            other => Self::Sqlx(other),
        }
    }
}

impl ModelError {
    fn from_database_error(error: &(dyn DatabaseError + 'static)) -> Option<Self> {
        match error.kind() {
            SqlxErrorKind::UniqueViolation => Some(Self::EntityAlreadyExists(
                Self::constraint_message(error, "A record with the supplied value already exists."),
            )),
            SqlxErrorKind::ForeignKeyViolation => Some(Self::InvalidReference(
                Self::constraint_message(error, "A referenced record does not exist."),
            )),
            SqlxErrorKind::CheckViolation => Some(Self::InvalidInput(Self::constraint_message(
                error,
                "One or more supplied values failed validation.",
            ))),
            SqlxErrorKind::NotNullViolation => Some(Self::InvalidInput(Self::constraint_message(
                error,
                "A required value is missing.",
            ))),
            _ => None,
        }
    }

    fn constraint_message(error: &(dyn DatabaseError + 'static), fallback: &str) -> String {
        match error.constraint() {
            Some("attributes_name_key") => {
                "Product attribute with this name already exists.".to_string()
            }
            Some("attributes_name_check") => "Product attribute name cannot be empty.".to_string(),
            Some("attribute_values_attribute_id_value_key") => {
                "Attribute already has this value.".to_string()
            }
            Some("attribute_values_attribute_id_fkey" | "product_options_attribute_id_fkey") => {
                "Attribute does not exist.".to_string()
            }
            Some("product_options_product_id_attribute_id_key") => {
                "Product already has this option attribute.".to_string()
            }
            Some(
                "pictures_product_id_fkey"
                | "product_options_product_id_fkey"
                | "product_variants_product_id_fkey",
            ) => "Product does not exist.".to_string(),
            Some("products_category_id_fkey") => "Category does not exist.".to_string(),
            Some("products_name_key") => "Product with this name already exists.".to_string(),
            Some("products_name_check") => "Product name cannot be empty.".to_string(),
            Some("pictures_variant_id_fkey") => "Product variant does not exist.".to_string(),
            Some("variant_attribute_values_variant_id_attribute_id_key") => {
                "Variant already has a value for this attribute.".to_string()
            }
            Some("variant_attribute_values_variant_id_fkey") => {
                "Product variant does not exist.".to_string()
            }
            Some("product_variants_sku_key") => {
                "Product variant with this SKU already exists.".to_string()
            }
            Some("one_default_variant_per_product") => {
                "Product already has a default variant.".to_string()
            }
            Some("product_variants_price_check") => {
                "Product variant price must be greater than or equal to zero.".to_string()
            }
            Some("product_variants_stock_quantity_check") => {
                "Product variant stock quantity must be greater than or equal to zero.".to_string()
            }
            Some("variant_attribute_values_attribute_value_id_attribute_id_fkey") => {
                "Attribute value does not exist for the selected attribute. Check that both the attribute and attribute value exist and belong together.".to_string()
            }
            Some(constraint) => format!("{fallback} Constraint: {constraint}."),
            None => fallback.to_string(),
        }
    }

    #[must_use]
    pub fn response_body(&self) -> (StatusCode, String) {
        let (status, message) = match self {
            Self::EntityAlreadyExists(message) => (StatusCode::CONFLICT, message.clone()),
            Self::EntityNotFound => (StatusCode::NOT_FOUND, "Entity not found".to_string()),
            Self::InvalidInput(message) => (StatusCode::BAD_REQUEST, message.clone()),
            Self::InvalidReference(message) => (StatusCode::NOT_FOUND, message.clone()),
            Self::InvalidClaimsKey => (StatusCode::UNAUTHORIZED, "Invalid claims key".to_string()),
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid email or password".to_string(),
            ),
            Self::InvalidResetToken => (StatusCode::FORBIDDEN, "Invalid reset token".to_string()),
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
