use insta::{Settings, assert_debug_snapshot, with_settings};
use serial_test::serial;
use service::{models::Order, schemas::NewOrder};
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

fn new_order(billing: Option<Uuid>, shipping: Option<Uuid>) -> NewOrder {
    serde_json::from_value(serde_json::json!({
        "customerPid": "bd6f7c26-d2c9-487e-b837-8f77be468033",
        "billingAddressPid": billing,
        "shippingAddressPid": shipping,
        "currency": "USD",
        "subtotal": "24.99",
        "discountTotal": "0.00",
        "shippingTotal": "5.00",
        "taxTotal": "0.00",
        "grandTotal": "29.99",
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
async fn creates_order_with_address_snapshots() {
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
    let order = Order::create(
        ctx.db(),
        &new_order(Some(billing.pid()), Some(shipping.pid())),
    )
    .await
    .expect("order should create");
    let found = Order::find_by_pid(ctx.db(), order.pid())
        .await
        .expect("order should find");

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date().to_vec()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("creates_order_with_address_snapshots", found)
    });
}
