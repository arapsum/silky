use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serde_json::{Value, json};
use serial_test::serial;
use service::{
    models::{ModelError, Product},
    schemas::{ProductListQuery, StockStatus},
};
use uuid::Uuid;

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("product_fetch");
        settings.set_snapshot_path("snapshots/product_fetch");
        let _guard = settings.bind_to_scope();
    };
}

fn query(params: Value) -> ProductListQuery {
    serde_json::from_value(params).expect("Failed to parse product list query")
}

fn default_query() -> ProductListQuery {
    query(json!({}))
}

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("Failed to parse UUID")
}

fn snapshot_filters() -> Vec<(&'static str, &'static str)> {
    let mut filters = cleanup_uuid().to_vec();
    filters.extend(cleanup_date().to_vec());
    filters.extend(cleanup_id());
    filters
}

async fn mark_product_deleted(db: &sqlx::PgPool, product_id: i32) {
    sqlx::query("UPDATE products SET deleted_at = NOW() WHERE id = $1")
        .bind(product_id)
        .execute(db)
        .await
        .expect("Failed to soft-delete product");
}

async fn clear_product_stock(db: &sqlx::PgPool, product_id: i32) {
    sqlx::query("UPDATE product_variants SET stock_quantity = 0 WHERE product_id = $1")
        .bind(product_id)
        .execute(db)
        .await
        .expect("Failed to clear product stock");
}

#[rstest]
#[case("can_find_products_with_default_pagination", default_query())]
#[case(
    "can_find_products_with_second_page",
    query(json!({ "limit": 2, "page": 2 }))
)]
#[case(
    "can_find_products_with_clamped_large_limit",
    query(json!({ "limit": 100, "page": 1 }))
)]
#[case(
    "can_find_products_by_search",
    query(json!({ "search": "leather" }))
)]
#[case(
    "can_find_products_by_category_id",
    query(json!({ "categoryId": 103 }))
)]
#[case(
    "can_find_products_by_category_slug",
    query(json!({ "categorySlug": "shoes" }))
)]
#[case(
    "can_find_products_by_sku",
    query(json!({ "sku": "SNEAKER-WHT-42" }))
)]
#[case(
    "can_find_products_by_price_range",
    query(json!({ "minPrice": "50.00", "maxPrice": "90.00" }))
)]
#[case(
    "can_find_in_stock_products",
    query(json!({ "stockStatus": StockStatus::InStock }))
)]
#[tokio::test]
#[serial]
async fn can_find_products(#[case] test_name: &str, #[case] query: ProductListQuery) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let result = Product::find_list(ctx.db(), &query).await;

    with_settings!({
        filters => snapshot_filters()
    }, {
        assert_debug_snapshot!(test_name, result)
    });
}

#[tokio::test]
#[serial]
async fn can_find_out_of_stock_products() {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");
    clear_product_stock(ctx.db(), 203).await;

    let result = Product::find_list(ctx.db(), &query(json!({ "stockStatus": "outOfStock" }))).await;

    with_settings!({
        filters => snapshot_filters()
    }, {
        assert_debug_snapshot!("can_find_out_of_stock_products", result)
    });
}

#[tokio::test]
#[serial]
async fn can_include_deleted_products() {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");
    mark_product_deleted(ctx.db(), 202).await;

    let without_deleted = Product::find_list(ctx.db(), &default_query()).await;
    let with_deleted =
        Product::find_list(ctx.db(), &query(json!({ "includeDeleted": true }))).await;

    with_settings!({
        filters => snapshot_filters()
    }, {
        assert_debug_snapshot!("can_include_deleted_products", (without_deleted, with_deleted))
    });
}

#[tokio::test]
#[serial]
async fn can_find_product_detail_by_pid() {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let result =
        Product::find_detail_by_pid(ctx.db(), uuid("6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001")).await;

    with_settings!({
        filters => snapshot_filters()
    }, {
        assert_debug_snapshot!("can_find_product_detail_by_pid", result)
    });
}

#[tokio::test]
#[serial]
async fn cannot_find_product_detail_when_pid_does_not_exist() {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let result =
        Product::find_detail_by_pid(ctx.db(), uuid("11111111-1111-4111-8111-111111111111")).await;

    assert_debug_snapshot!("cannot_find_product_detail_when_pid_does_not_exist", result);
}

#[rstest]
#[case("4532debd-67fe-4070-b36c-d5b9392b3002", true)]
#[case("11111111-1111-4111-8111-111111111111", false)]
#[tokio::test]
#[serial]
async fn can_delete_product(#[case] pid: &str, #[case] should_exist: bool) {
    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let pid = uuid(pid);
    let result = Product::delete(ctx.db(), pid).await;

    if should_exist {
        let product = result.expect("Product should be soft-deleted");
        assert!(product.deleted_at().is_some());

        let detail = Product::find_detail_by_pid(ctx.db(), pid).await;
        assert!(matches!(detail, Err(ModelError::EntityNotFound)));
    } else {
        assert!(matches!(result, Err(ModelError::EntityNotFound)));
    }
}
