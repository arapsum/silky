use std::collections::{HashMap, HashSet};

use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

use crate::schemas::{CheckoutOrder, NewOrder, NewOrderItem, OrderListQuery, UpdateOrder};

use super::{
    ModelError, ModelResult, PaginatedModel, Pagination, Seedable,
    order_items::{CheckoutItem, OrderItem},
    payment_attempts::PaymentAttempt,
};

const ORDER_CURRENCY: &str = "USD";

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
pub struct Order {
    id: i32,
    pid: Uuid,
    order_number: i64,
    #[serde(skip_serializing)]
    checkout_key: Option<Uuid>,
    customer_id: i32,
    billing_address_id: Option<i32>,
    shipping_address_id: Option<i32>,
    customer_name: String,
    customer_email: String,
    #[sqlx(default)]
    customer_image: Option<String>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderWithItems {
    pub order: Order,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Clone)]
pub struct CheckoutOrderState {
    pub order: OrderWithItems,
    pub attempt: PaymentAttempt,
    pub created: bool,
}

struct CheckoutOrderInput<'a> {
    customer_pid: Uuid,
    checkout_key: Option<Uuid>,
    billing_address_pid: Option<Uuid>,
    shipping_address_pid: Option<Uuid>,
    items: &'a [NewOrderItem],
    customer_note: Option<&'a str>,
    staff_note: Option<&'a str>,
}

impl<'a> CheckoutOrderInput<'a> {
    fn from_new_order(params: &'a NewOrder) -> Self {
        Self {
            customer_pid: params.customer_pid(),
            checkout_key: None,
            billing_address_pid: params.billing_address_pid(),
            shipping_address_pid: params.shipping_address_pid(),
            items: params.items(),
            customer_note: params.customer_note(),
            staff_note: params.staff_note(),
        }
    }

    fn from_checkout(customer_pid: Uuid, params: &'a CheckoutOrder) -> Self {
        let shipping_address_pid = params.shipping_address_pid();
        Self {
            customer_pid,
            checkout_key: Some(params.checkout_key()),
            billing_address_pid: Some(params.billing_address_pid().unwrap_or(shipping_address_pid)),
            shipping_address_pid: Some(shipping_address_pid),
            items: params.items(),
            customer_note: params.customer_note(),
            staff_note: None,
        }
    }
}

impl Order {
    /// Creates a checkout once for a customer-supplied idempotency key.
    ///
    /// Concurrent requests using the same customer and key are serialized by
    /// a transaction-scoped advisory lock. A retry receives the original
    /// order and active payment attempt without reserving stock again.
    ///
    /// # Errors
    /// Returns the normal checkout validation errors, or an invalid payment
    /// state when the keyed checkout is no longer active.
    pub async fn prepare_checkout(
        db: &PgPool,
        customer_pid: Uuid,
        params: &CheckoutOrder,
    ) -> ModelResult<CheckoutOrderState> {
        let mut txn = db.begin().await?;
        let lock_key = format!("{customer_pid}:{}", params.checkout_key());
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(lock_key)
            .execute(&mut *txn)
            .await?;

        let existing = sqlx::query_as::<_, Self>(
            r"SELECT orders.*, customer.image AS customer_image
              FROM orders AS orders
              JOIN users AS customer ON customer.id = orders.customer_id
              WHERE customer.pid = $1
                AND orders.checkout_key = $2",
        )
        .bind(customer_pid)
        .bind(params.checkout_key())
        .fetch_optional(&mut *txn)
        .await?;

        let state = if let Some(order) = existing {
            let items = OrderItem::find_by_order_id(&mut txn, order.id).await?;
            let attempt = PaymentAttempt::find_active_for_order(&mut txn, order.id).await?;
            CheckoutOrderState {
                order: OrderWithItems { order, items },
                attempt,
                created: false,
            }
        } else {
            let input = CheckoutOrderInput::from_checkout(customer_pid, params);
            let order = Self::insert_checkout_order(&mut txn, input).await?;
            let attempt = PaymentAttempt::create_for_order(
                &mut txn,
                order.order.row_id(),
                order.order.grand_total(),
                order.order.currency(),
            )
            .await?;
            CheckoutOrderState {
                order,
                attempt,
                created: true,
            }
        };

        txn.commit().await?;
        Ok(state)
    }

    /// Checks out an order and reserves its inventory in one transaction.
    ///
    /// Any supplied billing and shipping address references must belong to the
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
        let mut txn = db.begin().await?;
        let input = CheckoutOrderInput::from_new_order(params);
        let created = Self::insert_checkout_order(&mut txn, input).await?;
        txn.commit().await?;
        Ok(created.order)
    }

    /// Creates a customer-owned order inside a caller-managed transaction.
    ///
    /// Customer identity is supplied separately from the public payload so it
    /// can only come from authenticated claims. The caller can create a payment
    /// attempt in the same transaction before contacting Stripe.
    ///
    /// # Errors
    /// Returns an invalid-reference error for missing customers, addresses, or
    /// variants; an invalid-input error for empty, duplicate, or unavailable
    /// items; or a database error when checkout rows cannot be written.
    #[allow(clippy::too_many_lines)]
    pub async fn create_checkout_order(
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        customer_pid: Uuid,
        params: &CheckoutOrder,
    ) -> ModelResult<OrderWithItems> {
        let input = CheckoutOrderInput::from_checkout(customer_pid, params);
        Self::insert_checkout_order(txn, input).await
    }

    #[allow(clippy::too_many_lines)]
    async fn insert_checkout_order(
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        input: CheckoutOrderInput<'_>,
    ) -> ModelResult<OrderWithItems> {
        validate_checkout_items(input.items)?;
        let (customer_id, customer_name, customer_email) =
            sqlx::query_as::<_, (i32, String, String)>(
                r"SELECT id, name, email::text
                  FROM users
                  WHERE pid = $1
                    AND deleted_at IS NULL",
            )
            .bind(input.customer_pid)
            .fetch_optional(&mut **txn)
            .await?
            .ok_or(ModelError::CustomerNotFound)?;

        let billing = load_customer_address(
            txn,
            customer_id,
            input.billing_address_pid,
            "billingAddressPid",
        )
        .await?;
        let shipping = load_customer_address(
            txn,
            customer_id,
            input.shipping_address_pid,
            "shippingAddressPid",
        )
        .await?;

        let mut variants = load_checkout_variants(txn, input.items).await?;
        let subtotal = calculate_subtotal(&variants, input.items)?;
        let order = sqlx::query_as::<_, Self>(
            r"INSERT INTO orders (
                customer_id,
                checkout_key,
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
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, now())
              RETURNING *,
                  (SELECT image FROM users WHERE id = customer_id) AS customer_image",
        )
        .bind(customer_id)
        .bind(input.checkout_key)
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
        .bind(input.customer_note.map(str::trim))
        .bind(input.staff_note.map(str::trim))
        .fetch_one(&mut **txn)
        .await?;

        let mut order_items = Vec::with_capacity(input.items.len());
        for requested_item in input.items {
            let variant = variants
                .remove(&requested_item.variant_pid())
                .ok_or(ModelError::ProductVariantUnavailable)?;
            reserve_inventory(
                txn,
                variant.variant_id,
                variant.variant_pid,
                requested_item.quantity(),
                variant.stock_quantity,
            )
            .await?;
            let line_total = variant.price * Decimal::from(requested_item.quantity());
            let item = OrderItem::insert(
                txn,
                CheckoutItem {
                    order_id: order.id,
                    product_id: variant.product_id,
                    variant_id: variant.variant_id,
                    product_pid: variant.product_pid,
                    variant_pid: variant.variant_pid,
                    product_slug: variant.product_slug,
                    image_url: variant.image_url,
                    product_name: variant.product_name,
                    sku: variant.sku,
                    selected_options: variant.selected_options,
                    quantity: requested_item.quantity(),
                    unit_price: variant.price,
                    line_total,
                },
            )
            .await?;
            order_items.push(item);
        }

        Ok(OrderWithItems {
            order,
            items: order_items,
        })
    }

    /// Finds an order by its public ID, including its persisted snapshots.
    ///
    /// # Errors
    /// Returns [`ModelError::EntityNotFound`] when no order exists for `pid`.
    pub async fn find_by_pid(db: &PgPool, pid: Uuid) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"SELECT o.*, u.image AS customer_image
              FROM orders o
              JOIN users u ON u.id = o.customer_id
              WHERE o.pid = $1",
        )
        .bind(pid)
        .fetch_optional(db)
        .await?
        .ok_or(ModelError::EntityNotFound)
    }

    /// Lists orders with pagination and optional lifecycle filters.
    ///
    /// Results are ordered newest first. The list contains order headers; use
    /// [`Self::find_detail_by_pid`] when the line items are required.
    ///
    /// # Errors
    /// Returns a database error when the count or page query fails.
    pub async fn find_all(
        db: &PgPool,
        query: &OrderListQuery,
    ) -> ModelResult<PaginatedModel<Self>> {
        let limit = query.limit().unwrap_or(20).clamp(1, 40);
        let page = query.page().unwrap_or(1).max(1);
        let offset = (page - 1) * limit;
        let customer_pid = query.customer_pid();
        let search = query.search().map(|value| format!("%{value}%"));

        let total_items = sqlx::query_scalar::<_, i64>(
            r"SELECT COUNT(*)
              FROM orders o
              LEFT JOIN users u ON u.id = o.customer_id
              WHERE ($1::TEXT IS NULL OR o.status = $1)
                AND ($2::TEXT IS NULL OR o.payment_status = $2)
                AND ($3::TEXT IS NULL OR o.fulfillment_status = $3)
                AND ($4::UUID IS NULL OR u.pid = $4)
                AND ($5::TEXT IS NULL OR o.order_number::TEXT ILIKE $5 OR o.customer_name ILIKE $5 OR o.customer_email ILIKE $5)
                AND ($6::DATE IS NULL OR o.created_at >= $6::DATE)
                AND ($7::DATE IS NULL OR o.created_at < ($7::DATE + INTERVAL '1 day'))",
        )
        .bind(query.status())
        .bind(query.payment_status())
        .bind(query.fulfillment_status())
        .bind(customer_pid)
        .bind(search.as_deref())
        .bind(query.created_from())
        .bind(query.created_to())
        .fetch_one(db)
        .await?;

        let orders = sqlx::query_as::<_, Self>(
            r"SELECT o.*, u.image AS customer_image
              FROM orders o
              LEFT JOIN users u ON u.id = o.customer_id
              WHERE ($3::TEXT IS NULL OR o.status = $3)
                AND ($4::TEXT IS NULL OR o.payment_status = $4)
                AND ($5::TEXT IS NULL OR o.fulfillment_status = $5)
                AND ($6::UUID IS NULL OR u.pid = $6)
                AND ($7::TEXT IS NULL OR o.order_number::TEXT ILIKE $7 OR o.customer_name ILIKE $7 OR o.customer_email ILIKE $7)
                AND ($8::DATE IS NULL OR o.created_at >= $8::DATE)
                AND ($9::DATE IS NULL OR o.created_at < ($9::DATE + INTERVAL '1 day'))
              ORDER BY o.created_at DESC, o.id DESC
              LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .bind(query.status())
        .bind(query.payment_status())
        .bind(query.fulfillment_status())
        .bind(customer_pid)
        .bind(search.as_deref())
        .bind(query.created_from())
        .bind(query.created_to())
        .fetch_all(db)
        .await?;

        Ok(PaginatedModel::new(
            orders,
            Pagination::new(page, limit, total_items),
        ))
    }

    /// Fetches an order header together with all of its immutable line snapshots.
    ///
    /// # Errors
    /// Returns [`ModelError::EntityNotFound`] when no order exists for `pid`,
    /// or a database error when its items cannot be loaded.
    pub async fn find_detail_by_pid(db: &PgPool, pid: Uuid) -> ModelResult<OrderWithItems> {
        let order = Self::find_by_pid(db, pid).await?;
        let items = OrderItem::find_by_order(db, pid).await?;
        Ok(OrderWithItems { order, items })
    }

    /// Fetches an order detail only when it belongs to the supplied customer.
    ///
    /// # Errors
    /// Returns [`ModelError::EntityNotFound`] when the order does not belong to
    /// the customer or when the order cannot be found.
    pub async fn find_detail_for_customer(
        db: &PgPool,
        pid: Uuid,
        customer_pid: Uuid,
    ) -> ModelResult<OrderWithItems> {
        let order = sqlx::query_as::<_, Self>(
            r"SELECT o.*, u.image AS customer_image
              FROM orders o
              JOIN users u ON u.id = o.customer_id
              WHERE o.pid = $1 AND u.pid = $2",
        )
        .bind(pid)
        .bind(customer_pid)
        .fetch_optional(db)
        .await?
        .ok_or(ModelError::EntityNotFound)?;
        let items = OrderItem::find_by_order(db, pid).await?;
        Ok(OrderWithItems { order, items })
    }

    /// Updates staff-managed order state and notes.
    ///
    /// # Errors
    /// Returns [`ModelError::EntityNotFound`] when no order exists, or a
    /// database error when a status constraint is violated.
    pub async fn update(db: &PgPool, pid: Uuid, params: &UpdateOrder) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>(
            r"UPDATE orders o
              SET status = COALESCE($2, status),
                  fulfillment_status = COALESCE($3, fulfillment_status),
                  staff_note = COALESCE($4, staff_note),
                  updated_at = now()
              FROM users u
              WHERE o.pid = $1
                AND u.id = o.customer_id
              RETURNING o.*, u.image AS customer_image",
        )
        .bind(pid)
        .bind(params.status())
        .bind(params.fulfillment_status())
        .bind(params.staff_note())
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

    #[must_use]
    pub fn customer_email(&self) -> &str {
        &self.customer_email
    }

    #[must_use]
    pub const fn grand_total(&self) -> Decimal {
        self.grand_total
    }

    #[must_use]
    pub fn currency(&self) -> &str {
        &self.currency
    }

    #[must_use]
    pub const fn row_id(&self) -> i32 {
        self.id
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
                    checkout_key,
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
                    $21, $22, $23, $24, $25, $26, $27
                )
                ON CONFLICT (id) DO UPDATE SET
                    pid = EXCLUDED.pid,
                    order_number = EXCLUDED.order_number,
                    checkout_key = EXCLUDED.checkout_key,
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
            .bind(order.checkout_key)
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

fn validate_checkout_items(items: &[NewOrderItem]) -> ModelResult<()> {
    if items.is_empty() {
        return Err(ModelError::OrderHasNoItems);
    }

    let mut variants = HashSet::with_capacity(items.len());
    for item in items {
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
    items: &[NewOrderItem],
) -> ModelResult<HashMap<Uuid, VariantSnapshot>> {
    let variant_pids = items
        .iter()
        .map(crate::schemas::NewOrderItem::variant_pid)
        .collect::<Vec<_>>();
    let variants = sqlx::query_as::<_, VariantSnapshot>(
        r"SELECT v.id AS variant_id,
                v.pid AS variant_pid,
                v.stock_quantity,
                p.id AS product_id,
                p.pid AS product_pid,
                p.slug AS product_slug,
                p.name AS product_name,
                v.sku,
                v.price,
                (
                    SELECT picture.image_link
                    FROM pictures AS picture
                    WHERE picture.product_id = p.id
                      AND (picture.variant_id IS NULL OR picture.variant_id = v.id)
                    ORDER BY (picture.variant_id = v.id) DESC,
                             picture.display_order NULLS LAST,
                             picture.id
                    LIMIT 1
                ) AS image_url,
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
    items: &[NewOrderItem],
) -> ModelResult<Decimal> {
    items.iter().try_fold(Decimal::ZERO, |total, item| {
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
    product_slug: String,
    product_name: String,
    sku: String,
    price: Decimal,
    image_url: Option<String>,
    selected_options: Json<JsonValue>,
}
