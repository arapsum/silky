use std::{borrow::Cow, collections::BTreeMap};

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StockStatus {
    InStock,
    OutOfStock,
}

impl StockStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InStock => "inStock",
            Self::OutOfStock => "outOfStock",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ProductListQuery {
    #[validate(range(min = 1, message = "Limit must be a positive integer"))]
    limit: Option<i64>,
    #[validate(range(min = 1, message = "Page must be a positive integer"))]
    page: Option<i64>,
    search: Option<String>,
    name: Option<String>,
    #[validate(range(min = 1, message = "Category ID must be a positive integer"))]
    category_id: Option<i32>,
    category_slug: Option<String>,
    sku: Option<String>,
    #[validate(custom(function = "validate_optional_price"))]
    min_price: Option<Decimal>,
    #[validate(custom(function = "validate_optional_price"))]
    max_price: Option<Decimal>,
    stock_status: Option<StockStatus>,
    include_deleted: Option<bool>,
}

impl ProductListQuery {
    #[must_use]
    pub const fn limit(&self) -> Option<i64> {
        self.limit
    }

    #[must_use]
    pub const fn page(&self) -> Option<i64> {
        self.page
    }

    #[must_use]
    pub fn search(&self) -> Option<&str> {
        normalized_optional_str(self.search.as_deref())
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        normalized_optional_str(self.name.as_deref())
    }

    #[must_use]
    pub const fn category_id(&self) -> Option<i32> {
        self.category_id
    }

    #[must_use]
    pub fn category_slug(&self) -> Option<&str> {
        normalized_optional_str(self.category_slug.as_deref())
    }

    #[must_use]
    pub fn sku(&self) -> Option<&str> {
        normalized_optional_str(self.sku.as_deref())
    }

    #[must_use]
    pub const fn min_price(&self) -> Option<Decimal> {
        self.min_price
    }

    #[must_use]
    pub const fn max_price(&self) -> Option<Decimal> {
        self.max_price
    }

    #[must_use]
    pub const fn stock_status(&self) -> Option<StockStatus> {
        self.stock_status
    }

    #[must_use]
    pub const fn include_deleted(&self) -> bool {
        matches!(self.include_deleted, Some(true))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProduct<'a> {
    #[validate(range(min = 1, message = "Category ID must be a positive integer"))]
    category_id: i32,
    #[validate(custom(function = "validate_product_name"))]
    name: Cow<'a, str>,
    #[validate(length(max = 2000, message = "Description must be under 2000 characters"))]
    description: Option<Cow<'a, str>>,
    #[serde(default)]
    information: Option<BTreeMap<String, String>>,
    #[serde(default)]
    tag_pids: Vec<Uuid>,
    #[validate(nested)]
    pictures: Option<Vec<CreateProductPicture<'a>>>,
    #[validate(nested)]
    variants: Option<Vec<CreateProductVariant<'a>>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductTag {
    #[validate(length(
        min = 1,
        max = 64,
        message = "Tag name must contain between 1 and 64 characters"
    ))]
    name: String,
}

impl CreateProductTag {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProduct {
    #[validate(range(min = 1, message = "Category ID must be a positive integer"))]
    category_id: Option<i32>,
    #[validate(custom(function = "validate_optional_product_name"))]
    name: Option<String>,
    #[expect(
        clippy::option_option,
        reason = "PATCH needs to distinguish omitted description from explicit null"
    )]
    #[serde(default, deserialize_with = "deserialize_nullable_description")]
    description: Option<Option<String>>,
    #[serde(default)]
    information: Option<BTreeMap<String, String>>,
}

impl UpdateProduct {
    #[must_use]
    pub const fn category_id(&self) -> Option<i32> {
        self.category_id
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[must_use]
    pub const fn description(&self) -> Option<&Option<String>> {
        self.description.as_ref()
    }

    #[must_use]
    pub const fn information(&self) -> Option<&BTreeMap<String, String>> {
        self.information.as_ref()
    }
}

impl<'a> CreateProduct<'a> {
    #[must_use]
    pub const fn new(
        category_id: i32,
        name: Cow<'a, str>,
        description: Option<Cow<'a, str>>,
        pictures: Option<Vec<CreateProductPicture<'a>>>,
        variants: Option<Vec<CreateProductVariant<'a>>>,
    ) -> Self {
        Self {
            category_id,
            name,
            description,
            information: None,
            tag_pids: Vec::new(),
            pictures,
            variants,
        }
    }

    #[must_use]
    pub fn with_information(mut self, information: BTreeMap<String, String>) -> Self {
        self.information = Some(information);
        self
    }

    #[must_use]
    pub fn with_tag_pids(mut self, tag_pids: Vec<Uuid>) -> Self {
        self.tag_pids = tag_pids;
        self
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
    pub const fn information(&self) -> Option<&BTreeMap<String, String>> {
        self.information.as_ref()
    }

    #[must_use]
    pub fn tag_pids(&self) -> &[Uuid] {
        &self.tag_pids
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
pub struct CreateProductPicture<'a> {
    #[validate(url(message = "Invalid image URL"))]
    image_link: Cow<'a, str>,
    display_order: Option<i32>,
    #[serde(default)]
    media_asset_pid: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductPicture {
    display_order: Option<i32>,
}

impl UpdateProductPicture {
    #[must_use]
    pub const fn display_order(&self) -> Option<i32> {
        self.display_order
    }
}

impl<'a> CreateProductPicture<'a> {
    #[must_use]
    pub const fn new(image_link: Cow<'a, str>, display_order: Option<i32>) -> Self {
        Self {
            image_link,
            display_order,
            media_asset_pid: None,
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

    #[must_use]
    pub const fn media_asset_pid(&self) -> Option<Uuid> {
        self.media_asset_pid
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
    options: Option<Vec<CreateVariantOption>>,
    #[validate(nested)]
    pictures: Option<Vec<CreateProductPicture<'a>>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateProductVariant {
    #[validate(custom(function = "validate_optional_sku"))]
    sku: Option<String>,
    #[validate(custom(function = "validate_optional_price"))]
    price: Option<Decimal>,
    #[validate(range(min = 0, message = "Stock quantity cannot be negative"))]
    stock_quantity: Option<i32>,
}

impl UpdateProductVariant {
    #[must_use]
    pub fn sku(&self) -> Option<&str> {
        self.sku.as_deref()
    }

    #[must_use]
    pub const fn price(&self) -> Option<Decimal> {
        self.price
    }

    #[must_use]
    pub const fn stock_quantity(&self) -> Option<i32> {
        self.stock_quantity
    }
}

impl<'a> CreateProductVariant<'a> {
    #[must_use]
    pub const fn new(
        sku: Cow<'a, str>,
        price: Decimal,
        stock_quantity: i32,
        is_default: bool,
        options: Option<Vec<CreateVariantOption>>,
        pictures: Option<Vec<CreateProductPicture<'a>>>,
    ) -> Self {
        Self {
            sku,
            price,
            stock_quantity,
            is_default,
            options,
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
    pub fn options(&self) -> &[CreateVariantOption] {
        self.options.as_deref().unwrap_or_default()
    }

    #[must_use]
    pub fn pictures(&self) -> &[CreateProductPicture<'a>] {
        self.pictures.as_deref().unwrap_or_default()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantOption {
    #[validate(range(min = 1, message = "Attribute ID must be a positive integer"))]
    attribute_id: i32,
    #[validate(range(min = 1, message = "Attribute value ID must be a positive integer"))]
    attribute_value_id: i32,
    display_order: Option<i32>,
}

impl CreateVariantOption {
    #[must_use]
    pub const fn new(
        attribute_id: i32,
        attribute_value_id: i32,
        display_order: Option<i32>,
    ) -> Self {
        Self {
            attribute_id,
            attribute_value_id,
            display_order,
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

    #[must_use]
    pub const fn display_order(&self) -> Option<i32> {
        self.display_order
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

fn validate_optional_product_name(name: &str) -> Result<(), ValidationError> {
    validate_product_name(name)
}

#[expect(
    clippy::option_option,
    reason = "PATCH needs to distinguish omitted description from explicit null"
)]
fn deserialize_nullable_description<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
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

fn validate_optional_sku(sku: &str) -> Result<(), ValidationError> {
    validate_sku(sku)
}

fn validate_price(price: &Decimal) -> Result<(), ValidationError> {
    if price.is_sign_negative() {
        Err(ValidationError::new("negative_price")
            .with_message(Cow::Borrowed("Price must be greater than or equal to zero")))
    } else {
        Ok(())
    }
}

fn validate_optional_price(price: &Decimal) -> Result<(), ValidationError> {
    if price.is_sign_negative() {
        Err(ValidationError::new("negative_price")
            .with_message(Cow::Borrowed("Price must be greater than or equal to zero")))
    } else {
        Ok(())
    }
}

fn normalized_optional_str(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}
