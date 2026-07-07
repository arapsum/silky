use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use rust_decimal::Decimal;
use serial_test::serial;
use service::models::{NewVariant, ProductVariant};

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("product_variants");
        settings.set_snapshot_path("snapshots/product_variants");
        let _guard = settings.bind_to_scope();
    };
}

#[rstest]
#[case(
    "can_create_product_variant_successful",
    203,
    "BAG-LEATHER-STD",
    "129.99",
    15,
    false
)]
#[case(
    "cannot_create_product_variant_when_sku_already_exists",
    203,
    "TSHIRT-BLK-M",
    "24.99",
    15,
    false
)]
#[case(
    "cannot_create_product_variant_when_product_does_not_exist",
    999,
    "MISSING-PRODUCT-SKU",
    "19.99",
    10,
    false
)]
#[case(
    "cannot_create_product_variant_when_price_is_negative",
    203,
    "BAG-NEGATIVE-PRICE",
    "-1.00",
    10,
    false
)]
#[case(
    "cannot_create_product_variant_when_stock_is_negative",
    203,
    "BAG-NEGATIVE-STOCK",
    "19.99",
    -1,
    false
)]
#[case(
    "cannot_create_product_variant_when_default_variant_exists",
    201,
    "TSHIRT-SECOND-DEFAULT",
    "24.99",
    10,
    true
)]
#[tokio::test]
#[serial]
async fn can_create_product_variant(
    #[case] test_name: &str,
    #[case] product_id: i32,
    #[case] sku: &str,
    #[case] price: &str,
    #[case] stock_quantity: i32,
    #[case] is_default: bool,
) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let price = price.parse::<Decimal>().expect("Failed to parse decimal");
    let params = NewVariant::new(
        product_id,
        sku.to_string(),
        price,
        stock_quantity,
        is_default,
    );

    let result = ProductVariant::create(ctx.db(), &params).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_id().to_vec());
            filters.extend([
                (r"line: Some\(\n\s+\d+,\n\s+\)", "line: Some(LINE)"),
                (r#"file: Some\(\n\s+"[^"]+",\n\s+\)"#, "file: Some(\"FILE\")"),
                (r"DATE \d{2}:\d{2}:\d{2}\.\d+\+00", "DATE"),
            ]);
            filters
        }
    }, {
            assert_debug_snapshot!(test_name, result)
    });
}
