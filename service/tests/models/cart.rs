use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::{models::CartQuote, schemas::CartQuoteRequest};

use crate::{boot_test, seed_data, utils::cleanup_uuid};

macro_rules! configure_insta {
    () => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("cart");
        settings.set_snapshot_path("snapshots/cart");
        let _guard = settings.bind_to_scope();
    };
}

#[rstest]
#[case(
    "quotes_available_inventory",
    "db365773-2ac1-49aa-a4b9-03dcf8ac3401",
    2
)]
#[case(
    "reports_insufficient_inventory",
    "db365773-2ac1-49aa-a4b9-03dcf8ac3401",
    33
)]
#[case(
    "retains_missing_variants_as_unavailable_lines",
    "00000000-0000-0000-0000-000000000000",
    1
)]
#[tokio::test]
#[serial]
async fn builds_authoritative_cart_quotes(
    #[case] name: &str,
    #[case] variant_pid: &str,
    #[case] quantity: i32,
) {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let request = serde_json::from_value::<CartQuoteRequest>(serde_json::json!({
        "items": [{ "variantPid": variant_pid, "quantity": quantity }]
    }))
    .expect("quote should deserialize");

    let quote = CartQuote::create(ctx.db(), &request)
        .await
        .expect("quote should build");

    with_settings!({ filters => cleanup_uuid().to_vec() }, {
        assert_debug_snapshot!(name, quote);
    });
}
