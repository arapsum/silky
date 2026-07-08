use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::{
    models::{ModelError, ModelResult, PaginatedModel, Pagination, Seedable},
    schemas::CreateProduct,
    schemas::ProductListQuery,
    views::{
        ProductCategorySummary, ProductCreateResponse, ProductDetailResponse, ProductListItem,
        ProductOptionResponse, ProductPictureResponse, ProductVariantDetail,
        ProductVariantOptionResponse, ProductVariantSummary,
    },
};

mod attribute_values;
mod attributes;
mod options;
mod pictures;
mod variant_attribute_values;
mod variants;

pub use self::{
    attribute_values::AttributeValue,
    attributes::{Attribute, AttributeWithValues},
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

#[derive(Debug, FromRow)]
struct ProductListRow {
    pid: Uuid,
    name: String,
    description: Option<String>,
    category_id: i32,
    category_pid: Uuid,
    category_name: String,
    category_slug: String,
    primary_image: Option<String>,
    default_variant_pid: Option<Uuid>,
    default_variant_sku: Option<String>,
    default_variant_price: Option<Decimal>,
    default_variant_stock_quantity: Option<i32>,
    variant_count: i32,
    option_count: i32,
    total_stock: i32,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

impl ProductListRow {
    fn into_response(self) -> ProductListItem {
        ProductListItem {
            pid: self.pid,
            name: self.name,
            description: self.description,
            category: ProductCategorySummary {
                id: self.category_id,
                pid: self.category_pid,
                name: self.category_name,
                slug: self.category_slug,
            },
            primary_image: self.primary_image,
            default_variant: self
                .default_variant_pid
                .zip(self.default_variant_sku)
                .zip(self.default_variant_price)
                .zip(self.default_variant_stock_quantity)
                .map(
                    |(((pid, sku), price), stock_quantity)| ProductVariantSummary {
                        pid,
                        sku,
                        price,
                        stock_quantity,
                    },
                ),
            variant_count: self.variant_count,
            option_count: self.option_count,
            total_stock: self.total_stock,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(Debug, FromRow)]
struct ProductDetailHeader {
    id: i32,
    pid: Uuid,
    name: String,
    description: Option<String>,
    category_id: i32,
    category_pid: Uuid,
    category_name: String,
    category_slug: String,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, FromRow)]
struct ProductPictureRow {
    id: i32,
    pid: Uuid,
    variant_id: Option<i32>,
    image_link: String,
    display_order: Option<i32>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl ProductPictureRow {
    fn into_response(self) -> ProductPictureResponse {
        ProductPictureResponse {
            id: self.id,
            pid: self.pid,
            image_link: self.image_link,
            display_order: self.display_order,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, FromRow)]
struct ProductOptionRow {
    id: i32,
    pid: Uuid,
    attribute_id: i32,
    attribute_pid: Uuid,
    attribute_name: String,
    display_order: Option<i32>,
    created_at: DateTime<FixedOffset>,
}

impl ProductOptionRow {
    fn into_response(self) -> ProductOptionResponse {
        ProductOptionResponse {
            id: self.id,
            pid: self.pid,
            attribute_id: self.attribute_id,
            attribute_pid: self.attribute_pid,
            attribute_name: self.attribute_name,
            display_order: self.display_order,
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, FromRow)]
struct ProductVariantRow {
    id: i32,
    pid: Uuid,
    sku: String,
    price: Decimal,
    stock_quantity: i32,
    is_default: bool,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, FromRow)]
struct ProductVariantOptionRow {
    id: i32,
    pid: Uuid,
    variant_id: i32,
    attribute_id: i32,
    attribute_pid: Uuid,
    attribute_name: String,
    attribute_value_id: i32,
    attribute_value_pid: Uuid,
    value: String,
    created_at: DateTime<FixedOffset>,
}

impl ProductVariantOptionRow {
    fn into_response(self) -> ProductVariantOptionResponse {
        ProductVariantOptionResponse {
            id: self.id,
            pid: self.pid,
            attribute_id: self.attribute_id,
            attribute_pid: self.attribute_pid,
            attribute_name: self.attribute_name,
            attribute_value_id: self.attribute_value_id,
            attribute_value_pid: self.attribute_value_pid,
            value: self.value,
            created_at: self.created_at,
        }
    }
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

    /// Lists products with catalogue summary data and pagination metadata.
    ///
    /// Defaults to page `1` and limit `20` when query values are missing. The
    /// limit is clamped to the range `1..=40`, and the page is clamped to a
    /// minimum of `1`.
    ///
    /// # Parameters
    ///
    /// - `db`: Database pool used to count and fetch products.
    /// - `query`: Validated product list filters and pagination settings.
    ///
    /// # Errors
    ///
    /// Returns a database error if counting or fetching products fails, or if
    /// committing the transaction fails.
    #[expect(
        clippy::too_many_lines,
        reason = "SQL-heavy catalogue projection is kept together for filter parity"
    )]
    pub async fn find_list(
        db: &PgPool,
        query: &ProductListQuery,
    ) -> ModelResult<PaginatedModel<ProductListItem>> {
        let limit = query.limit().unwrap_or(20).clamp(1, 40);
        let page = query.page().unwrap_or(1).max(1);
        let offset = (page - 1) * limit;
        let stock_status = query
            .stock_status()
            .map(crate::schemas::StockStatus::as_str);

        let mut txn = db.begin().await?;

        let total_items = sqlx::query_scalar::<_, i64>(
            r"
            SELECT COUNT(*)
            FROM products p
            INNER JOIN categories c ON c.id = p.category_id
            LEFT JOIN LATERAL (
                SELECT COALESCE(SUM(v.stock_quantity), 0)::int4 AS total_stock
                FROM product_variants v
                WHERE v.product_id = p.id
                    AND ($9::BOOL = TRUE OR v.deleted_at IS NULL)
            ) variant_stock ON TRUE
            WHERE
                ($9::BOOL = TRUE OR p.deleted_at IS NULL)
                AND (
                    $1::TEXT IS NULL
                    OR p.name ILIKE '%' || $1 || '%'
                    OR COALESCE(p.description, '') ILIKE '%' || $1 || '%'
                    OR c.name ILIKE '%' || $1 || '%'
                    OR c.slug ILIKE '%' || $1 || '%'
                    OR EXISTS (
                        SELECT 1
                        FROM product_variants sv
                        WHERE sv.product_id = p.id
                            AND sv.sku ILIKE '%' || $1 || '%'
                            AND ($9::BOOL = TRUE OR sv.deleted_at IS NULL)
                    )
                )
                AND ($2::TEXT IS NULL OR p.name = TRIM($2))
                AND ($3::INT4 IS NULL OR p.category_id = $3)
                AND ($4::TEXT IS NULL OR c.slug = LOWER(TRIM($4)))
                AND (
                    $5::TEXT IS NULL
                    OR EXISTS (
                        SELECT 1
                        FROM product_variants sku_filter
                        WHERE sku_filter.product_id = p.id
                            AND LOWER(sku_filter.sku) = LOWER(TRIM($5))
                            AND ($9::BOOL = TRUE OR sku_filter.deleted_at IS NULL)
                    )
                )
                AND (
                    ($6::NUMERIC IS NULL AND $7::NUMERIC IS NULL)
                    OR EXISTS (
                        SELECT 1
                        FROM product_variants price_filter
                        WHERE price_filter.product_id = p.id
                            AND ($6::NUMERIC IS NULL OR price_filter.price >= $6)
                            AND ($7::NUMERIC IS NULL OR price_filter.price <= $7)
                            AND ($9::BOOL = TRUE OR price_filter.deleted_at IS NULL)
                    )
                )
                AND (
                    $8::TEXT IS NULL
                    OR ($8 = 'inStock' AND COALESCE(variant_stock.total_stock, 0) > 0)
                    OR ($8 = 'outOfStock' AND COALESCE(variant_stock.total_stock, 0) = 0)
                )
        ",
        )
        .bind(query.search())
        .bind(query.name())
        .bind(query.category_id())
        .bind(query.category_slug())
        .bind(query.sku())
        .bind(query.min_price())
        .bind(query.max_price())
        .bind(stock_status)
        .bind(query.include_deleted())
        .fetch_one(&mut *txn)
        .await?;

        let products = sqlx::query_as::<_, ProductListRow>(
            r"
            SELECT
                p.pid,
                p.name,
                p.description,
                c.id AS category_id,
                c.pid AS category_pid,
                c.name AS category_name,
                c.slug AS category_slug,
                primary_picture.image_link AS primary_image,
                default_variant.pid AS default_variant_pid,
                default_variant.sku AS default_variant_sku,
                default_variant.price AS default_variant_price,
                default_variant.stock_quantity AS default_variant_stock_quantity,
                COALESCE(variant_stats.variant_count, 0)::int4 AS variant_count,
                COALESCE(option_stats.option_count, 0)::int4 AS option_count,
                COALESCE(variant_stats.total_stock, 0)::int4 AS total_stock,
                p.created_at,
                p.updated_at,
                p.deleted_at
            FROM products p
            INNER JOIN categories c ON c.id = p.category_id
            LEFT JOIN LATERAL (
                SELECT
                    COUNT(*)::int4 AS variant_count,
                    COALESCE(SUM(v.stock_quantity), 0)::int4 AS total_stock
                FROM product_variants v
                WHERE v.product_id = p.id
                    AND ($11::BOOL = TRUE OR v.deleted_at IS NULL)
            ) variant_stats ON TRUE
            LEFT JOIN LATERAL (
                SELECT COUNT(*)::int4 AS option_count
                FROM product_options po
                WHERE po.product_id = p.id
            ) option_stats ON TRUE
            LEFT JOIN LATERAL (
                SELECT pic.image_link
                FROM pictures pic
                WHERE pic.product_id = p.id
                ORDER BY (pic.variant_id IS NOT NULL), pic.display_order NULLS LAST, pic.id
                LIMIT 1
            ) primary_picture ON TRUE
            LEFT JOIN LATERAL (
                SELECT dv.pid, dv.sku, dv.price, dv.stock_quantity
                FROM product_variants dv
                WHERE dv.product_id = p.id
                    AND dv.is_default = TRUE
                    AND ($11::BOOL = TRUE OR dv.deleted_at IS NULL)
                ORDER BY dv.id
                LIMIT 1
            ) default_variant ON TRUE
            WHERE
                ($11::BOOL = TRUE OR p.deleted_at IS NULL)
                AND (
                    $3::TEXT IS NULL
                    OR p.name ILIKE '%' || $3 || '%'
                    OR COALESCE(p.description, '') ILIKE '%' || $3 || '%'
                    OR c.name ILIKE '%' || $3 || '%'
                    OR c.slug ILIKE '%' || $3 || '%'
                    OR EXISTS (
                        SELECT 1
                        FROM product_variants sv
                        WHERE sv.product_id = p.id
                            AND sv.sku ILIKE '%' || $3 || '%'
                            AND ($11::BOOL = TRUE OR sv.deleted_at IS NULL)
                    )
                )
                AND ($4::TEXT IS NULL OR p.name = TRIM($4))
                AND ($5::INT4 IS NULL OR p.category_id = $5)
                AND ($6::TEXT IS NULL OR c.slug = LOWER(TRIM($6)))
                AND (
                    $7::TEXT IS NULL
                    OR EXISTS (
                        SELECT 1
                        FROM product_variants sku_filter
                        WHERE sku_filter.product_id = p.id
                            AND LOWER(sku_filter.sku) = LOWER(TRIM($7))
                            AND ($11::BOOL = TRUE OR sku_filter.deleted_at IS NULL)
                    )
                )
                AND (
                    ($8::NUMERIC IS NULL AND $9::NUMERIC IS NULL)
                    OR EXISTS (
                        SELECT 1
                        FROM product_variants price_filter
                        WHERE price_filter.product_id = p.id
                            AND ($8::NUMERIC IS NULL OR price_filter.price >= $8)
                            AND ($9::NUMERIC IS NULL OR price_filter.price <= $9)
                            AND ($11::BOOL = TRUE OR price_filter.deleted_at IS NULL)
                    )
                )
                AND (
                    $10::TEXT IS NULL
                    OR ($10 = 'inStock' AND COALESCE(variant_stats.total_stock, 0) > 0)
                    OR ($10 = 'outOfStock' AND COALESCE(variant_stats.total_stock, 0) = 0)
                )
            ORDER BY p.created_at DESC, p.id DESC
            LIMIT $1 OFFSET $2
        ",
        )
        .bind(limit)
        .bind(offset)
        .bind(query.search())
        .bind(query.name())
        .bind(query.category_id())
        .bind(query.category_slug())
        .bind(query.sku())
        .bind(query.min_price())
        .bind(query.max_price())
        .bind(stock_status)
        .bind(query.include_deleted())
        .fetch_all(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(PaginatedModel::new(
            products
                .into_iter()
                .map(ProductListRow::into_response)
                .collect(),
            Pagination::new(page, limit, total_items),
        ))
    }

    /// Finds a product by public ID and returns its catalogue aggregate.
    ///
    /// The returned aggregate includes product pictures, ordered product
    /// options, variants, variant pictures, and selected attribute values.
    ///
    /// # Parameters
    ///
    /// - `db`: Database pool used for the aggregate queries.
    /// - `pid`: Public product ID to fetch.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no non-deleted product
    /// exists for `pid`. Returns a database error if any aggregate query fails
    /// or if committing the transaction fails.
    #[expect(
        clippy::too_many_lines,
        reason = "Aggregate fetch keeps the related read-model queries in one transaction"
    )]
    pub async fn find_detail_by_pid(db: &PgPool, pid: Uuid) -> ModelResult<ProductDetailResponse> {
        let mut txn = db.begin().await?;

        let product = sqlx::query_as::<_, ProductDetailHeader>(
            r"
            SELECT
                p.id,
                p.pid,
                p.name,
                p.description,
                c.id AS category_id,
                c.pid AS category_pid,
                c.name AS category_name,
                c.slug AS category_slug,
                p.created_at,
                p.updated_at,
                p.deleted_at
            FROM products p
            INNER JOIN categories c ON c.id = p.category_id
            WHERE p.pid = $1
                AND p.deleted_at IS NULL
        ",
        )
        .bind(pid)
        .fetch_optional(&mut *txn)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)?;

        let product_pictures = sqlx::query_as::<_, ProductPictureRow>(
            r"
            SELECT id, pid, variant_id, image_link, display_order, created_at, updated_at
            FROM pictures
            WHERE product_id = $1
                AND variant_id IS NULL
            ORDER BY display_order NULLS LAST, id
        ",
        )
        .bind(product.id)
        .fetch_all(&mut *txn)
        .await?
        .into_iter()
        .map(ProductPictureRow::into_response)
        .collect();

        let options = sqlx::query_as::<_, ProductOptionRow>(
            r"
            SELECT
                po.id,
                po.pid,
                po.attribute_id,
                a.pid AS attribute_pid,
                a.name AS attribute_name,
                po.display_order,
                po.created_at
            FROM product_options po
            INNER JOIN attributes a ON a.id = po.attribute_id
            WHERE po.product_id = $1
            ORDER BY po.display_order NULLS LAST, po.id
        ",
        )
        .bind(product.id)
        .fetch_all(&mut *txn)
        .await?
        .into_iter()
        .map(ProductOptionRow::into_response)
        .collect();

        let variant_rows = sqlx::query_as::<_, ProductVariantRow>(
            r"
            SELECT id, pid, sku, price, stock_quantity, is_default, created_at, updated_at, deleted_at
            FROM product_variants
            WHERE product_id = $1
                AND deleted_at IS NULL
            ORDER BY is_default DESC, id
        ",
        )
        .bind(product.id)
        .fetch_all(&mut *txn)
        .await?;

        let variant_pictures = sqlx::query_as::<_, ProductPictureRow>(
            r"
            SELECT id, pid, variant_id, image_link, display_order, created_at, updated_at
            FROM pictures
            WHERE product_id = $1
                AND variant_id IS NOT NULL
            ORDER BY variant_id, display_order NULLS LAST, id
        ",
        )
        .bind(product.id)
        .fetch_all(&mut *txn)
        .await?;

        let variant_options = sqlx::query_as::<_, ProductVariantOptionRow>(
            r"
            SELECT
                vav.id,
                vav.pid,
                vav.variant_id,
                vav.attribute_id,
                a.pid AS attribute_pid,
                a.name AS attribute_name,
                vav.attribute_value_id,
                av.pid AS attribute_value_pid,
                av.value,
                vav.created_at
            FROM variant_attribute_values vav
            INNER JOIN attributes a ON a.id = vav.attribute_id
            INNER JOIN attribute_values av ON av.id = vav.attribute_value_id
            INNER JOIN product_variants pv ON pv.id = vav.variant_id
            WHERE pv.product_id = $1
                AND pv.deleted_at IS NULL
            ORDER BY vav.variant_id, a.name, av.value, vav.id
        ",
        )
        .bind(product.id)
        .fetch_all(&mut *txn)
        .await?;

        txn.commit().await?;

        let mut pictures_by_variant = HashMap::<i32, Vec<ProductPictureResponse>>::new();
        for picture in variant_pictures {
            if let Some(variant_id) = picture.variant_id {
                pictures_by_variant
                    .entry(variant_id)
                    .or_default()
                    .push(picture.into_response());
            }
        }

        let mut options_by_variant = HashMap::<i32, Vec<ProductVariantOptionResponse>>::new();
        for option in variant_options {
            options_by_variant
                .entry(option.variant_id)
                .or_default()
                .push(option.into_response());
        }

        let variants = variant_rows
            .into_iter()
            .map(|variant| ProductVariantDetail {
                id: variant.id,
                pid: variant.pid,
                sku: variant.sku,
                price: variant.price,
                stock_quantity: variant.stock_quantity,
                is_default: variant.is_default,
                options: options_by_variant.remove(&variant.id).unwrap_or_default(),
                pictures: pictures_by_variant.remove(&variant.id).unwrap_or_default(),
                created_at: variant.created_at,
                updated_at: variant.updated_at,
                deleted_at: variant.deleted_at,
            })
            .collect();

        Ok(ProductDetailResponse {
            id: product.id,
            pid: product.pid,
            name: product.name,
            description: product.description,
            category: ProductCategorySummary {
                id: product.category_id,
                pid: product.category_pid,
                name: product.category_name,
                slug: product.category_slug,
            },
            pictures: product_pictures,
            options,
            variants,
            created_at: product.created_at,
            updated_at: product.updated_at,
            deleted_at: product.deleted_at,
        })
    }

    /// Finds a product by public ID.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no product exists for
    /// `pid`. Returns a database error if the lookup fails.
    pub async fn find_by_pid<'e, E>(db: E, pid: Uuid) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, Self>("SELECT * FROM products WHERE pid = $1")
            .bind(pid)
            .fetch_optional(db)
            .await?
            .ok_or_else(|| ModelError::EntityNotFound)
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
