use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

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

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NewAddress {
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
    #[validate(length(min = 1, max = 100), nested)]
    items: Vec<NewOrderItem>,
    customer_note: Option<String>,
    staff_note: Option<String>,
}

/// Customer-controlled input for starting a hosted Checkout flow.
///
/// Customer identity is intentionally absent and must be derived from the
/// authenticated access token by the controller. Staff-only notes are also not
/// accepted on this public customer operation.
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckoutOrder {
    checkout_key: Uuid,
    billing_address_pid: Option<Uuid>,
    shipping_address_pid: Uuid,
    #[validate(length(min = 1, max = 100), nested)]
    items: Vec<NewOrderItem>,
    #[validate(length(max = 2_000))]
    customer_note: Option<String>,
}

impl CheckoutOrder {
    #[must_use]
    pub const fn checkout_key(&self) -> Uuid {
        self.checkout_key
    }

    #[must_use]
    pub const fn billing_address_pid(&self) -> Option<Uuid> {
        self.billing_address_pid
    }

    #[must_use]
    pub const fn shipping_address_pid(&self) -> Uuid {
        self.shipping_address_pid
    }

    #[must_use]
    pub fn items(&self) -> &[NewOrderItem] {
        &self.items
    }

    #[must_use]
    pub fn customer_note(&self) -> Option<&str> {
        self.customer_note.as_deref()
    }
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
    pub fn items(&self) -> &[NewOrderItem] {
        &self.items
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
pub struct NewOrderItem {
    variant_pid: Uuid,
    #[validate(range(min = 1))]
    quantity: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateOrder {
    #[validate(length(min = 1, max = 32))]
    status: Option<String>,
    #[validate(length(min = 1, max = 32))]
    fulfillment_status: Option<String>,
    staff_note: Option<String>,
}

impl UpdateOrder {
    #[must_use]
    pub fn status(&self) -> Option<&str> {
        self.status
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
    }
    #[must_use]
    pub fn fulfillment_status(&self) -> Option<&str> {
        self.fulfillment_status
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
    }
    #[must_use]
    pub fn staff_note(&self) -> Option<&str> {
        self.staff_note.as_deref()
    }
}

impl NewOrderItem {
    #[must_use]
    pub const fn variant_pid(&self) -> Uuid {
        self.variant_pid
    }
    #[must_use]
    pub const fn quantity(&self) -> i32 {
        self.quantity
    }
}
