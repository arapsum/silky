use argon2::password_hash::Error as ArgonError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::error::{DatabaseError, ErrorKind as SqlxErrorKind};
use uuid::Uuid;

use crate::error::ErrorResponse;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("Categories with products cannot be deleted")]
    CategoryHasProducts,
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
    #[error("Customer does not exist")]
    CustomerNotFound,
    #[error("The selected customer address does not exist")]
    CustomerAddressNotFound { field: &'static str },
    #[error("An order must contain at least one item")]
    OrderHasNoItems,
    #[error("Order item quantity must be at least one")]
    InvalidOrderItemQuantity,
    #[error("Each product variant may appear only once per order")]
    DuplicateOrderItem,
    #[error("One or more product variants do not exist")]
    ProductVariantUnavailable,
    #[error(
        "Insufficient stock for variant {variant_pid}: requested {requested}, available {available}"
    )]
    InsufficientStock {
        variant_pid: Uuid,
        requested: i32,
        available: i32,
    },
    #[error("Order items can only be changed while an order is pending")]
    OrderNotEditable,
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
        if let Some(order_error) = Self::from_order_constraint(error) {
            return Some(order_error);
        }

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

    fn from_order_constraint(error: &(dyn DatabaseError + 'static)) -> Option<Self> {
        match error.constraint()? {
            "orders_customer_id_fkey" => Some(Self::CustomerNotFound),
            "orders_billing_address_id_fkey" => Some(Self::CustomerAddressNotFound {
                field: "billingAddressPid",
            }),
            "orders_shipping_address_id_fkey" => Some(Self::CustomerAddressNotFound {
                field: "shippingAddressPid",
            }),
            "order_items_order_id_fkey" => {
                Some(Self::InvalidReference("Order does not exist.".to_string()))
            }
            "order_items_product_id_fkey" | "order_items_variant_id_fkey" => {
                Some(Self::ProductVariantUnavailable)
            }
            "orders_order_number_key" | "orders_pid_key" => Some(Self::EntityAlreadyExists(
                "Order identifier already exists.".to_string(),
            )),
            _ => None,
        }
    }

    fn constraint_message(error: &(dyn DatabaseError + 'static), fallback: &str) -> String {
        match error.constraint() {
            Some("attributes_name_key") => {
                "Product attribute with this name already exists.".to_string()
            }
            Some("attributes_name_check") => "Product attribute name cannot be empty.".to_string(),
            Some("tags_name_unique") => "Product tag with this name already exists.".to_string(),
            Some("tags_name_check") => {
                "Product tag name must contain between 1 and 64 characters.".to_string()
            }
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
            Some("orders_customer_name_check") => {
                "Order customer name cannot be empty.".to_string()
            }
            Some("orders_status_check") => "Order status is not supported.".to_string(),
            Some("orders_payment_status_check") => {
                "Order payment status is not supported.".to_string()
            }
            Some("orders_fulfillment_status_check") => {
                "Order fulfillment status is not supported.".to_string()
            }
            Some("orders_currency_check") => {
                "Order currency must be a three-letter uppercase code.".to_string()
            }
            Some("orders_subtotal_check") => "Order subtotal cannot be negative.".to_string(),
            Some("orders_discount_total_check") => {
                "Order discount total cannot be negative.".to_string()
            }
            Some("orders_shipping_total_check") => {
                "Order shipping total cannot be negative.".to_string()
            }
            Some("orders_tax_total_check") => {
                "Order tax total cannot be negative.".to_string()
            }
            Some("orders_grand_total_check") => {
                "Order grand total does not match its amount breakdown.".to_string()
            }
            Some("order_items_selected_options_check") => {
                "Order item options must be an object.".to_string()
            }
            Some("order_items_quantity_check") => {
                "Order item quantity must be greater than zero.".to_string()
            }
            Some("order_items_unit_price_check") => {
                "Order item unit price cannot be negative.".to_string()
            }
            Some("order_items_discount_total_check") => {
                "Order item discount total cannot be negative.".to_string()
            }
            Some("order_items_tax_total_check") => {
                "Order item tax total cannot be negative.".to_string()
            }
            Some("order_items_line_total_check") => {
                "Order item total does not match its amount breakdown.".to_string()
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
            Self::CategoryHasProducts => (StatusCode::CONFLICT, self.to_string()),
            Self::EntityAlreadyExists(message) => (StatusCode::CONFLICT, message.clone()),
            Self::EntityNotFound => (StatusCode::NOT_FOUND, "Entity not found".to_string()),
            Self::InvalidInput(message) => (StatusCode::BAD_REQUEST, message.clone()),
            Self::InvalidReference(message) => (StatusCode::NOT_FOUND, message.clone()),
            Self::CustomerNotFound => {
                (StatusCode::NOT_FOUND, "Customer does not exist".to_string())
            }
            Self::CustomerAddressNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "The selected customer address does not exist".to_string(),
            ),
            Self::OrderHasNoItems => (
                StatusCode::BAD_REQUEST,
                "An order must contain at least one item".to_string(),
            ),
            Self::InvalidOrderItemQuantity => (
                StatusCode::BAD_REQUEST,
                "Order item quantity must be at least one".to_string(),
            ),
            Self::DuplicateOrderItem => (
                StatusCode::BAD_REQUEST,
                "Each product variant may appear only once per order".to_string(),
            ),
            Self::ProductVariantUnavailable => (
                StatusCode::NOT_FOUND,
                "One or more product variants do not exist".to_string(),
            ),
            Self::InsufficientStock { .. } | Self::OrderNotEditable => {
                (StatusCode::CONFLICT, self.to_string())
            }
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

    /// Returns a stable machine-readable error code for API consumers.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::CategoryHasProducts => "category_has_products",
            Self::CustomerNotFound => "customer_not_found",
            Self::CustomerAddressNotFound { .. } => "customer_address_not_found",
            Self::OrderHasNoItems => "order_items_required",
            Self::InvalidOrderItemQuantity => "invalid_order_item_quantity",
            Self::DuplicateOrderItem => "duplicate_order_item",
            Self::ProductVariantUnavailable => "product_variant_unavailable",
            Self::InsufficientStock { .. } => "insufficient_stock",
            Self::OrderNotEditable => "order_not_editable",
            Self::EntityAlreadyExists(_) => "entity_already_exists",
            Self::EntityNotFound => "entity_not_found",
            Self::InvalidInput(_) => "invalid_input",
            Self::InvalidReference(_) => "invalid_reference",
            Self::InvalidClaimsKey => "invalid_claims_key",
            Self::InvalidCredentials => "invalid_credentials",
            Self::InvalidResetToken => "invalid_reset_token",
            Self::InvalidVerificationToken => "invalid_verification_token",
            Self::FileNotFound => "file_not_found",
            Self::UnsupportedFileType => "unsupported_file_type",
            Self::IO(_)
            | Self::PasswordHash(_)
            | Self::SerdeJson(_)
            | Self::SerdeSaphyr(_)
            | Self::Sqlx(_) => "internal_error",
        }
    }

    /// Returns the request field most closely associated with the error.
    #[must_use]
    pub const fn field(&self) -> Option<&'static str> {
        match self {
            Self::CustomerNotFound => Some("customerPid"),
            Self::CustomerAddressNotFound { field } => Some(field),
            Self::OrderHasNoItems | Self::DuplicateOrderItem => Some("items"),
            Self::InvalidOrderItemQuantity => Some("items.quantity"),
            Self::ProductVariantUnavailable | Self::InsufficientStock { .. } => {
                Some("items.variantPid")
            }
            Self::OrderNotEditable => Some("status"),
            _ => None,
        }
    }

    /// Returns structured context that can help a client resolve the error.
    #[must_use]
    pub fn details(&self) -> Option<serde_json::Value> {
        match self {
            Self::InsufficientStock {
                variant_pid,
                requested,
                available,
            } => Some(json!({
                "variantPid": variant_pid,
                "requested": requested,
                "available": available,
            })),
            _ => None,
        }
    }

    #[must_use]
    pub fn response(&self) -> Response {
        let (status, message) = self.response_body();
        let body = ErrorResponse {
            error: message,
            code: self.code(),
            field: self.field(),
            details: self.details(),
        };
        (status, Json(body)).into_response()
    }
}
