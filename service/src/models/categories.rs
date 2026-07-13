#![allow(unused_imports)]
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, PgConnection, PgPool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::{
    schemas::{CategoryAttributesInput, CategoryListQuery, NewCategory, UpdateCategory},
    views::{
        CategoryAttributeResponse, CategoryChildResponse, CategoryDetailResponse, CategoryResponse,
        CategoryTopProductResponse,
    },
};

use super::{ModelError, ModelResult, PaginatedModel, Pagination, Seedable};

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    id: i32,
    pid: Uuid,
    name: String,
    slug: String,
    image_link: String,
    description: Option<String>,
    parent_id: Option<i32>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryAttributeLink {
    category_id: i32,
    attribute_id: i32,
}

#[derive(Debug, FromRow)]
struct CategoryChildRow {
    id: i32,
    pid: Uuid,
    name: String,
    slug: String,
    image_link: String,
    product_count: i32,
}

#[derive(Debug, FromRow)]
struct CategoryAttributeRow {
    id: i32,
    pid: Uuid,
    name: String,
    description: Option<String>,
}

#[derive(Debug, FromRow)]
struct CategoryTopProductRow {
    pid: Uuid,
    name: String,
    image_link: Option<String>,
    sku: Option<String>,
    stock_quantity: i32,
}

impl Category {
    /// Creates a new category.
    ///
    /// The category name is trimmed and stored in lowercase before insertion.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityAlreadyExists`] when a category with the
    /// same normalized name exists. Returns a database error if the duplicate
    /// lookup, insertion, or transaction commit fails.
    pub async fn create(db: &PgPool, params: &NewCategory<'_>) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        if let Some(category) = Self::find_by_name(&mut *txn, params.name()).await? {
            return Err(ModelError::EntityAlreadyExists(format!(
                "Category {} already exists!",
                category.name()
            )));
        }

        let slug_base = params
            .slug()
            .map_or_else(|| params.name(), |slug| slug.as_ref());
        let slug = Self::create_unique_slug(&mut txn, slug_base, None).await?;

        let created = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO categories (
                name,
                slug,
                image_link,
                parent_id,
                description,
                media_asset_id
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                (SELECT id FROM media_assets WHERE pid = $6)
            ) RETURNING *
        ",
        )
        .bind(params.name().to_lowercase().trim())
        .bind(slug)
        .bind(params.image_link().trim())
        .bind(params.parent_id())
        .bind(params.description().map(|s| s.trim()))
        .bind(params.media_asset_pid())
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(created)
    }

    /// Updates an existing category by public ID.
    ///
    /// Provided string fields are trimmed, and the category name is stored in
    /// lowercase when changed.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no category exists for
    /// `pid`. Returns [`ModelError::EntityAlreadyExists`] when the requested
    /// name belongs to another category. Returns a database error if the lookup,
    /// update, or transaction commit fails.
    pub async fn update(db: &PgPool, pid: Uuid, params: &UpdateCategory<'_>) -> ModelResult<Self> {
        let mut txn = db.begin().await?;

        let exists = Self::find_by_pid(&mut *txn, pid).await?;

        if let Some(name) = params.name()
            && let Some(category) = Self::find_by_name(&mut *txn, name).await?
            && category.pid() != exists.pid()
        {
            return Err(ModelError::EntityAlreadyExists(format!(
                "Category {} already exists!",
                category.name()
            )));
        }

        let slug = if let Some(value) = params.slug().or_else(|| params.name()) {
            Some(Self::create_unique_slug(&mut txn, value.as_ref(), Some(exists.pid())).await?)
        } else {
            None
        };

        let updated = sqlx::query_as::<_, Self>(
            r"
                UPDATE categories
                SET
                    name = COALESCE($1, name),
                    slug = COALESCE($2, slug),
                    image_link = COALESCE($3, image_link),
                    parent_id = CASE WHEN $4 THEN NULL ELSE COALESCE($5, parent_id) END,
                    description = COALESCE($6, description),
                    media_asset_id = COALESCE((SELECT id FROM media_assets WHERE pid = $7), media_asset_id)
                WHERE pid = $8
                RETURNING *
        ",
        )
        .bind(params.name().map(|s| s.trim().to_lowercase()))
        .bind(slug)
        .bind(params.image_link().map(|s| s.trim()))
        .bind(params.clear_parent())
        .bind(params.parent_id())
        .bind(params.description().map(|s| s.trim()))
        .bind(params.media_asset_pid())
        .bind(exists.pid())
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(updated)
    }

    /// Lists categories with pagination metadata.
    ///
    /// Defaults to page `1` and limit `20` when query values are missing. The
    /// limit is clamped to the range `1..=40`, and the page is clamped to a
    /// minimum of `1`.
    ///
    /// # Errors
    ///
    /// Returns a database error if counting or fetching categories fails, or if
    /// the transaction commit fails.
    pub async fn find_all(
        db: &PgPool,
        query: &CategoryListQuery,
    ) -> ModelResult<PaginatedModel<Self>> {
        let mut txn = db.begin().await?;

        let limit = query.limit().unwrap_or(20).clamp(1, 40);
        let page = query.page().unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let total_items: i64 = sqlx::query_scalar(
            r"
            SELECT COUNT(*) FROM categories
            WHERE
                ($1::TEXT IS NULL OR name ILIKE '%' || $1 || '%' OR slug ILIKE '%' || $1 || '%')
                AND ($2::TEXT IS NULL OR name = LOWER(TRIM($2)))
                AND ($3::TEXT IS NULL OR slug = LOWER(TRIM($3)))
                AND ($4::INT4 IS NULL OR parent_id = $4)
                AND (
                    $5::BOOL IS NULL
                    OR ($5 = TRUE AND parent_id IS NOT NULL)
                    OR ($5 = FALSE AND parent_id IS NULL)
                )
                AND ($6::BOOL = TRUE OR deleted_at IS NULL)
        ",
        )
        .bind(query.search())
        .bind(query.name())
        .bind(query.slug())
        .bind(query.parent_id())
        .bind(query.has_parent())
        .bind(query.include_deleted())
        .fetch_one(&mut *txn)
        .await?;

        let categories = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM categories
            WHERE
                ($3::TEXT IS NULL OR name ILIKE '%' || $3 || '%' OR slug ILIKE '%' || $3 || '%')
                AND ($4::TEXT IS NULL OR name = LOWER(TRIM($4)))
                AND ($5::TEXT IS NULL OR slug = LOWER(TRIM($5)))
                AND ($6::INT4 IS NULL OR parent_id = $6)
                AND (
                    $7::BOOL IS NULL
                    OR ($7 = TRUE AND parent_id IS NOT NULL)
                    OR ($7 = FALSE AND parent_id IS NULL)
                )
                AND ($8::BOOL = TRUE OR deleted_at IS NULL)
            ORDER BY created_at DESC, id DESC
            LIMIT $1 OFFSET $2
        ",
        )
        .bind(limit)
        .bind(offset)
        .bind(query.search())
        .bind(query.name())
        .bind(query.slug())
        .bind(query.parent_id())
        .bind(query.has_parent())
        .bind(query.include_deleted())
        .fetch_all(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(PaginatedModel::new(
            categories,
            Pagination::new(page, limit, total_items),
        ))
    }

    /// Lists categories with pagination metadata and product counts.
    ///
    /// Defaults to page `1` and limit `20` when query values are missing. The
    /// limit is clamped to the range `1..=40`, and the page is clamped to a
    /// minimum of `1`.
    ///
    /// # Errors
    ///
    /// Returns a database error if counting categories, fetching categories
    /// with product totals, or committing the transaction fails.
    pub async fn find_all_with_products_count(
        db: &PgPool,
        query: &CategoryListQuery,
    ) -> ModelResult<PaginatedModel<CategoryResponse>> {
        let limit = query.limit().unwrap_or(20).clamp(1, 40);
        let page = query.page().unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let mut txn = db.begin().await?;

        let total_items = sqlx::query_scalar::<_, i64>(
            r"
            SELECT COUNT(*) FROM categories
            WHERE
                ($1::TEXT IS NULL OR name ILIKE '%' || $1 || '%' OR slug ILIKE '%' || $1 || '%')
                AND ($2::TEXT IS NULL OR name = LOWER(TRIM($2)))
                AND ($3::TEXT IS NULL OR slug = LOWER(TRIM($3)))
                AND ($4::INT4 IS NULL OR parent_id = $4)
                AND (
                    $5::BOOL IS NULL
                    OR ($5 = TRUE AND parent_id IS NOT NULL)
                    OR ($5 = FALSE AND parent_id IS NULL)
                )
                AND ($6::BOOL = TRUE OR deleted_at IS NULL)
        ",
        )
        .bind(query.search())
        .bind(query.name())
        .bind(query.slug())
        .bind(query.parent_id())
        .bind(query.has_parent())
        .bind(query.include_deleted())
        .fetch_one(&mut *txn)
        .await?;

        let categories = sqlx::query_as::<_, CategoryResponse>(
            r"
            SELECT
                   c.id,
                   c.pid,
                   c.name,
                   c.slug,
                   c.image_link,
                   c.description,
                   c.parent_id,
                   p.name AS parent_name,
                   COUNT(prod.id)::int4 AS product_count,
                   c.created_at,
                   c.updated_at,
                   c.deleted_at
               FROM categories c
               LEFT JOIN categories p ON p.id = c.parent_id
               LEFT JOIN products prod ON prod.category_id = c.id
               WHERE
                   ($3::TEXT IS NULL OR c.name ILIKE '%' || $3 || '%' OR c.slug ILIKE '%' || $3 || '%')
                   AND ($4::TEXT IS NULL OR c.name = LOWER(TRIM($4)))
                   AND ($5::TEXT IS NULL OR c.slug = LOWER(TRIM($5)))
                   AND ($6::INT4 IS NULL OR c.parent_id = $6)
                   AND (
                       $7::BOOL IS NULL
                       OR ($7 = TRUE AND c.parent_id IS NOT NULL)
                       OR ($7 = FALSE AND c.parent_id IS NULL)
                   )
                   AND ($8::BOOL = TRUE OR c.deleted_at IS NULL)
               GROUP BY c.id, p.name
               ORDER BY c.created_at DESC, c.id DESC
               LIMIT $1 OFFSET $2
        ",
        )
        .bind(limit)
        .bind(offset)
        .bind(query.search())
        .bind(query.name())
        .bind(query.slug())
        .bind(query.parent_id())
        .bind(query.has_parent())
        .bind(query.include_deleted())
        .fetch_all(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(PaginatedModel::new(
            categories,
            Pagination::new(page, limit, total_items),
        ))
    }

    /// Finds a category by normalized slug.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup fails.
    pub async fn find_by_slug<'e, E>(db: E, slug: &str) -> ModelResult<Option<Self>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM categories WHERE slug = $1
        ",
        )
        .bind(slug.to_lowercase().trim())
        .fetch_optional(db)
        .await
        .map_err(Into::into)
    }

    fn slugify(value: &str) -> String {
        let slug = value
            .to_lowercase()
            .trim()
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        if slug.is_empty() {
            "category".to_string()
        } else {
            slug
        }
    }

    async fn create_unique_slug(
        db: &mut PgConnection,
        value: &str,
        current_pid: Option<Uuid>,
    ) -> ModelResult<String> {
        let base = Self::slugify(value);

        for suffix in 0..1000 {
            let candidate = if suffix == 0 {
                base.clone()
            } else {
                format!("{base}-{suffix}")
            };

            let existing = Self::find_by_slug(&mut *db, &candidate).await?;
            let is_available = existing
                .as_ref()
                .is_none_or(|category| current_pid.is_some_and(|pid| category.pid() == pid));

            if is_available {
                return Ok(candidate);
            }
        }

        Ok(format!("{}-{}", base, Uuid::new_v4().simple()))
    }

    /// Finds a category by public ID.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no category exists for
    /// `pid`. Returns a database error if the lookup fails.
    pub async fn find_by_pid<'e, E>(db: E, pid: Uuid) -> ModelResult<Self>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM categories WHERE pid = $1
        ",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Loads a category with its children, linked attributes, and top products.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when the category does not exist,
    /// or a database error when any related data cannot be queried.
    #[expect(
        clippy::too_many_lines,
        reason = "The category detail read model keeps its related queries transactional"
    )]
    pub async fn find_detail_by_pid(db: &PgPool, pid: Uuid) -> ModelResult<CategoryDetailResponse> {
        let mut txn = db.begin().await?;
        let category = Self::find_by_pid(&mut *txn, pid).await?;
        let parent_name = if let Some(parent_id) = category.parent_id() {
            sqlx::query_scalar::<_, String>("SELECT name FROM categories WHERE id = $1")
                .bind(parent_id)
                .fetch_optional(&mut *txn)
                .await?
        } else {
            None
        };
        let product_count = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*)::int4 FROM products WHERE category_id = $1 AND deleted_at IS NULL",
        )
        .bind(category.id())
        .fetch_one(&mut *txn)
        .await?;
        let (total_variants, total_stock) = sqlx::query_as::<_, (i64, i64)>(
            r"
            SELECT COUNT(v.id)::bigint, COALESCE(SUM(v.stock_quantity), 0)::bigint
            FROM products p
            LEFT JOIN product_variants v ON v.product_id = p.id AND v.deleted_at IS NULL
            WHERE p.category_id = $1 AND p.deleted_at IS NULL
            ",
        )
        .bind(category.id())
        .fetch_one(&mut *txn)
        .await?;
        let children = sqlx::query_as::<_, CategoryChildRow>(
            r"
            SELECT c.id, c.pid, c.name, c.slug, c.image_link,
                   COUNT(p.id)::int4 AS product_count
            FROM categories c
            LEFT JOIN products p ON p.category_id = c.id AND p.deleted_at IS NULL
            WHERE c.parent_id = $1 AND c.deleted_at IS NULL
            GROUP BY c.id
            ORDER BY c.name
            ",
        )
        .bind(category.id())
        .fetch_all(&mut *txn)
        .await?;
        let attributes = sqlx::query_as::<_, CategoryAttributeRow>(
            r"
            SELECT a.id, a.pid, a.name, a.description
            FROM category_attributes ca
            INNER JOIN attributes a ON a.id = ca.attribute_id
            WHERE ca.category_id = $1
            ORDER BY a.name
            ",
        )
        .bind(category.id())
        .fetch_all(&mut *txn)
        .await?;
        let top_products = sqlx::query_as::<_, CategoryTopProductRow>(
            r"
            SELECT p.pid, p.name, primary_picture.image_link,
                   default_variant.sku, COALESCE(stock.total_stock, 0)::int4 AS stock_quantity
            FROM products p
            LEFT JOIN LATERAL (
                SELECT pic.image_link
                FROM pictures pic
                WHERE pic.product_id = p.id AND pic.variant_id IS NULL
                ORDER BY pic.display_order NULLS LAST, pic.id
                LIMIT 1
            ) primary_picture ON TRUE
            LEFT JOIN LATERAL (
                SELECT v.sku
                FROM product_variants v
                WHERE v.product_id = p.id AND v.is_default = TRUE AND v.deleted_at IS NULL
                ORDER BY v.id
                LIMIT 1
            ) default_variant ON TRUE
            LEFT JOIN LATERAL (
                SELECT SUM(v.stock_quantity) AS total_stock
                FROM product_variants v
                WHERE v.product_id = p.id AND v.deleted_at IS NULL
            ) stock ON TRUE
            WHERE p.category_id = $1 AND p.deleted_at IS NULL
            ORDER BY stock.total_stock DESC NULLS LAST, p.created_at DESC
            LIMIT 5
            ",
        )
        .bind(category.id())
        .fetch_all(&mut *txn)
        .await?;
        txn.commit().await?;

        Ok(CategoryDetailResponse {
            category: CategoryResponse::new(&category, product_count, parent_name.as_deref()),
            children: children
                .into_iter()
                .map(|child| CategoryChildResponse {
                    id: child.id,
                    pid: child.pid,
                    name: child.name,
                    slug: child.slug,
                    image_link: child.image_link,
                    product_count: child.product_count,
                })
                .collect(),
            attributes: attributes
                .into_iter()
                .map(|attribute| CategoryAttributeResponse {
                    id: attribute.id,
                    pid: attribute.pid,
                    name: attribute.name,
                    description: attribute.description,
                })
                .collect(),
            top_products: top_products
                .into_iter()
                .map(|product| CategoryTopProductResponse {
                    pid: product.pid,
                    name: product.name,
                    image_link: product.image_link,
                    sku: product.sku,
                    stock_quantity: product.stock_quantity,
                })
                .collect(),
            total_variants,
            total_stock,
        })
    }

    /// Replaces the many-to-many attribute links for a category.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when the category does not exist,
    /// or a database error when links cannot be updated.
    pub async fn set_attributes(
        db: &PgPool,
        pid: Uuid,
        params: &CategoryAttributesInput,
    ) -> ModelResult<CategoryDetailResponse> {
        let mut txn = db.begin().await?;
        let category_id = sqlx::query_scalar::<_, i32>("SELECT id FROM categories WHERE pid = $1")
            .bind(pid)
            .fetch_optional(&mut *txn)
            .await?
            .ok_or(ModelError::EntityNotFound)?;

        sqlx::query("DELETE FROM category_attributes WHERE category_id = $1")
            .bind(category_id)
            .execute(&mut *txn)
            .await?;
        sqlx::query(
            r"
            INSERT INTO category_attributes (category_id, attribute_id)
            SELECT $1, a.id
            FROM attributes a
            WHERE a.pid = ANY($2::uuid[])
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(category_id)
        .bind(params.attribute_pids())
        .execute(&mut *txn)
        .await?;
        txn.commit().await?;

        Self::find_detail_by_pid(db, pid).await
    }

    /// Finds a category by normalized name.
    ///
    /// # Errors
    ///
    /// Returns a database error if the lookup fails.
    pub async fn find_by_name<'e, E>(db: E, name: &str) -> ModelResult<Option<Self>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM categories WHERE name = $1
        ",
        )
        .bind(name.to_lowercase().trim())
        .fetch_optional(db)
        .await
        .map_err(Into::into)
    }

    /// Soft deletes a category by public ID.
    ///
    /// The row is retained and `deleted_at` is set to the current database
    /// timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::EntityNotFound`] when no category exists for
    /// `pid`, and [`ModelError::CategoryHasProducts`] when active products
    /// still belong to the category. Returns a database error if the update
    /// fails.
    pub async fn delete(db: &PgPool, pid: Uuid) -> ModelResult<Self> {
        let mut txn = db.begin().await?;
        let category = sqlx::query_as::<_, Self>(
            r"
            SELECT *
            FROM categories
            WHERE pid = $1
            FOR UPDATE
        ",
        )
        .bind(pid)
        .fetch_optional(&mut *txn)
        .await?
        .ok_or(ModelError::EntityNotFound)?;

        let has_products = sqlx::query_scalar::<_, bool>(
            r"
            SELECT EXISTS(
                SELECT 1
                FROM products
                WHERE category_id = $1 AND deleted_at IS NULL
            )
            ",
        )
        .bind(category.id())
        .fetch_one(&mut *txn)
        .await?;

        if has_products {
            return Err(ModelError::CategoryHasProducts);
        }

        let category = sqlx::query_as::<_, Self>(
            r"
            UPDATE categories
            SET deleted_at = NOW()
            WHERE id = $1
            RETURNING *
            ",
        )
        .bind(category.id())
        .fetch_one(&mut *txn)
        .await?;
        txn.commit().await?;

        Ok(category)
    }

    /// Seeds categories from a file in `src/data`.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::FileNotFound`] when the file is missing,
    /// [`ModelError::UnsupportedFileType`] when the extension is not supported,
    /// deserialization errors for invalid seed data, or database errors when
    /// inserting the loaded permissions fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }

    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    #[must_use]
    pub fn image_link(&self) -> &str {
        &self.image_link
    }

    #[must_use]
    pub const fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }

    #[must_use]
    pub const fn deleted_at(&self) -> Option<DateTime<FixedOffset>> {
        self.deleted_at
    }

    #[must_use]
    pub const fn parent_id(&self) -> Option<i32> {
        self.parent_id
    }
}

impl CategoryAttributeLink {
    /// Seeds category-to-attribute assignments from a file in `src/data`.
    ///
    /// # Errors
    ///
    /// Returns a file, deserialisation, or database error if loading or
    /// inserting the loaded assignments fails.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;

        Self::seed(db, &data).await
    }
}

impl Seedable for CategoryAttributeLink {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for link in data {
            sqlx::query(
                r"
                INSERT INTO category_attributes (category_id, attribute_id)
                VALUES ($1, $2)
                ON CONFLICT (category_id, attribute_id) DO NOTHING
                ",
            )
            .bind(link.category_id)
            .bind(link.attribute_id)
            .execute(db)
            .await?;
        }

        Ok(())
    }
}

impl Seedable for Category {
    async fn seed(db: &sqlx::PgPool, data: &[Self]) -> super::ModelResult<()> {
        for category in data {
            sqlx::query(
                r"
                INSERT INTO categories (
                    id,
                    pid,
                    name,
                    slug,
                    image_link,
                    description,
                    parent_id,
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
                   $8,
                   $9,
                   $10
                ) ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    name = EXCLUDED.name,
                    slug = EXCLUDED.slug,
                    image_link = EXCLUDED.image_link,
                    description = EXCLUDED.description,
                    parent_id = EXCLUDED.parent_id,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at,
                    deleted_at = EXCLUDED.deleted_at
            ",
            )
            .bind(category.id())
            .bind(category.pid())
            .bind(category.name().to_lowercase().trim())
            .bind(category.slug().to_lowercase())
            .bind(category.image_link())
            .bind(category.description())
            .bind(category.parent_id())
            .bind(category.created_at())
            .bind(category.updated_at())
            .bind(category.deleted_at())
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
