use axum::{Json, Router, http::StatusCode, routing::get};
use axum_test::{TestServer, TestServerConfig};
use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use serde_json::json;
use serial_test::serial;
use service::{
    controllers,
    middlewares::{auth::AuthLayer, rbac::RbacLayer},
};

use crate::utils;

#[derive(Clone, Copy)]
enum Credentials {
    AuthorizationHeader,
    Missing,
}

macro_rules! configure_insta {
    ($(expr:expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/rbac");
        settings.set_snapshot_suffix("rbac");
        let _guard = settings.bind_to_scope();
    };
}

async fn grant_permission(db: &sqlx::PgPool, role: &str, permission: &str) {
    sqlx::query(
        r"
        INSERT INTO roles_permissions (role_id, permission_id)
        SELECT roles.id, permissions.id
        FROM roles
        CROSS JOIN permissions
        WHERE roles.name = $1
            AND permissions.name = $2
        ON CONFLICT (role_id, permission_id) DO NOTHING
    ",
    )
    .bind(role)
    .bind(permission)
    .execute(db)
    .await
    .expect("Failed to grant permission");
}

async fn assign_role(db: &sqlx::PgPool, email: &str, role: &str) {
    sqlx::query(
        r"
        INSERT INTO users_roles (user_id, role_id)
        SELECT users.id, roles.id
        FROM users
        CROSS JOIN roles
        WHERE users.email = $1
            AND roles.name = $2
        ON CONFLICT (user_id, role_id) DO NOTHING
    ",
    )
    .bind(email)
    .bind(role)
    .execute(db)
    .await
    .expect("Failed to assign role");
}

#[rstest]
#[case(
    "can_access_route_when_role_has_permission",
    "roles:read",
    Credentials::AuthorizationHeader,
    "administrator",
    true
)]
#[case(
    "can_access_route_when_customer_role_has_permission",
    "roles:read",
    Credentials::AuthorizationHeader,
    "customer",
    true
)]
#[case(
    "cannot_access_route_when_role_lacks_permission",
    "roles:write",
    Credentials::AuthorizationHeader,
    "administrator",
    true
)]
#[case(
    "cannot_access_route_without_credentials",
    "roles:read",
    Credentials::Missing,
    "administrator",
    false
)]
#[tokio::test]
#[serial]
async fn can_authorise_with_rbac(
    #[case] test_name: &str,
    #[case] required_permission: &str,
    #[case] credentials: Credentials,
    #[case] role: &str,
    #[case] assign_user_role: bool,
) {
    configure_insta!();

    let ctx = crate::boot_test().await.unwrap();

    crate::seed_data(ctx.db())
        .await
        .expect("Failed to seed data");
    grant_permission(ctx.db(), role, "roles:read").await;
    if assign_user_role {
        assign_role(ctx.db(), "john.doe@acme.com", role).await;
    }

    let protected = Router::new()
        .route(
            "/rbac/protected",
            get(|| async { (StatusCode::OK, Json(json!({ "message": "granted" }))) }),
        )
        .layer(RbacLayer::new(ctx.clone(), required_permission))
        .layer(AuthLayer::new(ctx.clone()));

    let cfg = TestServerConfig {
        default_content_type: Some("application/json".into()),
        save_cookies: true,
        ..Default::default()
    };
    let server = TestServer::new_with_config(controllers::router(&ctx).merge(protected), cfg);

    let mut request = server.get("/rbac/protected");

    if let Credentials::AuthorizationHeader = credentials {
        let params = serde_json::json!({
            "email": "john.doe@acme.com",
            "password": "Password"
        });
        let user = utils::login_users(&server, &params).await;
        let (auth_header, auth_value) = utils::auth_header(user.access_token);
        request = request.add_header(auth_header, auth_value);
    }

    let response = request.await;

    assert_debug_snapshot!(test_name, (response.status_code(), response.text()));
}
