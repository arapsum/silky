use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{FromRow, PgPool, Postgres, Transaction, types::Json};
use uuid::Uuid;

use super::{ModelError, ModelResult, Seedable};

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrderItem {
    id: i32,
    pid: Uuid,
    order_id: i32,
    product_id: Option<i32>,
    variant_id: Option<i32>,
    product_pid: Uuid,
    variant_pid: Uuid,
    product_slug: Option<String>,
    image_url: Option<String>,
    product_name: String,
    sku: String,
    selected_options: Json<JsonValue>,
    quantity: i32,
    unit_price: Decimal,
    discount_total: Decimal,
    tax_total: Decimal,
    line_total: Decimal,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl OrderItem {
    pub(super) async fn find_by_order_id(
        txn: &mut Transaction<'_, Postgres>,
        order_id: i32,
    ) -> ModelResult<Vec<Self>> {
        Ok(sqlx::query_as::<_, Self>(
            r"SELECT *
              FROM order_items
              WHERE order_id = $1
              ORDER BY id",
        )
        .bind(order_id)
        .fetch_all(&mut **txn)
        .await?)
    }

    /// Inserts a server-calculated item as part of an open checkout transaction.
    ///
    /// This function is restricted to the order repository so an item cannot
    /// be appended independently after an order has been placed.
    ///
    /// # Errors
    /// Returns an invalid-input error when the order is no longer pending, or
    /// a database error when the item cannot be inserted.
    pub(super) async fn insert(
        txn: &mut Transaction<'_, Postgres>,
        item: CheckoutItem,
    ) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"INSERT INTO order_items (
                order_id,
                product_id,
                variant_id,
                product_pid,
                variant_pid,
                product_slug,
                image_url,
                product_name,
                sku,
                selected_options,
                quantity,
                unit_price,
                discount_total,
                tax_total,
                line_total
              )
              SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 0, 0, $13
              FROM orders
              WHERE id = $1
                AND status = 'pending'
              RETURNING *",
        )
        .bind(item.order_id)
        .bind(item.product_id)
        .bind(item.variant_id)
        .bind(item.product_pid)
        .bind(item.variant_pid)
        .bind(item.product_slug)
        .bind(item.image_url)
        .bind(item.product_name)
        .bind(item.sku)
        .bind(item.selected_options)
        .bind(item.quantity)
        .bind(item.unit_price)
        .bind(item.line_total)
        .fetch_optional(&mut **txn)
        .await?
        .ok_or(ModelError::OrderNotEditable)
    }

    /// Lists order lines by order public ID in insertion order.
    ///
    /// # Errors
    /// Returns an invalid-reference error when the order does not exist or a database error.
    pub async fn find_by_order(db: &PgPool, order_pid: Uuid) -> ModelResult<Vec<Self>> {
        let order_exists = sqlx::query_scalar::<_, bool>(
            r"SELECT EXISTS (
                        SELECT 1
                        FROM orders
                        WHERE pid = $1
                    )",
        )
        .bind(order_pid)
        .fetch_one(db)
        .await?;
        if !order_exists {
            return Err(ModelError::EntityNotFound);
        }
        Ok(sqlx::query_as::<_, Self>(
            r"SELECT oi.*
              FROM order_items oi
              JOIN orders o ON o.id = oi.order_id
              WHERE o.pid = $1
              ORDER BY oi.id",
        )
        .bind(order_pid)
        .fetch_all(db)
        .await?)
    }

    /// Loads order items from a data file and seeds them into the database.
    ///
    /// The file path is resolved relative to `src/data` and must contain a
    /// JSON or YAML array matching [`OrderItem`]. Existing rows are updated
    /// by their internal ID so the operation can be repeated safely.
    ///
    /// # Errors
    /// Returns a file, deserialization, or database error when the seed cannot
    /// be loaded or persisted.
    pub async fn seed_data(db: &PgPool, file: &str) -> ModelResult<()> {
        let data = Self::load(file).await?;
        Self::seed(db, &data).await
    }

    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
    }
    #[must_use]
    pub fn sku(&self) -> &str {
        &self.sku
    }
    #[must_use]
    pub const fn quantity(&self) -> i32 {
        self.quantity
    }

    #[must_use]
    pub fn product_name(&self) -> &str {
        &self.product_name
    }

    #[must_use]
    pub const fn variant_pid(&self) -> Uuid {
        self.variant_pid
    }

    #[must_use]
    pub const fn unit_price(&self) -> Decimal {
        self.unit_price
    }
}

pub(super) struct CheckoutItem {
    pub order_id: i32,
    pub product_id: i32,
    pub variant_id: i32,
    pub product_pid: Uuid,
    pub variant_pid: Uuid,
    pub product_slug: String,
    pub image_url: Option<String>,
    pub product_name: String,
    pub sku: String,
    pub selected_options: Json<JsonValue>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub line_total: Decimal,
}

impl Seedable for OrderItem {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for item in data {
            sqlx::query(
                r"
                INSERT INTO order_items (
                    id,
                    pid,
                    order_id,
                    product_id,
                    variant_id,
                    product_pid,
                    variant_pid,
                    product_slug,
                    image_url,
                    product_name,
                    sku,
                    selected_options,
                    quantity,
                    unit_price,
                    discount_total,
                    tax_total,
                    line_total,
                    created_at,
                    updated_at
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19
                )
                ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    order_id = EXCLUDED.order_id,
                    product_id = EXCLUDED.product_id,
                    variant_id = EXCLUDED.variant_id,
                    product_pid = EXCLUDED.product_pid,
                    variant_pid = EXCLUDED.variant_pid,
                    product_slug = EXCLUDED.product_slug,
                    image_url = EXCLUDED.image_url,
                    product_name = EXCLUDED.product_name,
                    sku = EXCLUDED.sku,
                    selected_options = EXCLUDED.selected_options,
                    quantity = EXCLUDED.quantity,
                    unit_price = EXCLUDED.unit_price,
                    discount_total = EXCLUDED.discount_total,
                    tax_total = EXCLUDED.tax_total,
                    line_total = EXCLUDED.line_total,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at
                ",
            )
            .bind(item.id)
            .bind(item.pid)
            .bind(item.order_id)
            .bind(item.product_id)
            .bind(item.variant_id)
            .bind(item.product_pid)
            .bind(item.variant_pid)
            .bind(item.product_slug.as_deref())
            .bind(item.image_url.as_deref())
            .bind(&item.product_name)
            .bind(&item.sku)
            .bind(&item.selected_options)
            .bind(item.quantity)
            .bind(item.unit_price)
            .bind(item.discount_total)
            .bind(item.tax_total)
            .bind(item.line_total)
            .bind(item.created_at)
            .bind(item.updated_at)
            .execute(db)
            .await?;
        }

        sqlx::query(
            r"SELECT setval(
                    pg_get_serial_sequence('order_items', 'id'),
                    COALESCE((SELECT MAX(id) FROM order_items), 1),
                    (SELECT COUNT(*) > 0 FROM order_items)
                )",
        )
        .execute(db)
        .await?;

        Ok(())
    }
}
