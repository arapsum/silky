use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

use crate::schemas::NewOrder;

use super::{ModelError, ModelResult};

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
#[expect(
    dead_code,
    reason = "internal database keys are retained for repository operations"
)]
#[allow(clippy::struct_field_names)]
pub struct Order {
    #[serde(skip)]
    id: i32,
    pid: Uuid,
    order_number: i64,
    #[serde(skip)]
    customer_id: i32,
    #[serde(skip)]
    billing_address_id: Option<i32>,
    #[serde(skip)]
    shipping_address_id: Option<i32>,
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
    /// Creates a pending order for an existing customer and optional addresses.
    ///
    /// Address references must belong to the customer and are copied into order
    /// snapshots so later address edits do not change the order history.
    ///
    /// # Errors
    /// Returns an invalid-reference error for missing customers or addresses,
    /// an invalid-input error for inconsistent totals, or a database error.
    pub async fn create(db: &PgPool, params: &NewOrder) -> ModelResult<Self> {
        validate_totals(params)?;
        let mut txn = db.begin().await?;
        let (customer_id, customer_name, customer_email) =
            sqlx::query_as::<_, (i32, String, String)>(
                "SELECT id,name,email::text FROM users WHERE pid = $1 AND deleted_at IS NULL",
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
                customer_id,billing_address_id,shipping_address_id,billing_address_snapshot,
                shipping_address_snapshot,customer_name,customer_email,currency,subtotal,
                discount_total,shipping_total,tax_total,grand_total,customer_note,staff_note,placed_at
              ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,now()
              ) RETURNING *",
        )
        .bind(customer_id)
        .bind(billing.as_ref().map(|value| value.0))
        .bind(shipping.as_ref().map(|value| value.0))
        .bind(Json(billing.map_or_else(|| serde_json::json!({}), |value| value.1)))
        .bind(Json(shipping.map_or_else(|| serde_json::json!({}), |value| value.1)))
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

    /// Finds an order by its public ID.
    ///
    /// # Errors
    /// Returns [`ModelError::EntityNotFound`] when no order exists for `pid`.
    pub async fn find_by_pid(db: &PgPool, pid: Uuid) -> ModelResult<Self> {
        sqlx::query_as::<_, Self>("SELECT * FROM orders WHERE pid = $1")
            .bind(pid)
            .fetch_optional(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
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
        r"SELECT id, jsonb_build_object(
            'pid', pid, 'addressType', address_type, 'label', label,
            'recipientName', recipient_name, 'company', company, 'lineOne', line_one,
            'lineTwo', line_two, 'city', city, 'region', region, 'postalCode', postal_code,
            'countryCode', country_code, 'email', email, 'phone', phone
          ) FROM addresses WHERE pid=$1 AND customer_id=$2 AND deleted_at IS NULL",
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
