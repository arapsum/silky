use std::collections::{HashMap, HashSet};

use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

use crate::schemas::NewOrder;

use super::{
    ModelError, ModelResult, Seedable,
    order_items::{CheckoutItem, OrderItem},
};

const ORDER_CURRENCY: &str = "USD";

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
pub struct Order {
    id: i32,
    pid: Uuid,
    order_number: i64,
    customer_id: i32,
    billing_address_id: Option<i32>,
    shipping_address_id: Option<i32>,
    customer_name: String,
    customer_email: String,
    billing_address_snapshot: Json<JsonValue>,
    shipping_address_snapshot: Json<JsonValue>,
    status: String,
    payment_status: String,
    fulfillment_status: String,
    currency: String,
    subtotal: Decimal,
    discount_total: Decimal,
    shipping_total: Decimal,
    tax_total: Decimal,
    grand_total: Decimal,
    customer_note: Option<String>,
    staff_note: Option<String>,
    placed_at: Option<DateTime<FixedOffset>>,
    cancelled_at: Option<DateTime<FixedOffset>>,
    completed_at: Option<DateTime<FixedOffset>>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl Order {
    /// Checks out an order and reserves its inventory in one transaction.
    ///
    /// Optional billing and shipping address references must belong to the
    /// customer. Their values are copied into JSON snapshots so later address
    /// edits do not change the order history. Product prices and totals are
    /// loaded and calculated on the server. Every item and its corresponding
    /// stock deduction is committed together with the order.
    ///
    /// # Errors
    /// Returns an invalid-reference error for missing customers, addresses, or
    /// variants; an invalid-input error for empty, duplicate, or unavailable
    /// items; or a database error when checkout cannot be committed.
    #[allow(clippy::too_many_lines)]
    pub async fn create(db: &PgPool, params: &NewOrder) -> ModelResult<Self> {
        validate_checkout_items(params)?;
        let mut txn = db.begin().await?;
        let (customer_id, customer_name, customer_email) =
            sqlx::query_as::<_, (i32, String, String)>(
                r"SELECT id, name, email::text
                  FROM users
                  WHERE pid = $1
                    AND deleted_at IS NULL",
            )
            .bind(params.customer_pid())
            .fetch_optional(&mut *txn)
            .await?
            .ok_or(ModelError::CustomerNotFound)?;

        let billing = load_customer_address(
            &mut txn,
            customer_id,
            params.billing_address_pid(),
            "billingAddressPid",
        )
        .await?;
        let shipping = load_customer_address(
            &mut txn,
            customer_id,
            params.shipping_address_pid(),
            "shippingAddressPid",
        )
        .await?;

        let mut variants = load_checkout_variants(&mut txn, params).await?;
        let subtotal = calculate_subtotal(&variants, params)?;
        let order = sqlx::query_as::<_, Self>(
            r"INSERT INTO orders (
                customer_id,
                billing_address_id,
                shipping_address_id,
                billing_address_snapshot,
                shipping_address_snapshot,
                customer_name,
                customer_email,
                currency,
                subtotal,
                discount_total,
                shipping_total,
                tax_total,
                grand_total,
                customer_note,
                staff_note,
                placed_at
              )
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, now())
              RETURNING *",
        )
        .bind(customer_id)
        .bind(billing.as_ref().map(|value| value.0))
        .bind(shipping.as_ref().map(|value| value.0))
        .bind(Json(
            billing.map_or_else(|| serde_json::json!({}), |value| value.1),
        ))
        .bind(Json(
            shipping.map_or_else(|| serde_json::json!({}), |value| value.1),
        ))
        .bind(customer_name)
        .bind(customer_email)
        .bind(ORDER_CURRENCY)
        .bind(subtotal)
        .bind(Decimal::ZERO)
        .bind(Decimal::ZERO)
        .bind(Decimal::ZERO)
        .bind(subtotal)
        .bind(params.customer_note().map(str::trim))
        .bind(params.staff_note().map(str::trim))
        .fetch_one(&mut *txn)
        .await?;

        for requested_item in params.items() {
            let variant = variants
                .remove(&requested_item.variant_pid())
                .ok_or(ModelError::ProductVariantUnavailable)?;
            reserve_inventory(
                &mut txn,
                variant.variant_id,
                variant.variant_pid,
                requested_item.quantity(),
                variant.stock_quantity,
            )
            .await?;
            let line_total = variant.price * Decimal::from(requested_item.quantity());
            OrderItem::insert(
                &mut txn,
                CheckoutItem {
                    order_id: order.id,
                    product_id: variant.product_id,
                    variant_id: variant.variant_id,
                    product_pid: variant.product_pid,
                    variant_pid: variant.variant_pid,
                    product_name: variant.product_name,
                    sku: variant.sku,
                    selected_options: variant.selected_options,
                    quantity: requested_item.quantity(),
                    unit_price: variant.price,
                    line_total,
                },
            )
            .await?;
        }

        txn.commit().await?;
        Ok(order)
    }

    /// Finds an order by its public ID, including its persisted snapshots.
    ///
    /// # Errors
    /// Returns [`ModelError::EntityNotFound`] when no order exists for `pid`.
    pub async fn find_by_pid(db: &PgPool, pid: Uuid) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"SELECT *
              FROM orders
              WHERE pid = $1",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?
        .ok_or(ModelError::EntityNotFound)
    }

    /// Loads orders from a data file and seeds them into the database.
    ///
    /// The file path is resolved relative to `src/data` and must contain a
    /// JSON or YAML array matching [`Order`]. Existing rows are updated by
    /// their internal ID so the operation can be repeated safely.
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
    pub const fn order_number(&self) -> i64 {
        self.order_number
    }
    #[must_use]
    pub fn display_number(&self) -> String {
        format!("ORD-{:06}", self.order_number)
    }
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }
}

impl Seedable for Order {
    #[allow(clippy::too_many_lines)]
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()> {
        for order in data {
            sqlx::query(
                r"
                INSERT INTO orders (
                    id,
                    pid,
                    order_number,
                    customer_id,
                    customer_name,
                    customer_email,
                    billing_address_id,
                    shipping_address_id,
                    billing_address_snapshot,
                    shipping_address_snapshot,
                    status,
                    payment_status,
                    fulfillment_status,
                    currency,
                    subtotal,
                    discount_total,
                    shipping_total,
                    tax_total,
                    grand_total,
                    customer_note,
                    staff_note,
                    placed_at,
                    cancelled_at,
                    completed_at,
                    created_at,
                    updated_at
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, $23, $24, $25, $26
                )
                ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    order_number = EXCLUDED.order_number,
                    customer_id = EXCLUDED.customer_id,
                    customer_name = EXCLUDED.customer_name,
                    customer_email = EXCLUDED.customer_email,
                    billing_address_id = EXCLUDED.billing_address_id,
                    shipping_address_id = EXCLUDED.shipping_address_id,
                    billing_address_snapshot = EXCLUDED.billing_address_snapshot,
                    shipping_address_snapshot = EXCLUDED.shipping_address_snapshot,
                    status = EXCLUDED.status,
                    payment_status = EXCLUDED.payment_status,
                    fulfillment_status = EXCLUDED.fulfillment_status,
                    currency = EXCLUDED.currency,
                    subtotal = EXCLUDED.subtotal,
                    discount_total = EXCLUDED.discount_total,
                    shipping_total = EXCLUDED.shipping_total,
                    tax_total = EXCLUDED.tax_total,
                    grand_total = EXCLUDED.grand_total,
                    customer_note = EXCLUDED.customer_note,
                    staff_note = EXCLUDED.staff_note,
                    placed_at = EXCLUDED.placed_at,
                    cancelled_at = EXCLUDED.cancelled_at,
                    completed_at = EXCLUDED.completed_at,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at
                ",
            )
            .bind(order.id)
            .bind(order.pid)
            .bind(order.order_number)
            .bind(order.customer_id)
            .bind(&order.customer_name)
            .bind(&order.customer_email)
            .bind(order.billing_address_id)
            .bind(order.shipping_address_id)
            .bind(&order.billing_address_snapshot)
            .bind(&order.shipping_address_snapshot)
            .bind(&order.status)
            .bind(&order.payment_status)
            .bind(&order.fulfillment_status)
            .bind(&order.currency)
            .bind(order.subtotal)
            .bind(order.discount_total)
            .bind(order.shipping_total)
            .bind(order.tax_total)
            .bind(order.grand_total)
            .bind(order.customer_note.as_deref())
            .bind(order.staff_note.as_deref())
            .bind(order.placed_at)
            .bind(order.cancelled_at)
            .bind(order.completed_at)
            .bind(order.created_at)
            .bind(order.updated_at)
            .execute(db)
            .await?;
        }

        sqlx::query(
            r"SELECT
                    setval(
                        pg_get_serial_sequence('orders', 'id'),
                        COALESCE((SELECT MAX(id) FROM orders), 1),
                        (SELECT COUNT(*) > 0 FROM orders)
                    ),
                    setval(
                        pg_get_serial_sequence('orders', 'order_number'),
                        COALESCE((SELECT MAX(order_number) FROM orders), 1),
                        (SELECT COUNT(*) > 0 FROM orders)
                    )",
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

fn validate_checkout_items(params: &NewOrder) -> ModelResult<()> {
    if params.items().is_empty() {
        return Err(ModelError::OrderHasNoItems);
    }

    let mut variants = HashSet::with_capacity(params.items().len());
    for item in params.items() {
        if item.quantity() < 1 {
            return Err(ModelError::InvalidOrderItemQuantity);
        }
        if !variants.insert(item.variant_pid()) {
            return Err(ModelError::DuplicateOrderItem);
        }
    }

    Ok(())
}

async fn load_checkout_variants(
    txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &NewOrder,
) -> ModelResult<HashMap<Uuid, VariantSnapshot>> {
    let variant_pids = params
        .items()
        .iter()
        .map(crate::schemas::NewOrderItem::variant_pid)
        .collect::<Vec<_>>();
    let variants = sqlx::query_as::<_, VariantSnapshot>(
        r"SELECT v.id AS variant_id,
                v.pid AS variant_pid,
                v.stock_quantity,
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
          WHERE v.pid = ANY($1)
            AND v.deleted_at IS NULL
            AND p.deleted_at IS NULL
          ORDER BY v.id
          FOR UPDATE OF v",
    )
    .bind(&variant_pids)
    .fetch_all(&mut **txn)
    .await?;

    if variants.len() != variant_pids.len() {
        return Err(ModelError::ProductVariantUnavailable);
    }

    Ok(variants
        .into_iter()
        .map(|variant| (variant.variant_pid, variant))
        .collect())
}

fn calculate_subtotal(
    variants: &HashMap<Uuid, VariantSnapshot>,
    params: &NewOrder,
) -> ModelResult<Decimal> {
    params
        .items()
        .iter()
        .try_fold(Decimal::ZERO, |total, item| {
            let variant = variants
                .get(&item.variant_pid())
                .ok_or(ModelError::ProductVariantUnavailable)?;
            if variant.stock_quantity < item.quantity() {
                return Err(ModelError::InsufficientStock {
                    variant_pid: item.variant_pid(),
                    requested: item.quantity(),
                    available: variant.stock_quantity,
                });
            }
            Ok(total + variant.price * Decimal::from(item.quantity()))
        })
}

async fn reserve_inventory(
    txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    variant_row_id: i32,
    public_variant_id: Uuid,
    quantity: i32,
    available: i32,
) -> ModelResult<()> {
    let result = sqlx::query(
        r"UPDATE product_variants
          SET stock_quantity = stock_quantity - $1
          WHERE id = $2
            AND stock_quantity >= $1
            AND deleted_at IS NULL",
    )
    .bind(quantity)
    .bind(variant_row_id)
    .execute(&mut **txn)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ModelError::InsufficientStock {
            variant_pid: public_variant_id,
            requested: quantity,
            available,
        });
    }
    Ok(())
}

async fn load_customer_address(
    txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    customer_id: i32,
    pid: Option<Uuid>,
    field: &'static str,
) -> ModelResult<Option<(i32, JsonValue)>> {
    let Some(pid) = pid else {
        return Ok(None);
    };
    sqlx::query_as::<_, (i32, JsonValue)>(
        r"SELECT id,
                jsonb_build_object(
                    'pid', pid,
                    'addressType', address_type,
                    'label', label,
                    'recipientName', recipient_name,
                    'company', company,
                    'lineOne', line_one,
                    'lineTwo', line_two,
                    'city', city,
                    'region', region,
                    'postalCode', postal_code,
                    'countryCode', country_code,
                    'email', email,
                    'phone', phone
                )
              FROM addresses
              WHERE pid = $1
                AND customer_id = $2
                AND deleted_at IS NULL",
    )
    .bind(pid)
    .bind(customer_id)
    .fetch_optional(&mut **txn)
    .await?
    .map_or_else(
        || Err(ModelError::CustomerAddressNotFound { field }),
        |value| Ok(Some(value)),
    )
}

#[derive(Debug, FromRow)]
struct VariantSnapshot {
    variant_id: i32,
    variant_pid: Uuid,
    stock_quantity: i32,
    product_id: i32,
    product_pid: Uuid,
    product_name: String,
    sku: String,
    price: Decimal,
    selected_options: Json<JsonValue>,
}
