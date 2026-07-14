use axum::http::HeaderValue;
use axum_test::TestServer;
use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;

use crate::utils;

const ROLES_READ_PID: &str = "9e230b11-cb47-4fe8-8bc0-5185fd9f9bb2";
const PERMISSIONS_READ_PID: &str = "a0199d51-0147-477f-8778-070785ee81f3";
const MISSING_PID: &str = "00000000-0000-0000-0000-000000000000";

macro_rules! configure_insta {
    ($(expr:expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/permissions");
        settings.set_snapshot_suffix("permissions");
        let _guard = settings.bind_to_scope();
    };
}

async fn access_token(server: &TestServer) -> HeaderValue {
    let params = serde_json::json!({
        "email": "admin@silk.com",
        "password": "Password"
    });

    utils::login_users(server, &params).await.access_token
}

fn response_filters() -> Vec<(&'static str, &'static str)> {
    let mut filters = utils::cleanup_date().to_vec();
    filters.extend(utils::cleanup_uuid().to_vec());
    filters.extend(utils::cleanup_headers());
    filters.push((r#""id":\d+"#, r#""id":ID"#));
    filters
}

#[rstest]
#[case("can_list_permissions", "/permissions")]
#[case("can_list_permissions_by_role", "/permissions?role=%20Customer%20")]
#[case(
    "can_list_permissions_when_role_does_not_exist",
    "/permissions?role=missing"
)]
#[tokio::test]
#[serial]
async fn can_list_permissions(#[case] test_name: &str, #[case] path: &str) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server.get(path).add_header(auth_header, auth_value).await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        })
    })
    .await;
}

#[rstest]
#[case("can_get_roles_read_permission", ROLES_READ_PID)]
#[case("can_get_permissions_read_permission", PERMISSIONS_READ_PID)]
#[case("cannot_get_permission_when_pid_does_not_exist", MISSING_PID)]
#[case("cannot_get_permission_when_pid_is_invalid", "not-a-uuid")]
#[tokio::test]
#[serial]
async fn can_get_permission(#[case] test_name: &str, #[case] pid: &str) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .get(&format!("/permissions/{pid}"))
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
#[case("cannot_list_permissions_without_credentials", "/permissions")]
#[case(
    "cannot_get_permission_without_credentials",
    "/permissions/9e230b11-cb47-4fe8-8bc0-5185fd9f9bb2"
)]
#[tokio::test]
#[serial]
async fn cannot_access_permissions_without_credentials(
    #[case] test_name: &str,
    #[case] path: &str,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server.get(path).await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        })
    })
    .await;
}
