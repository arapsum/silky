use std::{borrow::Cow, collections::BTreeMap};

use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use rust_decimal::Decimal;
use serial_test::serial;
use service::{
    models::{ModelError, Product, Tag},
    schemas::{CreateProduct, CreateProductPicture, CreateProductVariant, CreateVariantOption},
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
        settings.set_snapshot_suffix("product_create");
        settings.set_snapshot_path("snapshots/product_create");
        let _guard = settings.bind_to_scope();
    };
}

#[derive(Debug, Clone)]
struct ProductCreateScenario {
    product: CreateProduct<'static>,
    product_name: &'static str,
    variant_sku: Option<&'static str>,
}

impl ProductCreateScenario {
    const fn new(
        product: CreateProduct<'static>,
        product_name: &'static str,
        variant_sku: Option<&'static str>,
    ) -> Self {
        Self {
            product,
            product_name,
            variant_sku,
        }
    }
}

fn base_product() -> ProductCreateScenario {
    ProductCreateScenario::new(
        CreateProduct::new(
            103,
            Cow::Borrowed("Minimal Product"),
            Some(Cow::Borrowed("A product without setup records")),
            None,
            None,
        )
        .with_information(BTreeMap::from([
            ("Material".to_string(), "Cotton".to_string()),
            ("Origin".to_string(), "Kenya".to_string()),
        ])),
        "Minimal Product",
        None,
    )
}

fn aggregate_product() -> ProductCreateScenario {
    ProductCreateScenario::new(
        CreateProduct::new(
            103,
            Cow::Borrowed("Aggregate Product"),
            Some(Cow::Borrowed("A product with complete setup records")),
            Some(vec![CreateProductPicture::new(
                Cow::Borrowed("https://cdn.example.com/products/aggregate-main.png"),
                Some(1),
            )]),
            Some(vec![CreateProductVariant::new(
                Cow::Borrowed("AGGREGATE-BLK-M"),
                Decimal::new(5999, 2),
                12,
                true,
                Some(vec![
                    CreateVariantOption::new(201, 203, Some(1)),
                    CreateVariantOption::new(202, 205, Some(2)),
                ]),
                Some(vec![CreateProductPicture::new(
                    Cow::Borrowed("https://cdn.example.com/products/aggregate-black.png"),
                    Some(2),
                )]),
            )]),
        ),
        "Aggregate Product",
        Some("AGGREGATE-BLK-M"),
    )
}

fn duplicate_product_name() -> ProductCreateScenario {
    ProductCreateScenario::new(
        CreateProduct::new(
            103,
            Cow::Borrowed("Classic Cotton T-shirt"),
            None,
            None,
            None,
        ),
        "Classic Cotton T-shirt",
        None,
    )
}

fn missing_category() -> ProductCreateScenario {
    ProductCreateScenario::new(
        CreateProduct::new(
            999,
            Cow::Borrowed("Missing Category Product"),
            None,
            None,
            None,
        ),
        "Missing Category Product",
        None,
    )
}

fn missing_attribute_value() -> ProductCreateScenario {
    ProductCreateScenario::new(
        CreateProduct::new(
            103,
            Cow::Borrowed("Rollback Attribute Product"),
            None,
            None,
            Some(vec![CreateProductVariant::new(
                Cow::Borrowed("ROLLBACK-ATTRIBUTE"),
                Decimal::new(1999, 2),
                4,
                true,
                Some(vec![CreateVariantOption::new(201, 999, Some(1))]),
                None,
            )]),
        ),
        "Rollback Attribute Product",
        Some("ROLLBACK-ATTRIBUTE"),
    )
}

async fn product_count(db: &sqlx::PgPool, name: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM products WHERE name = $1")
        .bind(name)
        .fetch_one(db)
        .await
        .expect("Failed to count products")
}

async fn variant_count(db: &sqlx::PgPool, sku: Option<&str>) -> i64 {
    let Some(sku) = sku else {
        return 0;
    };

    sqlx::query_scalar("SELECT COUNT(*) FROM product_variants WHERE sku = $1")
        .bind(sku)
        .fetch_one(db)
        .await
        .expect("Failed to count product variants")
}

#[tokio::test]
#[serial]
async fn creates_product_tag_assignments_atomically() {
    let ctx = boot_test().await.expect("Failed to boot test");
    seed_data(ctx.db()).await.expect("Failed to seed data");
    sqlx::query(
        "DELETE FROM products WHERE name IN ('Tagged Creation Product', 'Invalid Tagged Product')",
    )
    .execute(ctx.db())
    .await
    .expect("Failed to clean tagged products");
    sqlx::query("DELETE FROM tags WHERE name IN ('Creation casual', 'Creation summer')")
        .execute(ctx.db())
        .await
        .expect("Failed to clean tag creation fixtures");

    let casual = Tag::create(ctx.db(), "Creation casual")
        .await
        .expect("Failed to create casual tag");
    let summer = Tag::create(ctx.db(), "Creation summer")
        .await
        .expect("Failed to create summer tag");
    let params = CreateProduct::new(
        103,
        Cow::Borrowed("Tagged Creation Product"),
        None,
        None,
        None,
    )
    .with_tag_pids(vec![casual.pid(), summer.pid(), casual.pid()]);

    let result = Product::create(ctx.db(), &params)
        .await
        .expect("Failed to create tagged product");
    assert_eq!(result.tags.len(), 2);
    assert_eq!(
        Tag::find_by_product(ctx.db(), result.product.pid())
            .await
            .expect("Failed to load created product tags")
            .len(),
        2
    );

    let invalid_params = CreateProduct::new(
        103,
        Cow::Borrowed("Invalid Tagged Product"),
        None,
        None,
        None,
    )
    .with_tag_pids(vec![Uuid::nil()]);
    let invalid_result = Product::create(ctx.db(), &invalid_params).await;
    assert!(matches!(
        invalid_result,
        Err(ModelError::InvalidReference(_))
    ));
    assert_eq!(product_count(ctx.db(), "Invalid Tagged Product").await, 0);

    sqlx::query("DELETE FROM products WHERE name = 'Tagged Creation Product'")
        .execute(ctx.db())
        .await
        .expect("Failed to clean tagged product");
    Tag::delete(ctx.db(), casual.pid())
        .await
        .expect("Failed to clean casual tag");
    Tag::delete(ctx.db(), summer.pid())
        .await
        .expect("Failed to clean summer tag");
}

#[rstest]
#[case("can_create_base_product", base_product())]
#[case("can_create_aggregate_product", aggregate_product())]
#[case("cannot_create_product_when_name_exists", duplicate_product_name())]
#[case(
    "cannot_create_product_when_category_does_not_exist",
    missing_category()
)]
#[case(
    "rolls_back_product_when_nested_attribute_value_does_not_exist",
    missing_attribute_value()
)]
#[tokio::test]
#[serial]
async fn can_create_product(#[case] test_name: &str, #[case] scenario: ProductCreateScenario) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let result = Product::create(ctx.db(), &scenario.product).await;
    let product_count = product_count(ctx.db(), scenario.product_name).await;
    let variant_count = variant_count(ctx.db(), scenario.variant_sku).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_id().to_vec());
            filters
        }
    }, {
            assert_debug_snapshot!(test_name, (result, product_count, variant_count))
    });
}
