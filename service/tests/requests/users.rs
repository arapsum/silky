use axum::http::HeaderValue;
use axum_test::TestServer;
use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;

use crate::utils;

macro_rules! configure_insta {
    ($(expr:expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/users");
        settings.set_snapshot_suffix("users");
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

async fn revoke_role(db: &sqlx::PgPool, email: &str, role: &str) {
    sqlx::query(
        r"
        DELETE FROM users_roles
        USING users, roles
        WHERE users_roles.user_id = users.id
            AND users_roles.role_id = roles.id
            AND users.email = $1
            AND roles.name = $2
    ",
    )
    .bind(email)
    .bind(role)
    .execute(db)
    .await
    .expect("Failed to revoke role");
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
#[case("can_list_users", "/users")]
#[case("can_list_customer_users", "/users?role=customer")]
#[case("can_list_administrator_users", "/users?role=administrator")]
#[case("can_list_users_when_role_does_not_exist", "/users?role=missing")]
#[tokio::test]
#[serial]
async fn can_list_users(#[case] test_name: &str, #[case] path: &str) {
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
#[case("cannot_list_users_without_credentials", "/users")]
#[tokio::test]
#[serial]
async fn cannot_access_users_without_credentials(#[case] test_name: &str, #[case] path: &str) {
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

#[rstest]
#[case("cannot_list_users_without_permission")]
#[tokio::test]
#[serial]
async fn cannot_list_users_without_permission(#[case] test_name: &str) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");
        revoke_role(ctx.db(), "john.doe@acme.com", "administrator").await;

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .get("/users")
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
