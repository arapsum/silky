use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::models::{NewProductOption, ProductOption};

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("product_options");
        settings.set_snapshot_path("snapshots/product_options");
        let _guard = settings.bind_to_scope();
    };
}

#[rstest]
#[case("can_create_product_option_successful", 203, 203, Some(3))]
#[case("can_create_product_option_without_display_order", 202, 203, None)]
#[case("cannot_create_product_option_when_relation_exists", 201, 201, Some(1))]
#[case(
    "cannot_create_product_option_when_product_does_not_exist",
    999,
    203,
    Some(1)
)]
#[case(
    "cannot_create_product_option_when_attribute_does_not_exist",
    203,
    999,
    Some(1)
)]
#[tokio::test]
#[serial]
async fn can_create_product_option(
    #[case] test_name: &str,
    #[case] product_id: i32,
    #[case] attribute_id: i32,
    #[case] display_order: Option<i32>,
) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let params = NewProductOption::new(product_id, attribute_id, display_order);

    let result = ProductOption::create(ctx.db(), &params).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_id().to_vec());
            filters
        }
    }, {
            assert_debug_snapshot!(test_name, result)
    });
}
