use insta::{Settings, assert_debug_snapshot, with_settings};
use serial_test::serial;
use service::{models::Address, schemas::NewAddress};
use uuid::Uuid;

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    () => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("addresses");
        settings.set_snapshot_path("snapshots/addresses");
        let _guard = settings.bind_to_scope();
    };
}

fn new_address() -> NewAddress {
    serde_json::from_value(serde_json::json!({
        "customerPid": "bd6f7c26-d2c9-487e-b837-8f77be468033",
        "addressType": "shipping",
        "label": "Home",
        "recipientName": "John Doe",
        "lineOne": "10 Market Street",
        "city": "Nairobi",
        "countryCode": "KE",
        "email": "john.doe@acme.com",
        "isDefault": true
    }))
    .expect("address should deserialize")
}

#[tokio::test]
#[serial]
async fn creates_and_lists_customer_addresses() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let address = Address::create(ctx.db(), &new_address())
        .await
        .expect("address should create");
    let found = Address::find_by_pid(ctx.db(), address.pid()).await;
    let addresses = Address::find_by_customer(
        ctx.db(),
        Uuid::parse_str("bd6f7c26-d2c9-487e-b837-8f77be468033").expect("customer pid"),
    )
    .await;
    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date().to_vec()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("creates_and_lists_customer_addresses", (found, addresses.map(|values| values.len())))
    });
}
