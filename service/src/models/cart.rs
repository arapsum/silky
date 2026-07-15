use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value as JsonValue;
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

use crate::schemas::CartQuoteRequest;

use super::ModelResult;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CartQuote {
    items: Vec<CartQuoteItem>,
    currency: &'static str,
    subtotal: Decimal,
    shipping_total: Decimal,
    tax_total: Decimal,
    grand_total: Decimal,
    can_checkout: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CartQuoteItem {
    variant_pid: Uuid,
    product_pid: Option<Uuid>,
    product_slug: Option<String>,
    product_name: Option<String>,
    sku: Option<String>,
    image_url: Option<String>,
    selected_options: Json<JsonValue>,
    requested_quantity: i32,
    available_quantity: i32,
    unit_price: Option<Decimal>,
    line_total: Option<Decimal>,
    status: CartQuoteStatus,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum CartQuoteStatus {
    Available,
    InsufficientStock,
    Unavailable,
}

#[derive(Debug, FromRow)]
struct QuotedVariant {
    variant_pid: Uuid,
    product_pid: Uuid,
    product_slug: String,
    product_name: String,
    sku: String,
    image_url: Option<String>,
    selected_options: Json<JsonValue>,
    stock_quantity: i32,
    price: Decimal,
}

impl CartQuote {
    /// Builds a read-only, server-priced quote for a set of variant quantities.
    ///
    /// Missing and out-of-stock variants remain in the response as line-level
    /// problems so a storefront can reconcile the whole cart in one request.
    /// No stock is locked or reserved by this operation.
    ///
    /// # Errors
    /// Returns a database error when catalogue data cannot be loaded.
    pub async fn create(db: &PgPool, request: &CartQuoteRequest) -> ModelResult<Self> {
        let variant_pids = request
            .items()
            .iter()
            .map(crate::schemas::NewOrderItem::variant_pid)
            .collect::<Vec<_>>();
        let variants = sqlx::query_as::<_, QuotedVariant>(
            r"SELECT variant.pid AS variant_pid,
                    product.pid AS product_pid,
                    product.slug AS product_slug,
                    product.name AS product_name,
                    variant.sku,
                    COALESCE(
                        (
                            SELECT picture.image_link
                            FROM pictures AS picture
                            WHERE picture.product_id = product.id
                               OR picture.variant_id = variant.id
                            ORDER BY picture.display_order NULLS LAST, picture.id
                            LIMIT 1
                        ),
                        NULL
                    ) AS image_url,
                    COALESCE(
                        (
                            SELECT jsonb_object_agg(attribute.name, value.value)
                            FROM variant_attribute_values AS link
                            JOIN attributes AS attribute ON attribute.id = link.attribute_id
                            JOIN attribute_values AS value ON value.id = link.attribute_value_id
                            WHERE link.variant_id = variant.id
                        ),
                        '{}'::jsonb
                    ) AS selected_options,
                    variant.stock_quantity,
                    variant.price
              FROM product_variants AS variant
              JOIN products AS product ON product.id = variant.product_id
              WHERE variant.pid = ANY($1)
                AND variant.deleted_at IS NULL
                AND product.deleted_at IS NULL",
        )
        .bind(&variant_pids)
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|variant| (variant.variant_pid, variant))
        .collect::<HashMap<_, _>>();

        let mut subtotal = Decimal::ZERO;
        let mut can_checkout = true;
        let mut items = Vec::with_capacity(request.items().len());

        for requested in request.items() {
            let Some(variant) = variants.get(&requested.variant_pid()) else {
                can_checkout = false;
                items.push(CartQuoteItem {
                    variant_pid: requested.variant_pid(),
                    product_pid: None,
                    product_slug: None,
                    product_name: None,
                    sku: None,
                    image_url: None,
                    selected_options: Json(serde_json::json!({})),
                    requested_quantity: requested.quantity(),
                    available_quantity: 0,
                    unit_price: None,
                    line_total: None,
                    status: CartQuoteStatus::Unavailable,
                });
                continue;
            };

            let has_stock = variant.stock_quantity >= requested.quantity();
            let line_total = variant.price * Decimal::from(requested.quantity());
            if has_stock {
                subtotal += line_total;
            } else {
                can_checkout = false;
            }
            items.push(CartQuoteItem {
                variant_pid: variant.variant_pid,
                product_pid: Some(variant.product_pid),
                product_slug: Some(variant.product_slug.clone()),
                product_name: Some(variant.product_name.clone()),
                sku: Some(variant.sku.clone()),
                image_url: variant.image_url.clone(),
                selected_options: variant.selected_options.clone(),
                requested_quantity: requested.quantity(),
                available_quantity: variant.stock_quantity,
                unit_price: Some(variant.price),
                line_total: Some(line_total),
                status: if has_stock {
                    CartQuoteStatus::Available
                } else {
                    CartQuoteStatus::InsufficientStock
                },
            });
        }

        Ok(Self {
            items,
            currency: "USD",
            subtotal,
            shipping_total: Decimal::ZERO,
            tax_total: Decimal::ZERO,
            grand_total: subtotal,
            can_checkout,
        })
    }
}
