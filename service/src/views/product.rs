use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Picture, Product, ProductOption, ProductVariant, Tag, VariantAttributeValue};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductCreateResponse {
    pub product: Product,
    pub tags: Vec<Tag>,
    pub options: Vec<ProductOption>,
    pub pictures: Vec<Picture>,
    pub variants: Vec<ProductVariant>,
    pub variant_attribute_values: Vec<VariantAttributeValue>,
}

impl ProductCreateResponse {
    #[must_use]
    pub const fn new(
        product: Product,
        tags: Vec<Tag>,
        options: Vec<ProductOption>,
        pictures: Vec<Picture>,
        variants: Vec<ProductVariant>,
        variant_attribute_values: Vec<VariantAttributeValue>,
    ) -> Self {
        Self {
            product,
            tags,
            options,
            pictures,
            variants,
            variant_attribute_values,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductCategorySummary {
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductVariantSummary {
    pub pid: Uuid,
    pub sku: String,
    pub price: Decimal,
    pub stock_quantity: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductListItem {
    pub pid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub information: BTreeMap<String, String>,
    pub category: ProductCategorySummary,
    pub primary_image: Option<String>,
    pub default_variant: Option<ProductVariantSummary>,
    pub variant_count: i32,
    pub option_count: i32,
    pub total_stock: i32,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductPictureResponse {
    pub id: i32,
    pub pid: Uuid,
    pub image_link: String,
    pub display_order: Option<i32>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductOptionResponse {
    pub id: i32,
    pub pid: Uuid,
    pub attribute_id: i32,
    pub attribute_pid: Uuid,
    pub attribute_name: String,
    pub attribute_description: Option<String>,
    pub display_order: Option<i32>,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductVariantOptionResponse {
    pub id: i32,
    pub pid: Uuid,
    pub attribute_id: i32,
    pub attribute_pid: Uuid,
    pub attribute_name: String,
    pub attribute_description: Option<String>,
    pub attribute_value_id: i32,
    pub attribute_value_pid: Uuid,
    pub value: String,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductVariantDetail {
    pub id: i32,
    pub pid: Uuid,
    pub sku: String,
    pub price: Decimal,
    pub stock_quantity: i32,
    pub is_default: bool,
    pub options: Vec<ProductVariantOptionResponse>,
    pub pictures: Vec<ProductPictureResponse>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductDetailResponse {
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub information: BTreeMap<String, String>,
    pub tags: Vec<Tag>,
    pub category: ProductCategorySummary,
    pub pictures: Vec<ProductPictureResponse>,
    pub options: Vec<ProductOptionResponse>,
    pub variants: Vec<ProductVariantDetail>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
