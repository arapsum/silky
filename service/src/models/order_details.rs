use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

use crate::schemas::NewOrderDetail;

use super::{ModelError, ModelResult, Seedable};

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetail {
    id: i32,
    pid: Uuid,
    order_id: i32,
    product_id: Option<i32>,
    variant_id: Option<i32>,
    product_pid: Uuid,
    variant_pid: Uuid,
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

impl OrderDetail {
    /// Creates an immutable order line from the current variant snapshot.
    ///
    /// The product name, SKU, selected options, and unit price are copied at
    /// insertion time; later catalogue edits do not change this line.
    ///
    /// # Errors
    /// Returns an invalid-reference error when the order or variant is missing,
    /// an invalid-input error when the line total is negative, or a database error.
    pub async fn create(db: &PgPool, params: &NewOrderDetail) -> ModelResult<Self> {
        if params.discount_total().is_sign_negative() || params.tax_total().is_sign_negative() {
            return Err(ModelError::InvalidInput(
                "Order line amounts cannot be negative.".to_string(),
            ));
        }
        let mut txn = db.begin().await?;
        let order_id = sqlx::query_scalar::<_, i32>(
            r"SELECT id
              FROM orders
              WHERE pid = $1",
        )
        .bind(params.order_pid())
        .fetch_optional(&mut *txn)
        .await?
        .ok_or_else(|| ModelError::InvalidReference("Order does not exist.".to_string()))?;
        let variant = sqlx::query_as::<_, VariantSnapshot>(
            r"SELECT v.id AS variant_id,
                    v.pid AS variant_pid,
                    p.id AS product_id,
                    p.pid AS product_pid,
                    p.name AS product_name,
                    v.sku,
                    v.price,
                    COALESCE(
                        (
                            SELECT jsonb_object_agg(a.name, av.value)
                            FROM variant_attribute_values vav
                            JOIN attributes a ON a.id = vav.attribute_id
                            JOIN attribute_values av ON av.id = vav.attribute_value_id
                            WHERE vav.variant_id = v.id
                        ),
                        '{}'::jsonb
                    ) AS selected_options
              FROM product_variants v
              JOIN products p ON p.id = v.product_id
              WHERE v.pid = $1
                AND v.deleted_at IS NULL
                AND p.deleted_at IS NULL",
        )
        .bind(params.variant_pid())
        .fetch_optional(&mut *txn)
        .await?
        .ok_or_else(|| {
            ModelError::InvalidReference("Product variant does not exist.".to_string())
        })?;
        let line_total = variant.price * Decimal::from(params.quantity()) - params.discount_total()
            + params.tax_total();
        if line_total.is_sign_negative() {
            return Err(ModelError::InvalidInput(
                "Order line total cannot be negative.".to_string(),
            ));
        }
        let detail = sqlx::query_as::<_, Self>(
            r"INSERT INTO order_details (
                order_id,
                product_id,
                variant_id,
                product_pid,
                variant_pid,
                product_name,
                sku,
                selected_options,
                quantity,
                unit_price,
                discount_total,
                tax_total,
                line_total
              )
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
              RETURNING *",
        )
        .bind(order_id)
        .bind(variant.product_id)
        .bind(variant.variant_id)
        .bind(variant.product_pid)
        .bind(variant.variant_pid)
        .bind(variant.product_name)
        .bind(variant.sku)
        .bind(variant.selected_options)
        .bind(params.quantity())
        .bind(variant.price)
        .bind(params.discount_total())
        .bind(params.tax_total())
        .bind(line_total)
        .fetch_one(&mut *txn)
        .await?;
        txn.commit().await?;
        Ok(detail)
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
            r"SELECT od.*
              FROM order_details od
              JOIN orders o ON o.id = od.order_id
              WHERE o.pid = $1
              ORDER BY od.id",
        )
        .bind(order_pid)
        .fetch_all(db)
        .await?)
    }

    /// Loads order details from a data file and seeds them into the database.
    ///
    /// The file path is resolved relative to `src/data` and must contain a
    /// JSON or YAML array matching [`OrderDetail`]. Existing rows are updated
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
}

impl Seedable for OrderDetail {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for detail in data {
            sqlx::query(
                r"
                INSERT INTO order_details (
                    id,
                    pid,
                    order_id,
                    product_id,
                    variant_id,
                    product_pid,
                    variant_pid,
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
                    $1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10, $11, $12, $13, $14, $15, $16, $17
                )
                ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    order_id = EXCLUDED.order_id,
                    product_id = EXCLUDED.product_id,
                    variant_id = EXCLUDED.variant_id,
                    product_pid = EXCLUDED.product_pid,
                    variant_pid = EXCLUDED.variant_pid,
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
            .bind(detail.id)
            .bind(detail.pid)
            .bind(detail.order_id)
            .bind(detail.product_id)
            .bind(detail.variant_id)
            .bind(detail.product_pid)
            .bind(detail.variant_pid)
            .bind(&detail.product_name)
            .bind(&detail.sku)
            .bind(&detail.selected_options)
            .bind(detail.quantity)
            .bind(detail.unit_price)
            .bind(detail.discount_total)
            .bind(detail.tax_total)
            .bind(detail.line_total)
            .bind(detail.created_at)
            .bind(detail.updated_at)
            .execute(db)
            .await?;
        }

        sqlx::query(
            r"SELECT setval(
                    pg_get_serial_sequence('order_details', 'id'),
                    COALESCE((SELECT MAX(id) FROM order_details), 1),
                    (SELECT COUNT(*) > 0 FROM order_details)
                )",
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct VariantSnapshot {
    variant_id: i32,
    variant_pid: Uuid,
    product_id: i32,
    product_pid: Uuid,
    product_name: String,
    sku: String,
    price: Decimal,
    selected_options: Json<JsonValue>,
}
