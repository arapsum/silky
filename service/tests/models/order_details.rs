use insta::{Settings, assert_debug_snapshot, with_settings};
use serial_test::serial;
use service::{
    models::{Address, Order, OrderDetail},
    schemas::{NewOrder, NewOrderDetail},
};

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    () => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("order_details");
        settings.set_snapshot_path("snapshots/order_details");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn creates_and_lists_order_details_from_variant_snapshot() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let address = Address::create(
        ctx.db(),
        &serde_json::from_value(serde_json::json!({
            "customerPid": "bd6f7c26-d2c9-487e-b837-8f77be468033",
            "addressType": "shipping",
            "recipientName": "John Doe",
            "lineOne": "10 Market Street",
            "city": "Nairobi",
            "countryCode": "KE",
            "isDefault": false
        }))
        .expect("address"),
    )
    .await
    .expect("address should create");
    let order: NewOrder = serde_json::from_value(serde_json::json!({
        "customerPid": "bd6f7c26-d2c9-487e-b837-8f77be468033",
        "shippingAddressPid": address.pid(),
        "currency": "USD",
        "subtotal": "24.99",
        "discountTotal": "0.00",
        "shippingTotal": "0.00",
        "taxTotal": "0.00",
        "grandTotal": "24.99"
    }))
    .expect("order");
    let order = Order::create(ctx.db(), &order)
        .await
        .expect("order should create");
    let input: NewOrderDetail = serde_json::from_value(serde_json::json!({
        "orderPid": order.pid(),
        "variantPid": "db365773-2ac1-49aa-a4b9-03dcf8ac3401",
        "quantity": 1,
        "discountTotal": "0.00",
        "taxTotal": "0.00"
    }))
    .expect("order detail");
    let detail = OrderDetail::create(ctx.db(), &input)
        .await
        .expect("detail should create");
    let details = OrderDetail::find_by_order(ctx.db(), order.pid())
        .await
        .expect("details should list");

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date().to_vec()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("creates_and_lists_order_details_from_variant_snapshot", (detail, details.len()))
    });
}
