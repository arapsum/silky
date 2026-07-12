use insta::{Settings, assert_debug_snapshot, with_settings};
use serial_test::serial;
use service::models::OrderItem;
use uuid::Uuid;

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    () => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("order_items");
        settings.set_snapshot_path("snapshots/order_items");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn lists_seeded_order_items_from_their_snapshots() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let order_pid = Uuid::parse_str("c2c4ed90-7b62-43d2-8b6d-96f4f4f94001")
        .expect("seeded order pid should parse");
    let items = OrderItem::find_by_order(ctx.db(), order_pid)
        .await
        .expect("items should list");

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date().to_vec()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("lists_seeded_order_items_from_their_snapshots", items)
    });
}
