use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

use crate::schemas::NewOrder;

use super::{ModelError, ModelResult, Seedable};

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
    /// Creates a pending order for an existing customer.
    ///
    /// Optional billing and shipping address references must belong to the
    /// customer. Their values are copied into JSON snapshots so later address
    /// edits do not change the order history. The amount breakdown is checked
    /// before insertion and the order is timestamped as placed.
    ///
    /// # Errors
    /// Returns an invalid-reference error for missing customers or addresses,
    /// an invalid-input error for inconsistent totals, or a database error.
    pub async fn create(db: &PgPool, params: &NewOrder) -> ModelResult<Self> {
        validate_totals(params)?;
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
            .ok_or_else(|| ModelError::InvalidReference("Customer does not exist.".to_string()))?;

        let billing =
            load_customer_address(&mut txn, customer_id, params.billing_address_pid()).await?;
        let shipping =
            load_customer_address(&mut txn, customer_id, params.shipping_address_pid()).await?;
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
        .bind(params.currency().to_uppercase())
        .bind(params.subtotal())
        .bind(params.discount_total())
        .bind(params.shipping_total())
        .bind(params.tax_total())
        .bind(params.grand_total())
        .bind(params.customer_note().map(str::trim))
        .bind(params.staff_note().map(str::trim))
        .fetch_one(&mut *txn)
        .await?;
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

fn validate_totals(params: &NewOrder) -> ModelResult<()> {
    let amounts = [
        params.subtotal(),
        params.discount_total(),
        params.shipping_total(),
        params.tax_total(),
        params.grand_total(),
    ];
    if amounts.iter().any(Decimal::is_sign_negative) {
        return Err(ModelError::InvalidInput(
            "Order amounts cannot be negative.".to_string(),
        ));
    }
    let expected =
        params.subtotal() - params.discount_total() + params.shipping_total() + params.tax_total();
    if expected != params.grand_total() {
        return Err(ModelError::InvalidInput(
            "Order total does not match its breakdown.".to_string(),
        ));
    }
    Ok(())
}

async fn load_customer_address(
    txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    customer_id: i32,
    pid: Option<Uuid>,
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
        || {
            Err(ModelError::InvalidReference(
                "Customer address does not exist.".to_string(),
            ))
        },
        |value| Ok(Some(value)),
    )
}
