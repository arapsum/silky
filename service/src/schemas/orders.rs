use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

fn validate_currency(value: &str) -> Result<(), ValidationError> {
    if value.len() == 3
        && value
            .chars()
            .all(|character| character.is_ascii_alphabetic())
    {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_currency"))
    }
}

fn validate_country_code(value: &str) -> Result<(), ValidationError> {
    if value.len() == 2
        && value
            .chars()
            .all(|character| character.is_ascii_alphabetic())
    {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_country_code"))
    }
}

fn validate_non_negative(value: &Decimal) -> Result<(), ValidationError> {
    if value.is_sign_negative() {
        Err(ValidationError::new("negative_amount"))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NewAddress {
    customer_pid: Uuid,
    #[validate(custom(function = "validate_address_type"))]
    address_type: String,
    #[validate(length(max = 120))]
    label: Option<String>,
    #[validate(length(min = 1, max = 255))]
    recipient_name: String,
    #[validate(length(max = 255))]
    company: Option<String>,
    #[validate(length(min = 1, max = 255))]
    line_one: String,
    #[validate(length(max = 255))]
    line_two: Option<String>,
    #[validate(length(min = 1, max = 120))]
    city: String,
    #[validate(length(max = 120))]
    region: Option<String>,
    #[validate(length(max = 32))]
    postal_code: Option<String>,
    #[validate(custom(function = "validate_country_code"))]
    country_code: String,
    #[validate(email)]
    email: Option<String>,
    #[validate(length(max = 32))]
    phone: Option<String>,
    is_default: bool,
}

fn validate_address_type(value: &str) -> Result<(), ValidationError> {
    if matches!(value, "billing" | "shipping" | "other") {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_address_type"))
    }
}

impl NewAddress {
    #[must_use]
    pub const fn customer_pid(&self) -> Uuid {
        self.customer_pid
    }
    #[must_use]
    pub fn address_type(&self) -> &str {
        &self.address_type
    }
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    #[must_use]
    pub fn recipient_name(&self) -> &str {
        &self.recipient_name
    }
    #[must_use]
    pub fn company(&self) -> Option<&str> {
        self.company.as_deref()
    }
    #[must_use]
    pub fn line_one(&self) -> &str {
        &self.line_one
    }
    #[must_use]
    pub fn line_two(&self) -> Option<&str> {
        self.line_two.as_deref()
    }
    #[must_use]
    pub fn city(&self) -> &str {
        &self.city
    }
    #[must_use]
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }
    #[must_use]
    pub fn postal_code(&self) -> Option<&str> {
        self.postal_code.as_deref()
    }
    #[must_use]
    pub fn country_code(&self) -> &str {
        &self.country_code
    }
    #[must_use]
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }
    #[must_use]
    pub fn phone(&self) -> Option<&str> {
        self.phone.as_deref()
    }
    #[must_use]
    pub const fn is_default(&self) -> bool {
        self.is_default
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NewOrder {
    customer_pid: Uuid,
    billing_address_pid: Option<Uuid>,
    shipping_address_pid: Option<Uuid>,
    #[validate(custom(function = "validate_currency"))]
    currency: String,
    #[validate(custom(function = "validate_non_negative"))]
    subtotal: Decimal,
    #[validate(custom(function = "validate_non_negative"))]
    discount_total: Decimal,
    #[validate(custom(function = "validate_non_negative"))]
    shipping_total: Decimal,
    #[validate(custom(function = "validate_non_negative"))]
    tax_total: Decimal,
    #[validate(custom(function = "validate_non_negative"))]
    grand_total: Decimal,
    customer_note: Option<String>,
    staff_note: Option<String>,
}

impl NewOrder {
    #[must_use]
    pub const fn customer_pid(&self) -> Uuid {
        self.customer_pid
    }
    #[must_use]
    pub const fn billing_address_pid(&self) -> Option<Uuid> {
        self.billing_address_pid
    }
    #[must_use]
    pub const fn shipping_address_pid(&self) -> Option<Uuid> {
        self.shipping_address_pid
    }
    #[must_use]
    pub fn currency(&self) -> &str {
        &self.currency
    }
    #[must_use]
    pub const fn subtotal(&self) -> Decimal {
        self.subtotal
    }
    #[must_use]
    pub const fn discount_total(&self) -> Decimal {
        self.discount_total
    }
    #[must_use]
    pub const fn shipping_total(&self) -> Decimal {
        self.shipping_total
    }
    #[must_use]
    pub const fn tax_total(&self) -> Decimal {
        self.tax_total
    }
    #[must_use]
    pub const fn grand_total(&self) -> Decimal {
        self.grand_total
    }
    #[must_use]
    pub fn customer_note(&self) -> Option<&str> {
        self.customer_note.as_deref()
    }
    #[must_use]
    pub fn staff_note(&self) -> Option<&str> {
        self.staff_note.as_deref()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderDetail {
    order_pid: Uuid,
    variant_pid: Uuid,
    #[validate(range(min = 1))]
    quantity: i32,
    #[validate(custom(function = "validate_non_negative"))]
    discount_total: Decimal,
    #[validate(custom(function = "validate_non_negative"))]
    tax_total: Decimal,
}

impl NewOrderDetail {
    #[must_use]
    pub const fn order_pid(&self) -> Uuid {
        self.order_pid
    }
    #[must_use]
    pub const fn variant_pid(&self) -> Uuid {
        self.variant_pid
    }
    #[must_use]
    pub const fn quantity(&self) -> i32 {
        self.quantity
    }
    #[must_use]
    pub const fn discount_total(&self) -> Decimal {
        self.discount_total
    }
    #[must_use]
    pub const fn tax_total(&self) -> Decimal {
        self.tax_total
    }
}
