use serde::{Deserialize, Serialize};

use crate::models::{Picture, Product, ProductOption, ProductVariant, VariantAttributeValue};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductCreateResponse {
    pub product: Product,
    pub options: Vec<ProductOption>,
    pub pictures: Vec<Picture>,
    pub variants: Vec<ProductVariant>,
    pub variant_attribute_values: Vec<VariantAttributeValue>,
}

impl ProductCreateResponse {
    #[must_use]
    pub const fn new(
        product: Product,
        options: Vec<ProductOption>,
        pictures: Vec<Picture>,
        variants: Vec<ProductVariant>,
        variant_attribute_values: Vec<VariantAttributeValue>,
    ) -> Self {
        Self {
            product,
            options,
            pictures,
            variants,
            variant_attribute_values,
        }
    }
}
