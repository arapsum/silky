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
    access_token_for(server, "admin@silk.com").await
}

async fn access_token_for(server: &TestServer, email: &str) -> HeaderValue {
    let params = serde_json::json!({
        "email": email,
        "password": "Password"
    });

    utils::login_users(server, &params).await.access_token
}

async fn revoke_user_role(db: &sqlx::PgPool, user_id: i32, role_id: i32) {
    sqlx::query(
        r"
        DELETE FROM users_roles
        WHERE user_id = $1 AND role_id = $2
    ",
    )
    .bind(user_id)
    .bind(role_id)
    .execute(db)
    .await
    .expect("Failed to revoke user role");
}

async fn remove_user(db: &sqlx::PgPool, email: &str) {
    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(db)
        .await
        .expect("Failed to remove test user");
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
#[case(
    "can_create_staff_user",
    serde_json::json!({
        "name": "Warehouse Manager",
        "email": "warehouse.manager@silk.com",
        "password": "Password123",
        "confirmPassword": "Password123",
        "roleId": 11
    }),
    1
)]
#[case(
    "cannot_create_staff_user_when_email_already_exists",
    serde_json::json!({
        "name": "John Doe",
        "email": "john.doe@silk.com",
        "password": "Password123",
        "confirmPassword": "Password123",
        "roleId": 11
    }),
    1
)]
#[case(
    "cannot_create_staff_user_with_customer_role",
    serde_json::json!({
        "name": "Customer Account",
        "email": "shopper@silk.com",
        "password": "Password123",
        "confirmPassword": "Password123",
        "roleId": 22
    }),
    0
)]
#[case(
    "cannot_create_staff_user_with_missing_role",
    serde_json::json!({
        "name": "Missing Role",
        "email": "missing.role@silk.com",
        "password": "Password123",
        "confirmPassword": "Password123",
        "roleId": 999
    }),
    0
)]
#[case(
    "cannot_create_staff_user_with_invalid_payload",
    serde_json::json!({
        "name": "Warehouse Manager",
        "email": "warehouse.invalid@silk.com",
        "password": "Password123",
        "confirmPassword": "Different123",
        "roleId": 11
    }),
    0
)]
#[tokio::test]
#[serial]
async fn can_create_staff_user(
    #[case] test_name: &str,
    #[case] params: serde_json::Value,
    #[case] expected_role_count: i64,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let email = params["email"].as_str().expect("Email should be present");
        let is_seeded_email = email == "john.doe@silk.com";
        if !is_seeded_email {
            remove_user(ctx.db(), email).await;
        }
        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .post("/users")
            .add_header(auth_header, auth_value)
            .json(&params)
            .await;

        let assigned_roles = sqlx::query_scalar::<_, i64>(
            r"
            SELECT COUNT(*)
            FROM users_roles
            JOIN users ON users.id = users_roles.user_id
            WHERE users.email = $1
        ",
        )
        .bind(email)
        .fetch_one(ctx.db())
        .await
        .expect("Failed to inspect staff role assignment");
        assert_eq!(
            assigned_roles, expected_role_count,
            "The request should leave the expected number of roles assigned to {email}",
        );
        if !is_seeded_email {
            remove_user(ctx.db(), email).await;
        }

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!(test_name, (
                response.status_code(),
                response.text(),
                assigned_roles,
            ))
        })
    })
    .await;
}

#[rstest]
#[case(
    "can_assign_role_to_user",
    serde_json::json!({ "userId": 22, "roleId": 11 })
)]
#[case(
    "cannot_assign_role_when_user_already_has_role",
    serde_json::json!({ "userId": 11, "roleId": 44 })
)]
#[case(
    "cannot_assign_role_when_user_does_not_exist",
    serde_json::json!({ "userId": 999, "roleId": 11 })
)]
#[case(
    "cannot_assign_role_when_role_does_not_exist",
    serde_json::json!({ "userId": 22, "roleId": 999 })
)]
#[case(
    "cannot_assign_role_when_payload_is_invalid",
    serde_json::json!({ "userId": 0, "roleId": 0 })
)]
#[tokio::test]
#[serial]
async fn can_assign_role_to_user(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        revoke_user_role(ctx.db(), 22, 11).await;
        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .post("/users/roles")
            .add_header(auth_header, auth_value)
            .json(&params)
            .await;

        let snapshot = (response.status_code(), response.text());
        revoke_user_role(ctx.db(), 22, 11).await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!(test_name, snapshot)
        })
    })
    .await;
}

#[rstest]
#[case(
    "can_revoke_role_from_user",
    serde_json::json!({ "userId": 11, "roleId": 44 })
)]
#[case(
    "cannot_revoke_role_that_is_not_assigned",
    serde_json::json!({ "userId": 33, "roleId": 11 })
)]
#[case(
    "cannot_revoke_role_when_payload_is_invalid",
    serde_json::json!({ "userId": 0, "roleId": 0 })
)]
#[tokio::test]
#[serial]
async fn can_revoke_role_from_user(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .delete("/users/roles")
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
#[case("cannot_list_users_without_credentials", "GET", "/users")]
#[case(
    "cannot_assign_role_to_user_without_credentials",
    "POST",
    "/users/roles"
)]
#[case("cannot_create_staff_user_without_credentials", "POST", "/users")]
#[case(
    "cannot_revoke_role_from_user_without_credentials",
    "DELETE",
    "/users/roles"
)]
#[tokio::test]
#[serial]
async fn cannot_access_users_without_credentials(
    #[case] test_name: &str,
    #[case] method: &str,
    #[case] path: &str,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = match method {
            "GET" => server.get(path).await,
            "POST" => {
                let params = if path == "/users" {
                    serde_json::json!({
                        "name": "Warehouse Manager",
                        "email": "warehouse.manager@silk.com",
                        "password": "Password123",
                        "confirmPassword": "Password123",
                        "roleId": 11
                    })
                } else {
                    serde_json::json!({ "userId": 22, "roleId": 11 })
                };

                server.post(path).json(&params).await
            }
            "DELETE" => {
                server
                    .delete(path)
                    .json(&serde_json::json!({ "userId": 11, "roleId": 11 }))
                    .await
            }
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

#[rstest]
#[case("cannot_list_users_without_permission", "GET", "/users")]
#[case(
    "cannot_assign_role_to_user_without_permission",
    "POST",
    "/users/roles"
)]
#[case("cannot_create_staff_user_without_permission", "POST", "/users")]
#[case(
    "cannot_revoke_role_from_user_without_permission",
    "DELETE",
    "/users/roles"
)]
#[tokio::test]
#[serial]
async fn cannot_modify_users_without_permission(
    #[case] test_name: &str,
    #[case] method: &str,
    #[case] path: &str,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");
        let token = access_token_for(&server, "james.moriaty@continental.org").await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = match method {
            "GET" => server.get(path).add_header(auth_header, auth_value).await,
            "POST" => {
                let params = if path == "/users" {
                    serde_json::json!({
                        "name": "Warehouse Manager",
                        "email": "warehouse.manager@silk.com",
                        "password": "Password123",
                        "confirmPassword": "Password123",
                        "roleId": 11
                    })
                } else {
                    serde_json::json!({ "userId": 22, "roleId": 11 })
                };

                server
                    .post(path)
                    .add_header(auth_header, auth_value)
                    .json(&params)
                    .await
            }
            "DELETE" => {
                server
                    .delete(path)
                    .add_header(auth_header, auth_value)
                    .json(&serde_json::json!({ "userId": 11, "roleId": 11 }))
                    .await
            }
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
