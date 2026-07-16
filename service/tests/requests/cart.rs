use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;

use crate::{seed_data, utils};

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
    "quotes_cart_without_authentication",
    serde_json::json!({
        "items": [{
            "variantPid": "db365773-2ac1-49aa-a4b9-03dcf8ac3401",
            "quantity": 2
        }]
    })
)]
#[case("rejects_an_empty_cart_quote", serde_json::json!({ "items": [] }))]
#[case(
    "rejects_non_positive_cart_quantities",
    serde_json::json!({
        "items": [{
            "variantPid": "db365773-2ac1-49aa-a4b9-03dcf8ac3401",
            "quantity": 0
        }]
    })
)]
#[tokio::test]
#[serial]
async fn validates_cart_quote_requests(#[case] name: &str, #[case] payload: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();
        seed_data(ctx.db()).await.expect("seed should complete");

        let response = server.post("/cart/quote").json(&payload).await;
        let mut filters = utils::cleanup_uuid().to_vec();
        filters.extend(utils::cleanup_date().to_vec());
        with_settings!({ filters => filters }, {
            assert_debug_snapshot!(name, (response.status_code(), response.text()));
        });
    })
    .await;
}
