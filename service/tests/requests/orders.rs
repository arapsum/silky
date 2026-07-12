use axum::http::HeaderValue;
use axum_test::TestServer;
use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;

use crate::seed_data;
use crate::utils;

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

async fn access_token(server: &TestServer, email: &str) -> HeaderValue {
    utils::login_users(
        server,
        &serde_json::json!({ "email": email, "password": "Password" }),
    )
    .await
    .access_token
}

fn with_auth(request: axum_test::TestRequest, token: HeaderValue) -> axum_test::TestRequest {
    let (name, value) = utils::auth_header(token);
    request.add_header(name, value)
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

        let response = with_auth(
            server.get(path),
            access_token(&server, "admin@silk.com").await,
        )
        .await;

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

        let response = with_auth(
            server.get(&format!("/orders/{pid}")),
            access_token(&server, "admin@silk.com").await,
        )
        .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        })
    })
    .await;
}

#[rstest]
#[case("customer_can_list_own_orders", "john.doe@acme.com", 2, None)]
#[case(
    "customer_cannot_override_order_scope",
    "john.doe@acme.com",
    2,
    Some("/orders?customerPid=e761d8e3-fc3e-4a2e-a6c9-7c7a4f2130e8")
)]
#[case(
    "customer_with_no_orders_gets_empty_list",
    "jane.smith@globex.com",
    0,
    None
)]
#[tokio::test]
#[serial]
async fn customer_can_list_only_their_orders(
    #[case] test_name: &str,
    #[case] email: &str,
    #[case] expected_total: i64,
    #[case] path: Option<&str>,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();
        seed_data(ctx.db()).await.expect("seed should complete");
        let response = with_auth(
            server.get(path.unwrap_or("/orders")),
            access_token(&server, email).await,
        )
        .await;

        let body: serde_json::Value = response.json();
        assert_eq!(body["pagination"]["totalItems"], expected_total);
        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!(test_name, (response.status_code(), body))
        });
    })
    .await;
}

#[rstest]
#[case("customer_cannot_update_order_john", "john.doe@acme.com", 403)]
#[case("customer_cannot_update_order_jane", "jane.smith@globex.com", 403)]
#[case(
    "staff_without_update_permission_cannot_update_order",
    "james.moriaty@continental.org",
    403
)]
#[tokio::test]
#[serial]
async fn customer_cannot_update_an_order(
    #[case] test_name: &str,
    #[case] email: &str,
    #[case] expected_status: u16,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();
        seed_data(ctx.db()).await.expect("seed should complete");
        let response = with_auth(
            server
                .patch(&format!("/orders/{ORDER_PID}"))
                .json(&serde_json::json!({ "status": "completed" })),
            access_token(&server, email).await,
        )
        .await;

        assert_eq!(response.status_code().as_u16(), expected_status);
        assert_debug_snapshot!(test_name, (response.status_code(), response.text()));
    })
    .await;
}

#[rstest]
#[case("staff_can_update_staff_note", serde_json::json!({ "staffNote": "Packed for dispatch" }))]
#[case("staff_can_update_fulfillment_status", serde_json::json!({ "fulfillmentStatus": "fulfilled" }))]
#[tokio::test]
#[serial]
async fn staff_with_order_permission_can_update_an_order(
    #[case] test_name: &str,
    #[case] payload: serde_json::Value,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();
        seed_data(ctx.db()).await.expect("seed should complete");
        let response = with_auth(
            server.patch(&format!("/orders/{ORDER_PID}")).json(&payload),
            access_token(&server, "admin@silk.com").await,
        )
        .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        });
    })
    .await;
}

#[rstest]
#[case("customer_can_view_owned_order", "john.doe@acme.com", 200)]
#[case(
    "customer_cannot_view_another_customers_order",
    "jane.smith@globex.com",
    404
)]
#[case(
    "staff_without_read_permission_cannot_view_orders",
    "james.moriaty@continental.org",
    403
)]
#[tokio::test]
#[serial]
async fn can_read_order_with_customer_and_staff_access_rules(
    #[case] test_name: &str,
    #[case] email: &str,
    #[case] expected_status: u16,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();
        seed_data(ctx.db()).await.expect("seed should complete");
        let response = with_auth(
            server.get(&format!("/orders/{ORDER_PID}")),
            access_token(&server, email).await,
        )
        .await;

        assert_eq!(response.status_code().as_u16(), expected_status);
        assert_debug_snapshot!(test_name, (response.status_code(), response.text()));
    })
    .await;
}
