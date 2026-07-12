use axum::body::to_bytes;
use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::{
    models::{ModelError, Order, OrderItem},
    schemas::NewOrder,
};
use uuid::Uuid;

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    () => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("orders");
        settings.set_snapshot_path("snapshots/orders");
        let _guard = settings.bind_to_scope();
    };
}

fn new_order(
    billing: Option<Uuid>,
    shipping: Option<Uuid>,
    variant_pid: Uuid,
    quantity: i32,
) -> NewOrder {
    serde_json::from_value(serde_json::json!({
        "customerPid": "bd6f7c26-d2c9-487e-b837-8f77be468033",
        "billingAddressPid": billing,
        "shippingAddressPid": shipping,
        "items": [{
            "variantPid": variant_pid,
            "quantity": quantity
        }],
        "customerNote": "Leave at the front desk"
    }))
    .expect("order should deserialize")
}

fn new_address() -> service::schemas::NewAddress {
    serde_json::from_value(serde_json::json!({
        "customerPid": "bd6f7c26-d2c9-487e-b837-8f77be468033",
        "addressType": "shipping",
        "recipientName": "John Doe",
        "lineOne": "10 Market Street",
        "city": "Nairobi",
        "countryCode": "KE",
        "isDefault": false
    }))
    .expect("address should deserialize")
}

#[tokio::test]
#[serial]
async fn checkout_calculates_totals_and_reserves_inventory() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let billing = service::models::Address::create(
        ctx.db(),
        &serde_json::from_value(serde_json::json!({
            "customerPid": "bd6f7c26-d2c9-487e-b837-8f77be468033",
            "addressType": "billing",
            "recipientName": "John Doe",
            "lineOne": "10 Market Street",
            "city": "Nairobi",
            "countryCode": "KE",
            "isDefault": false
        }))
        .expect("billing address"),
    )
    .await
    .expect("billing address should create");
    let shipping = service::models::Address::create(ctx.db(), &new_address())
        .await
        .expect("shipping address should create");
    let variant_pid =
        Uuid::parse_str("db365773-2ac1-49aa-a4b9-03dcf8ac3401").expect("variant pid should parse");
    let order = Order::create(
        ctx.db(),
        &new_order(Some(billing.pid()), Some(shipping.pid()), variant_pid, 2),
    )
    .await
    .expect("order should create");
    let found = Order::find_by_pid(ctx.db(), order.pid())
        .await
        .expect("order should find");
    let items = OrderItem::find_by_order(ctx.db(), order.pid())
        .await
        .expect("order items should find");
    let remaining_stock =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(variant_pid)
            .fetch_one(ctx.db())
            .await
            .expect("variant stock should load");

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date().to_vec()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("checkout_calculates_totals_and_reserves_inventory", (found, items, remaining_stock))
    });
}

#[rstest]
#[case("insufficient stock", "db365773-2ac1-49aa-a4b9-03dcf8ac3401", 33)]
#[case("missing variant", "00000000-0000-0000-0000-000000000000", 1)]
#[tokio::test]
#[serial]
async fn rejected_checkout_does_not_create_an_order_or_reserve_stock(
    #[case] name: &str,
    #[case] variant_pid: &str,
    #[case] quantity: i32,
) {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let variant_pid = Uuid::parse_str(variant_pid).expect("variant pid should parse");
    let orders_before = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM orders")
        .fetch_one(ctx.db())
        .await
        .expect("order count should load");
    let stock_before =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(Uuid::parse_str("db365773-2ac1-49aa-a4b9-03dcf8ac3401").expect("seed pid"))
            .fetch_one(ctx.db())
            .await
            .expect("stock should load");

    let result = Order::create(ctx.db(), &new_order(None, None, variant_pid, quantity))
        .await
        .map(|order| order.display_number())
        .map_err(|error| error.to_string());
    let orders_after = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM orders")
        .fetch_one(ctx.db())
        .await
        .expect("order count should reload");
    let stock_after =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(Uuid::parse_str("db365773-2ac1-49aa-a4b9-03dcf8ac3401").expect("seed pid"))
            .fetch_one(ctx.db())
            .await
            .expect("stock should reload");

    with_settings!({ filters => cleanup_uuid().to_vec() }, {
        assert_debug_snapshot!(name, (result, orders_before, orders_after, stock_before, stock_after))
    });
}

#[rstest]
#[case("empty order", serde_json::json!([]))]
#[case(
    "duplicate variant",
    serde_json::json!([
        {
            "variantPid": "db365773-2ac1-49aa-a4b9-03dcf8ac3401",
            "quantity": 1
        },
        {
            "variantPid": "db365773-2ac1-49aa-a4b9-03dcf8ac3401",
            "quantity": 2
        }
    ])
)]
#[tokio::test]
#[serial]
async fn rejects_invalid_order_item_collections(
    #[case] name: &str,
    #[case] items: serde_json::Value,
) {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let input: NewOrder = serde_json::from_value(serde_json::json!({
        "customerPid": "bd6f7c26-d2c9-487e-b837-8f77be468033",
        "items": items
    }))
    .expect("order should deserialize");

    let result = Order::create(ctx.db(), &input)
        .await
        .map(|order| order.display_number())
        .map_err(|error| error.to_string());
    let order_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM orders")
        .fetch_one(ctx.db())
        .await
        .expect("order count should load");

    assert_debug_snapshot!(name, (result, order_count));
}

#[test]
fn order_errors_expose_stable_codes_fields_and_statuses() {
    configure_insta!();
    let errors = [
        ModelError::CustomerNotFound,
        ModelError::CustomerAddressNotFound {
            field: "shippingAddressPid",
        },
        ModelError::OrderHasNoItems,
        ModelError::InvalidOrderItemQuantity,
        ModelError::DuplicateOrderItem,
        ModelError::ProductVariantUnavailable,
        ModelError::InsufficientStock {
            variant_pid: Uuid::nil(),
            requested: 4,
            available: 2,
        },
        ModelError::OrderNotEditable,
    ];
    let metadata = errors
        .iter()
        .map(|error| {
            let (status, message) = error.response_body();
            (error.code(), error.field(), status, message)
        })
        .collect::<Vec<_>>();

    assert_debug_snapshot!("order_error_metadata", metadata);
}

#[tokio::test]
async fn order_error_response_includes_machine_readable_fields() {
    configure_insta!();
    let response = ModelError::InsufficientStock {
        variant_pid: Uuid::nil(),
        requested: 4,
        available: 2,
    }
    .response();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("error response body should read");
    let body: serde_json::Value =
        serde_json::from_slice(&body).expect("error response should contain JSON");

    with_settings!({ filters => cleanup_uuid().to_vec() }, {
        assert_debug_snapshot!("structured_order_error_response", (status, body));
    });
}
