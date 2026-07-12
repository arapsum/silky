use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;

use crate::seed_data;

const ORDER_PID: &str = "c2c4ed90-7b62-43d2-8b6d-96f4f4f94001";
const MISSING_PID: &str = "00000000-0000-0000-0000-000000000000";

macro_rules! configure_insta {
    ($(expr:expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/orders");
        settings.set_snapshot_suffix("orders");
        let _guard = settings.bind_to_scope();
    };
}

fn response_filters() -> Vec<(&'static str, &'static str)> {
    let mut filters = crate::utils::cleanup_date().to_vec();
    filters.extend(crate::utils::cleanup_uuid().to_vec());
    filters.extend(crate::utils::cleanup_headers());
    filters.push((r#""id":\d+"#, r#""id":ID"#));
    filters.push(("DATEZ", "DATE"));
    filters
}

#[rstest]
#[case("can_list_orders", "/orders")]
#[case(
    "can_list_completed_orders",
    "/orders?status=completed&limit=10&page=1"
)]
#[case("can_list_pending_payments", "/orders?paymentStatus=pending")]
#[case("can_list_unfulfilled_orders", "/orders?fulfillmentStatus=unfulfilled")]
#[case("can_search_orders", "/orders?search=john.doe")]
#[case(
    "can_list_orders_in_date_range",
    "/orders?createdFrom=2026-07-01&createdTo=2026-07-02"
)]
#[case("cannot_list_orders_with_invalid_limit", "/orders?limit=0")]
#[tokio::test]
#[serial]
async fn can_list_orders(#[case] test_name: &str, #[case] path: &str) {
    crate::request(|server, ctx| async move {
        configure_insta!();
        seed_data(ctx.db()).await.expect("seed should complete");

        let response = server.get(path).await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        })
    })
    .await;
}

#[rstest]
#[case("can_fetch_order_with_its_items", ORDER_PID)]
#[case("cannot_fetch_order_when_pid_does_not_exist", MISSING_PID)]
#[case("cannot_fetch_order_when_pid_is_invalid", "not-a-uuid")]
#[tokio::test]
#[serial]
async fn can_fetch_order(#[case] test_name: &str, #[case] pid: &str) {
    crate::request(|server, ctx| async move {
        configure_insta!();
        seed_data(ctx.db()).await.expect("seed should complete");

        let response = server.get(&format!("/orders/{pid}")).await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        })
    })
    .await;
}
