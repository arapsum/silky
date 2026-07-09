use axum::http::{HeaderValue, StatusCode};
use axum_test::TestServer;
use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::access_control::permissions;

use crate::utils;

macro_rules! configure_insta {
    ($(expr:expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/products");
        settings.set_snapshot_suffix("products");
        let _guard = settings.bind_to_scope();
    };
}

async fn access_token(server: &TestServer) -> HeaderValue {
    access_token_for(server, "john.doe@acme.com").await
}

async fn access_token_for(server: &TestServer, email: &str) -> HeaderValue {
    let params = serde_json::json!({
        "email": email,
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

async fn allow_product_writes(db: &sqlx::PgPool) {
    assign_role(db, "john.doe@acme.com", "administrator").await;
    grant_permission(db, "administrator", permissions::products::CREATE.as_str()).await;
}

async fn allow_product_deletes(db: &sqlx::PgPool) {
    assign_role(db, "john.doe@acme.com", "administrator").await;
    grant_permission(db, "administrator", permissions::products::DELETE.as_str()).await;
}

async fn allow_product_reads(db: &sqlx::PgPool) {
    assign_role(db, "john.doe@acme.com", "administrator").await;
    grant_permission(db, "administrator", permissions::products::READ.as_str()).await;
}

fn response_filters() -> Vec<(&'static str, &'static str)> {
    let mut filters = utils::cleanup_date().to_vec();
    filters.extend(utils::cleanup_uuid().to_vec());
    filters.extend(utils::cleanup_headers());
    filters.push((r#""id":\d+"#, r#""id":ID"#));
    filters.push(("DATEZ", "DATE"));
    filters
}

fn aggregate_body() -> serde_json::Value {
    serde_json::json!({
        "categoryId": 103,
        "name": "API Aggregate Product",
        "description": "Created from the aggregate API",
        "pictures": [
            {
                "imageLink": "https://cdn.example.com/products/api-aggregate-main.png",
                "displayOrder": 1
            }
        ],
        "variants": [
            {
                "sku": "API-AGGREGATE-BLK-M",
                "price": "59.99",
                "stockQuantity": 12,
                "isDefault": true,
                "options": [
                    { "attributeId": 201, "attributeValueId": 203, "displayOrder": 1 },
                    { "attributeId": 202, "attributeValueId": 205, "displayOrder": 2 }
                ],
                "pictures": [
                    {
                        "imageLink": "https://cdn.example.com/products/api-aggregate-black.png",
                        "displayOrder": 2
                    }
                ]
            }
        ]
    })
}

#[rstest]
#[case("can_create_product_with_setup", aggregate_body())]
#[case(
    "cannot_create_product_when_name_is_invalid",
    serde_json::json!({
        "categoryId": 103,
        "name": " ",
        "variants": []
    })
)]
#[case(
    "cannot_create_product_when_picture_url_is_invalid",
    serde_json::json!({
        "categoryId": 103,
        "name": "Invalid Picture Product",
        "pictures": [
            {
                "imageLink": "not-a-url",
                "displayOrder": 1
            }
        ]
    })
)]
#[case(
    "cannot_create_product_when_category_does_not_exist",
    serde_json::json!({
        "categoryId": 999,
        "name": "Missing Category Product"
    })
)]
#[tokio::test]
#[serial]
async fn can_create_product(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");
        allow_product_writes(ctx.db()).await;

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .post("/products")
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

#[tokio::test]
#[serial]
async fn cannot_create_product_without_credentials() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server.post("/products").json(&aggregate_body()).await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("cannot_create_product_without_credentials", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_list_products() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server
            .get("/products?search=leather&categorySlug=shoes&limit=10&page=1")
            .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("can_list_products", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_list_products_without_credentials() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server.get("/products").await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("can_list_products_without_credentials", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_list_products_without_permission() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token_for(&server, "jane.smith@globex.com").await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .get("/products")
            .add_header(auth_header, auth_value)
            .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("can_list_products_without_permission", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_list_products_with_invalid_query() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server.get("/products?limit=0").await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("cannot_list_products_with_invalid_query", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_get_product_detail() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server
            .get("/products/6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001")
            .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("can_get_product_detail", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_get_product_detail_without_credentials() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server
            .get("/products/6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001")
            .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("can_get_product_detail_without_credentials", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_get_product_detail_without_permission() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token_for(&server, "jane.smith@globex.com").await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .get("/products/6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001")
            .add_header(auth_header, auth_value)
            .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("can_get_product_detail_without_permission", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_get_product_detail_when_pid_does_not_exist() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");
        allow_product_reads(ctx.db()).await;

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .get("/products/11111111-1111-4111-8111-111111111111")
            .add_header(auth_header, auth_value)
            .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("cannot_get_product_detail_when_pid_does_not_exist", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_create_product_without_permission() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token_for(&server, "jane.smith@globex.com").await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .post("/products")
            .add_header(auth_header, auth_value)
            .json(&aggregate_body())
            .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("cannot_create_product_without_permission", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[rstest]
#[case("4532debd-67fe-4070-b36c-d5b9392b3002", StatusCode::NO_CONTENT)]
#[case("11111111-1111-4111-8111-111111111111", StatusCode::NOT_FOUND)]
#[case("not-a-uuid", StatusCode::BAD_REQUEST)]
#[tokio::test]
#[serial]
async fn can_delete_product(#[case] pid: &str, #[case] expected_status: StatusCode) {
    crate::request(|server, ctx| async move {
        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");
        allow_product_deletes(ctx.db()).await;

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .delete(&format!("/products/{pid}"))
            .add_header(auth_header, auth_value)
            .await;

        assert_eq!(response.status_code(), expected_status);

        if expected_status == StatusCode::NO_CONTENT {
            let deleted_at =
                sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::FixedOffset>>>(
                    "SELECT deleted_at FROM products WHERE pid = $1",
                )
                .bind(uuid::Uuid::parse_str(pid).expect("valid product pid"))
                .fetch_one(ctx.db())
                .await
                .expect("Failed to fetch deleted product");

            assert!(deleted_at.is_some());
        }
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_delete_product_without_credentials() {
    crate::request(|server, ctx| async move {
        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server
            .delete("/products/4532debd-67fe-4070-b36c-d5b9392b3002")
            .await;

        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_delete_product_without_permission() {
    crate::request(|server, ctx| async move {
        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token_for(&server, "jane.smith@globex.com").await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .delete("/products/4532debd-67fe-4070-b36c-d5b9392b3002")
            .add_header(auth_header, auth_value)
            .await;

        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_list_product_attributes_with_values() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");
        allow_product_reads(ctx.db()).await;

        let token = access_token(&server).await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .get("/products/attributes")
            .add_header(auth_header, auth_value)
            .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("can_list_product_attributes_with_values", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_list_product_attributes_without_credentials() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server.get("/products/attributes").await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("cannot_list_product_attributes_without_credentials", (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_list_product_attributes_without_permission() {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token_for(&server, "jane.smith@globex.com").await;
        let (auth_header, auth_value) = utils::auth_header(token);

        let response = server
            .get("/products/attributes")
            .add_header(auth_header, auth_value)
            .await;

        with_settings!({
            filters => response_filters()
        }, {
            assert_debug_snapshot!("cannot_list_product_attributes_without_permission", (response.status_code(), response.text()))
        })
    })
    .await;
}
