use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::{
    models::{ModelError, ModelResult, Seedable},
    schemas::CreateProduct,
    views::ProductCreateResponse,
};

mod attribute_values;
mod attributes;
mod options;
mod pictures;
mod variant_attribute_values;
mod variants;

pub use self::{
    attribute_values::AttributeValue,
    attributes::Attribute,
    options::{NewProductOption, ProductOption},
    pictures::{NewPicture, Picture},
    variant_attribute_values::{NewVariantAttributeValue, VariantAttributeValue},
    variants::{NewVariant, ProductVariant},
};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    id: i32,
    pid: Uuid,
    category_id: i32,
    name: String,
    description: Option<String>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

impl Product {
    /// Creates a product and its optional setup records.
    ///
    /// The base product, variant options, pictures, variants, and variant
    /// attribute value links are inserted in a single transaction. If any nested insert
    /// fails, the whole transaction is rolled back.
    ///
    /// # Parameters
    ///
    /// - `db`: Database pool used to open the transaction.
    /// - `params`: Validated product creation request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::models::ModelError::EntityAlreadyExists`] for product
    /// name, SKU, default-variant, option, or variant-attribute uniqueness
    /// violations. Returns [`crate::models::ModelError::InvalidReference`]
    /// when a referenced category, attribute, attribute value, product, or
    /// variant does not exist. Returns a database error if any insert or
    /// transaction operation fails for another reason.
    pub async fn create(
        db: &PgPool,
        params: &CreateProduct<'_>,
    ) -> ModelResult<ProductCreateResponse> {
        let mut txn = db.begin().await?;

        let product = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO products (
                category_id,
                name,
                description
            ) VALUES (
                $1,
                $2,
                $3
            ) RETURNING *
        ",
        )
        .bind(params.category_id())
        .bind(params.name().trim())
        .bind(params.description().map(|description| description.trim()))
        .fetch_one(&mut *txn)
        .await?;

        let mut options = Vec::new();
        let mut pictures = Vec::with_capacity(params.pictures().len());
        let mut variants = Vec::with_capacity(params.variants().len());
        let variant_attribute_capacity = params
            .variants()
            .iter()
            .map(|variant| variant.options().len())
            .sum();
        let mut variant_attribute_values = Vec::with_capacity(variant_attribute_capacity);
        let mut option_orders = Vec::<(i32, Option<i32>)>::new();

        for picture in params.pictures() {
            let params = NewPicture::new(
                product.id(),
                picture.image_link().trim().to_string(),
                None,
                picture.display_order(),
            );
            let picture = Picture::create(&mut *txn, &params).await?;
            pictures.push(picture);
        }

        for variant in params.variants() {
            let params = NewVariant::new(
                product.id(),
                variant.sku().trim().to_string(),
                variant.price(),
                variant.stock_quantity(),
                variant.is_default(),
            );
            let created_variant = ProductVariant::create(&mut *txn, &params).await?;

            for option in variant.options() {
                if let Some((_, display_order)) = option_orders
                    .iter()
                    .find(|(attribute_id, _)| *attribute_id == option.attribute_id())
                {
                    if *display_order != option.display_order() {
                        return Err(ModelError::InvalidInput(
                            "Product option display order must be consistent for each attribute."
                                .to_string(),
                        ));
                    }
                } else {
                    let params = NewProductOption::new(
                        product.id(),
                        option.attribute_id(),
                        option.display_order(),
                    );
                    let product_option = ProductOption::create(&mut *txn, &params).await?;
                    options.push(product_option);
                    option_orders.push((option.attribute_id(), option.display_order()));
                }

                let params = NewVariantAttributeValue::new(
                    created_variant.id(),
                    option.attribute_id(),
                    option.attribute_value_id(),
                );
                let value = VariantAttributeValue::create(&mut *txn, &params).await?;
                variant_attribute_values.push(value);
            }

            for picture in variant.pictures() {
                let params = NewPicture::new(
                    product.id(),
                    picture.image_link().trim().to_string(),
                    Some(created_variant.id()),
                    picture.display_order(),
                );
                let picture = Picture::create(&mut *txn, &params).await?;
                pictures.push(picture);
            }

            variants.push(created_variant);
        }

        txn.commit().await?;

        Ok(ProductCreateResponse::new(
            product,
            options,
            pictures,
            variants,
            variant_attribute_values,
        ))
    }

    /// Loads products from a JSON file in `src/data` and seeds them.
    ///
    /// # Parameters
    ///
    /// - `db`: Database pool used for inserts and updates.
    /// - `file`: File name relative to `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialization, or database error if loading,
    /// decoding, inserting, or updating the products fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }

    /// Returns the internal database row ID.
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    /// Returns the public product ID.
    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    /// Returns the category row ID that owns this product.
    #[must_use]
    pub const fn category_id(&self) -> i32 {
        self.category_id
    }

    /// Returns the product name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the optional product description.
    #[must_use]
    pub const fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    /// Returns when the product was created.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    /// Returns when the product was last updated.
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }

    /// Returns when the product was soft-deleted, if it has been deleted.
    #[must_use]
    pub const fn deleted_at(&self) -> Option<DateTime<FixedOffset>> {
        self.deleted_at
    }
}

impl Seedable for Product {
    async fn seed(db: &sqlx::PgPool, data: &[Self]) -> super::ModelResult<()> {
        for product in data {
            sqlx::query(
                r"
                INSERT INTO products (
                    id,
                    pid,
                    category_id,
                    name,
                    description,
                    created_at,
                    updated_at,
                    deleted_at
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8
                ) ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    category_id = EXCLUDED.category_id,
                    name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at,
                    deleted_at = EXCLUDED.deleted_at
           ",
            )
            .bind(product.id())
            .bind(product.pid())
            .bind(product.category_id())
            .bind(product.name())
            .bind(product.description())
            .bind(product.created_at())
            .bind(product.updated_at())
            .bind(product.deleted_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
