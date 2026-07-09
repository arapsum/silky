use std::borrow::Cow;

use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use serial_test::serial;
use service::{
    models::{ModelError, Product},
    schemas::{
        CreateProductPicture, CreateProductVariant, CreateVariantOption, ProductListQuery,
        StockStatus, UpdateProduct, UpdateProductPicture, UpdateProductVariant,
    },
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

fn decimal(value: &str) -> Decimal {
    value.parse().expect("Failed to parse decimal")
}

fn update_product(params: Value) -> UpdateProduct {
    serde_json::from_value(params).expect("Failed to parse update product")
}

fn update_variant(params: Value) -> UpdateProductVariant {
    serde_json::from_value(params).expect("Failed to parse update variant")
}

fn update_picture(params: Value) -> UpdateProductPicture {
    serde_json::from_value(params).expect("Failed to parse update picture")
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

#[tokio::test]
#[serial]
async fn can_update_base_product() {
    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let product = Product::update(
        ctx.db(),
        uuid("6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001"),
        &update_product(json!({
            "categoryId": 103,
            "name": "Updated Cotton T-shirt",
            "description": null
        })),
    )
    .await
    .expect("Failed to update product");

    assert_eq!(product.name, "Updated Cotton T-shirt");
    assert_eq!(product.category.id, 103);
    assert!(product.description.is_none());
}

#[tokio::test]
#[serial]
async fn can_create_variant_with_exact_product_options() {
    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let params = CreateProductVariant::new(
        Cow::Borrowed("TSHIRT-GRN-S"),
        decimal("25.99"),
        5,
        true,
        Some(vec![
            CreateVariantOption::new(201, 203, Some(1)),
            CreateVariantOption::new(202, 204, Some(2)),
        ]),
        None,
    );

    let product = Product::create_variant(
        ctx.db(),
        uuid("6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001"),
        &params,
    )
    .await
    .expect("Failed to create variant");

    let variant = product
        .variants
        .iter()
        .find(|variant| variant.sku == "TSHIRT-GRN-S")
        .expect("created variant missing");

    assert!(!variant.is_default);
    assert_eq!(variant.options.len(), 2);
}

#[tokio::test]
#[serial]
async fn cannot_create_variant_with_missing_product_option() {
    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let params = CreateProductVariant::new(
        Cow::Borrowed("TSHIRT-MISSING-OPTION"),
        decimal("25.99"),
        5,
        false,
        Some(vec![CreateVariantOption::new(201, 203, Some(1))]),
        None,
    );

    let result = Product::create_variant(
        ctx.db(),
        uuid("6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001"),
        &params,
    )
    .await;

    assert!(matches!(result, Err(ModelError::InvalidInput(_))));
}

#[tokio::test]
#[serial]
async fn can_update_variant_without_changing_options() {
    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let product = Product::update_variant(
        ctx.db(),
        uuid("6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001"),
        uuid("6bf1821a-35d9-42cc-a2d8-b42fcf4f3402"),
        &update_variant(json!({
            "sku": "TSHIRT-BLU-L-UPDATED",
            "price": "29.99",
            "stockQuantity": 8
        })),
    )
    .await
    .expect("Failed to update variant");

    let variant = product
        .variants
        .iter()
        .find(|variant| variant.pid == uuid("6bf1821a-35d9-42cc-a2d8-b42fcf4f3402"))
        .expect("updated variant missing");

    assert_eq!(variant.sku, "TSHIRT-BLU-L-UPDATED");
    assert_eq!(variant.price, decimal("29.99"));
    assert_eq!(variant.stock_quantity, 8);
}

#[tokio::test]
#[serial]
async fn can_set_default_variant_and_reject_deleting_default() {
    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let product = Product::set_default_variant(
        ctx.db(),
        uuid("6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001"),
        uuid("6bf1821a-35d9-42cc-a2d8-b42fcf4f3402"),
    )
    .await
    .expect("Failed to set default variant");

    assert!(product.variants.iter().any(|variant| variant.pid
        == uuid("6bf1821a-35d9-42cc-a2d8-b42fcf4f3402")
        && variant.is_default));

    let result = Product::delete_variant(
        ctx.db(),
        uuid("6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001"),
        uuid("6bf1821a-35d9-42cc-a2d8-b42fcf4f3402"),
    )
    .await;

    assert!(matches!(result, Err(ModelError::InvalidInput(_))));
}

#[tokio::test]
#[serial]
async fn can_soft_delete_non_default_variant() {
    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let product = Product::delete_variant(
        ctx.db(),
        uuid("6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001"),
        uuid("6bf1821a-35d9-42cc-a2d8-b42fcf4f3402"),
    )
    .await
    .expect("Failed to delete variant");

    assert!(
        product
            .variants
            .iter()
            .all(|variant| variant.pid != uuid("6bf1821a-35d9-42cc-a2d8-b42fcf4f3402"))
    );
}

#[tokio::test]
#[serial]
async fn can_manage_product_picture() {
    let ctx = boot_test().await.expect("Failed to boot test!");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let picture = Product::add_picture(
        ctx.db(),
        uuid("6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001"),
        &CreateProductPicture::new(
            Cow::Borrowed("https://cdn.example.com/products/tshirt-main.png"),
            Some(3),
        ),
    )
    .await
    .expect("Failed to add picture");

    let updated = Product::update_picture_order(
        ctx.db(),
        uuid("6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001"),
        picture.pid(),
        &update_picture(json!({ "displayOrder": 1 })),
    )
    .await
    .expect("Failed to update picture");

    assert_eq!(updated.display_order(), Some(1));

    let deleted = Product::delete_picture(
        ctx.db(),
        uuid("6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001"),
        picture.pid(),
    )
    .await
    .expect("Failed to delete picture");

    assert_eq!(deleted.pid(), picture.pid());
}
