use axum::http::HeaderValue;
use axum_test::TestServer;
use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;

use crate::utils;

const ADMIN_PID: &str = "7d416019-34c6-4f25-a39a-fa6752f8b319";
const CUSTOMER_PID: &str = "f028f910-1a4f-4b79-8619-71a8c185e221";
const MISSING_PID: &str = "00000000-0000-0000-0000-000000000000";

macro_rules! configure_insta {
    ($(expr:expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/roles");
        settings.set_snapshot_suffix("roles");
        let _guard = settings.bind_to_scope();
    };
}

async fn access_token(server: &TestServer) -> HeaderValue {
    let params = serde_json::json!({
        "email": "john.doe@acme.com",
        "password": "Password"
    });

    utils::login_users(server, &params).await.access_token
}

fn response_filters() -> Vec<(&'static str, &'static str)> {
    let mut filters = utils::cleanup_date().to_vec();
    filters.extend(utils::cleanup_uuid().to_vec());
    filters.extend(utils::cleanup_headers());
    filters.push((r#""id":\d+"#, r#""id":ID"#));
    filters.push(("DATEZ", "DATE"));
    filters
}

#[rstest]
#[case(
    "can_create_role_with_description",
    serde_json::json!({
        "name": "Warehouse Manager",
        "description": "Owns warehouse operations"
    })
)]
#[case(
    "can_create_role_without_description",
    serde_json::json!({
        "name": "Support Agent"
    })
)]
#[case(
    "can_create_role_and_normalize_name",
    serde_json::json!({
        "name": "  Billing Admin  ",
        "description": "Billing team"
    })
)]
#[case(
    "cannot_create_role_when_name_already_exists",
    serde_json::json!({
        "name": "Administrator",
        "description": "Duplicate seeded role"
    })
)]
#[case(
    "cannot_create_role_when_name_is_invalid",
    serde_json::json!({
        "name": "Admin+",
        "description": "Invalid name"
    })
)]
#[case(
    "cannot_create_role_when_description_is_too_long",
    serde_json::json!({
        "name": "Auditor",
        "description": "a".repeat(257)
    })
)]
#[tokio::test]
#[serial]
async fn can_create_role(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .post("/roles")
            .add_header(auth_header, auth_value)
            .json(&params)
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
#[case("can_list_roles")]
#[tokio::test]
#[serial]
async fn can_list_roles(#[case] test_name: &str) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .get("/roles")
            .add_header(auth_header, auth_value)
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
#[case("can_get_administrator_role", ADMIN_PID)]
#[case("can_get_customer_role", CUSTOMER_PID)]
#[case("cannot_get_role_when_pid_does_not_exist", MISSING_PID)]
#[case("cannot_get_role_when_pid_is_invalid", "not-a-uuid")]
#[tokio::test]
#[serial]
async fn can_get_role(#[case] test_name: &str, #[case] pid: &str) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .get(&format!("/roles/{pid}"))
            .add_header(auth_header, auth_value)
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
#[case(
    "can_update_role_description",
    CUSTOMER_PID,
    serde_json::json!({
        "description": "Both registered shoppers and guest browsers"
    })
)]
#[case(
    "can_update_role_name",
    CUSTOMER_PID,
    serde_json::json!({
        "name": "Buyer"
    })
)]
#[case(
    "can_update_role_name_and_description",
    CUSTOMER_PID,
    serde_json::json!({
        "name": "Wholesale Buyer",
        "description": "Bulk order customers"
    })
)]
#[case(
    "can_update_role_and_normalize_name",
    CUSTOMER_PID,
    serde_json::json!({
        "name": "  VIP Customer  ",
        "description": "Priority shoppers"
    })
)]
#[case(
    "cannot_update_role_when_name_already_exists",
    CUSTOMER_PID,
    serde_json::json!({
        "name": "Administrator",
        "description": "Duplicate seeded role"
    })
)]
#[case(
    "cannot_update_role_when_pid_does_not_exist",
    MISSING_PID,
    serde_json::json!({
        "name": "Guest",
        "description": "Guest shoppers"
    })
)]
#[case(
    "cannot_update_role_when_name_is_invalid",
    CUSTOMER_PID,
    serde_json::json!({
        "name": "Buyer+",
        "description": "Invalid role"
    })
)]
#[case(
    "cannot_update_role_when_description_is_too_long",
    CUSTOMER_PID,
    serde_json::json!({
        "name": "Auditor",
        "description": "a".repeat(257)
    })
)]
#[tokio::test]
#[serial]
async fn can_update_role(
    #[case] test_name: &str,
    #[case] pid: &str,
    #[case] params: serde_json::Value,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .patch(&format!("/roles/{pid}"))
            .add_header(auth_header, auth_value)
            .json(&params)
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
#[case("cannot_create_role_without_credentials", "POST", "/roles")]
#[case("cannot_list_roles_without_credentials", "GET", "/roles")]
#[case(
    "cannot_get_role_without_credentials",
    "GET",
    "/roles/7d416019-34c6-4f25-a39a-fa6752f8b319"
)]
#[case(
    "cannot_update_role_without_credentials",
    "PATCH",
    "/roles/f028f910-1a4f-4b79-8619-71a8c185e221"
)]
#[tokio::test]
#[serial]
async fn cannot_access_roles_without_credentials(
    #[case] test_name: &str,
    #[case] method: &str,
    #[case] path: &str,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let body = serde_json::json!({
            "name": "Guest",
            "description": "Guest shoppers"
        });

        let response = match method {
            "POST" => server.post(path).json(&body).await,
            "PATCH" => server.patch(path).json(&body).await,
            "GET" => server.get(path).await,
            _ => unreachable!("unsupported request method"),
        };

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        })
    })
    .await;
}
