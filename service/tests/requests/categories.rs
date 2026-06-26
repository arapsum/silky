use axum::http::HeaderValue;
use axum_test::TestServer;
use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;

use crate::utils;

const TSHIRTS_PID: &str = "f63b79c9-4753-40c3-bc78-8c4fd38abd5b";
const TROUSERS_PID: &str = "00b92bcb-cc7a-4a2b-bd80-e9c1b40d1c46";
const MISSING_PID: &str = "00000000-0000-0000-0000-000000000000";

macro_rules! configure_insta {
    ($(expr:expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/categories");
        settings.set_snapshot_suffix("categories");
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

async fn allow_category_writes(db: &sqlx::PgPool) {
    assign_role(db, "john.doe@acme.com", "administrator").await;
    grant_permission(db, "administrator", "categories:create").await;
    grant_permission(db, "administrator", "categories:update").await;
    grant_permission(db, "administrator", "categories:delete").await;
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
#[case("can_list_categories", "/categories")]
#[case("can_list_categories_with_pagination", "/categories?limit=2&page=2")]
#[case(
    "cannot_list_categories_with_invalid_limit",
    "/categories?limit=0&page=1"
)]
#[tokio::test]
#[serial]
async fn can_list_categories(#[case] test_name: &str, #[case] path: &str) {
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
#[case("can_get_tshirts_category", TSHIRTS_PID)]
#[case("cannot_get_category_when_pid_does_not_exist", MISSING_PID)]
#[case("cannot_get_category_when_pid_is_invalid", "not-a-uuid")]
#[tokio::test]
#[serial]
async fn can_get_category(#[case] test_name: &str, #[case] pid: &str) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server.get(&format!("/categories/{pid}")).await;

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
    "can_create_category_with_description",
    serde_json::json!({
        "name": "Accessories",
        "imageLink": "https://cdn.example.com/categories/accessories.png",
        "description": "Bags and belts"
    })
)]
#[case(
    "can_create_category_without_description",
    serde_json::json!({
        "name": "Hats",
        "imageLink": "https://cdn.example.com/categories/hats.png"
    })
)]
#[case(
    "can_create_category_and_normalize_name",
    serde_json::json!({
        "name": "  Winter Wear  ",
        "imageLink": "  https://cdn.example.com/categories/winter.png  ",
        "description": "Warm layers"
    })
)]
#[case(
    "cannot_create_category_when_name_already_exists",
    serde_json::json!({
        "name": "T-shirts",
        "imageLink": "https://cdn.example.com/categories/duplicate.png",
        "description": "Duplicate seeded category"
    })
)]
#[case(
    "cannot_create_category_when_name_is_invalid",
    serde_json::json!({
        "name": "T-shirts+",
        "imageLink": "https://cdn.example.com/categories/tshirts.png"
    })
)]
#[case(
    "cannot_create_category_when_image_link_is_invalid",
    serde_json::json!({
        "name": "Accessories",
        "imageLink": "not-a-url"
    })
)]
#[case(
    "cannot_create_category_when_description_is_too_long",
    serde_json::json!({
        "name": "Accessories",
        "imageLink": "https://cdn.example.com/categories/accessories.png",
        "description": "a".repeat(1001)
    })
)]
#[tokio::test]
#[serial]
async fn can_create_category(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");
        allow_category_writes(ctx.db()).await;

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .post("/categories")
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
#[case(
    "can_update_category_description",
    TROUSERS_PID,
    serde_json::json!({
        "description": "Casual and formal trousers"
    })
)]
#[case(
    "can_update_category_name",
    TROUSERS_PID,
    serde_json::json!({
        "name": "Pants"
    })
)]
#[case(
    "can_update_category_name_image_and_description",
    TROUSERS_PID,
    serde_json::json!({
        "name": "Chinos",
        "imageLink": "https://cdn.example.com/categories/chinos.png",
        "description": "Smart casual trousers"
    })
)]
#[case(
    "cannot_update_category_when_name_already_exists",
    TROUSERS_PID,
    serde_json::json!({
        "name": "T-shirts",
        "description": "Duplicate seeded category"
    })
)]
#[case(
    "cannot_update_category_when_pid_does_not_exist",
    MISSING_PID,
    serde_json::json!({
        "name": "Outerwear",
        "description": "Jackets and coats"
    })
)]
#[case(
    "cannot_update_category_when_name_is_invalid",
    TROUSERS_PID,
    serde_json::json!({
        "name": "Pants+"
    })
)]
#[case(
    "cannot_update_category_when_image_link_is_invalid",
    TROUSERS_PID,
    serde_json::json!({
        "imageLink": "not-a-url"
    })
)]
#[tokio::test]
#[serial]
async fn can_update_category(
    #[case] test_name: &str,
    #[case] pid: &str,
    #[case] params: serde_json::Value,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");
        allow_category_writes(ctx.db()).await;

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .patch(&format!("/categories/{pid}"))
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
#[case("can_delete_category", TROUSERS_PID)]
#[case("cannot_delete_category_when_pid_does_not_exist", MISSING_PID)]
#[case("cannot_delete_category_when_pid_is_invalid", "not-a-uuid")]
#[tokio::test]
#[serial]
async fn can_delete_category(#[case] test_name: &str, #[case] pid: &str) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");
        allow_category_writes(ctx.db()).await;

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .delete(&format!("/categories/{pid}"))
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
#[case("cannot_create_category_without_credentials", "POST", "/categories")]
#[case(
    "cannot_update_category_without_credentials",
    "PATCH",
    "/categories/00b92bcb-cc7a-4a2b-bd80-e9c1b40d1c46"
)]
#[case(
    "cannot_delete_category_without_credentials",
    "DELETE",
    "/categories/00b92bcb-cc7a-4a2b-bd80-e9c1b40d1c46"
)]
#[tokio::test]
#[serial]
async fn cannot_write_categories_without_credentials(
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
            "name": "Accessories",
            "imageLink": "https://cdn.example.com/categories/accessories.png"
        });

        let response = match method {
            "POST" => server.post(path).json(&body).await,
            "PATCH" => server.patch(path).json(&body).await,
            "DELETE" => server.delete(path).await,
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
#[case("cannot_create_category_without_permission")]
#[tokio::test]
#[serial]
async fn cannot_write_category_without_permission(#[case] test_name: &str) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);
        let body = serde_json::json!({
            "name": "Accessories",
            "imageLink": "https://cdn.example.com/categories/accessories.png"
        });

        let response = server
            .post("/categories")
            .add_header(auth_header, auth_value)
            .json(&body)
            .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        })
    })
    .await;
}
