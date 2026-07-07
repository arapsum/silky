use std::borrow::Cow;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProduct<'a> {
    #[validate(range(min = 1, message = "Category ID must be a positive integer"))]
    category_id: i32,
    #[validate(custom(function = "validate_product_name"))]
    name: Cow<'a, str>,
    #[validate(length(max = 2000, message = "Description must be under 2000 characters"))]
    description: Option<Cow<'a, str>>,
    #[validate(nested)]
    options: Option<Vec<CreateProductOption>>,
    #[validate(nested)]
    pictures: Option<Vec<CreateProductPicture<'a>>>,
    #[validate(nested)]
    variants: Option<Vec<CreateProductVariant<'a>>>,
}

impl<'a> CreateProduct<'a> {
    #[must_use]
    pub const fn new(
        category_id: i32,
        name: Cow<'a, str>,
        description: Option<Cow<'a, str>>,
        options: Option<Vec<CreateProductOption>>,
        pictures: Option<Vec<CreateProductPicture<'a>>>,
        variants: Option<Vec<CreateProductVariant<'a>>>,
    ) -> Self {
        Self {
            category_id,
            name,
            description,
            options,
            pictures,
            variants,
        }
    }

    #[must_use]
    pub const fn category_id(&self) -> i32 {
        self.category_id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn description(&self) -> Option<&Cow<'a, str>> {
        self.description.as_ref()
    }

    #[must_use]
    pub fn options(&self) -> &[CreateProductOption] {
        self.options.as_deref().unwrap_or_default()
    }

    #[must_use]
    pub fn pictures(&self) -> &[CreateProductPicture<'a>] {
        self.pictures.as_deref().unwrap_or_default()
    }

    #[must_use]
    pub fn variants(&self) -> &[CreateProductVariant<'a>] {
        self.variants.as_deref().unwrap_or_default()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductOption {
    #[validate(range(min = 1, message = "Attribute ID must be a positive integer"))]
    attribute_id: i32,
    display_order: Option<i32>,
}

impl CreateProductOption {
    #[must_use]
    pub const fn new(attribute_id: i32, display_order: Option<i32>) -> Self {
        Self {
            attribute_id,
            display_order,
        }
    }

    #[must_use]
    pub const fn attribute_id(&self) -> i32 {
        self.attribute_id
    }

    #[must_use]
    pub const fn display_order(&self) -> Option<i32> {
        self.display_order
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductPicture<'a> {
    #[validate(url(message = "Invalid image URL"))]
    image_link: Cow<'a, str>,
    display_order: Option<i32>,
}

impl<'a> CreateProductPicture<'a> {
    #[must_use]
    pub const fn new(image_link: Cow<'a, str>, display_order: Option<i32>) -> Self {
        Self {
            image_link,
            display_order,
        }
    }

    #[must_use]
    pub fn image_link(&self) -> &str {
        &self.image_link
    }

    #[must_use]
    pub const fn display_order(&self) -> Option<i32> {
        self.display_order
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductVariant<'a> {
    #[validate(custom(function = "validate_sku"))]
    sku: Cow<'a, str>,
    #[validate(custom(function = "validate_price"))]
    price: Decimal,
    #[validate(range(min = 0, message = "Stock quantity cannot be negative"))]
    stock_quantity: i32,
    is_default: bool,
    #[validate(nested)]
    attribute_values: Option<Vec<CreateVariantAttributeValue>>,
    #[validate(nested)]
    pictures: Option<Vec<CreateProductPicture<'a>>>,
}

impl<'a> CreateProductVariant<'a> {
    #[must_use]
    pub const fn new(
        sku: Cow<'a, str>,
        price: Decimal,
        stock_quantity: i32,
        is_default: bool,
        attribute_values: Option<Vec<CreateVariantAttributeValue>>,
        pictures: Option<Vec<CreateProductPicture<'a>>>,
    ) -> Self {
        Self {
            sku,
            price,
            stock_quantity,
            is_default,
            attribute_values,
            pictures,
        }
    }

    #[must_use]
    pub fn sku(&self) -> &str {
        &self.sku
    }

    #[must_use]
    pub const fn price(&self) -> Decimal {
        self.price
    }

    #[must_use]
    pub const fn stock_quantity(&self) -> i32 {
        self.stock_quantity
    }

    #[must_use]
    pub const fn is_default(&self) -> bool {
        self.is_default
    }

    #[must_use]
    pub fn attribute_values(&self) -> &[CreateVariantAttributeValue] {
        self.attribute_values.as_deref().unwrap_or_default()
    }

    #[must_use]
    pub fn pictures(&self) -> &[CreateProductPicture<'a>] {
        self.pictures.as_deref().unwrap_or_default()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantAttributeValue {
    #[validate(range(min = 1, message = "Attribute ID must be a positive integer"))]
    attribute_id: i32,
    #[validate(range(min = 1, message = "Attribute value ID must be a positive integer"))]
    attribute_value_id: i32,
}

impl CreateVariantAttributeValue {
    #[must_use]
    pub const fn new(attribute_id: i32, attribute_value_id: i32) -> Self {
        Self {
            attribute_id,
            attribute_value_id,
        }
    }

    #[must_use]
    pub const fn attribute_id(&self) -> i32 {
        self.attribute_id
    }

    #[must_use]
    pub const fn attribute_value_id(&self) -> i32 {
        self.attribute_value_id
    }
}

fn validate_product_name(name: &str) -> Result<(), ValidationError> {
    const MIN_LENGTH: usize = 2;
    const MAX_LENGTH: usize = 255;

    let name = name.trim();
    let length = name.len();

    if name.is_empty() {
        return Err(
            ValidationError::new("empty_name").with_message(Cow::Borrowed("Name is required"))
        );
    }

    if length < MIN_LENGTH {
        return Err(ValidationError::new("short_name")
            .with_message(Cow::Borrowed("Name requires 2 characters")));
    }

    if length > MAX_LENGTH {
        return Err(ValidationError::new("long_name")
            .with_message(Cow::Borrowed("Name must be under 255 characters")));
    }

    Ok(())
}

fn validate_sku(sku: &str) -> Result<(), ValidationError> {
    let sku = sku.trim();

    if sku.is_empty() {
        return Err(
            ValidationError::new("empty_sku").with_message(Cow::Borrowed("SKU is required"))
        );
    }

    if sku.len() > 64 {
        return Err(ValidationError::new("long_sku")
            .with_message(Cow::Borrowed("SKU must be under 64 characters")));
    }

    Ok(())
}

fn validate_price(price: &Decimal) -> Result<(), ValidationError> {
    if price.is_sign_negative() {
        Err(ValidationError::new("negative_price")
            .with_message(Cow::Borrowed("Price must be greater than or equal to zero")))
    } else {
        Ok(())
    }
}
