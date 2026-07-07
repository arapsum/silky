use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::models::{NewVariantAttributeValue, VariantAttributeValue};

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("variant_attribute_values");
        settings.set_snapshot_path("snapshots/variant_attribute_values");
        let _guard = settings.bind_to_scope();
    };
}

#[rstest]
#[case("can_create_variant_attribute_value_successful", 201, 203, 209)]
#[case(
    "will_return_variant_attribute_value_if_relation_already_exists",
    206,
    201,
    203
)]
#[case("errors_when_variant_id_does_not_exist", 999, 203, 209)]
#[case("errors_when_attribute_id_does_not_exist", 202, 999, 209)]
#[case("errors_when_attribute_value_id_does_not_exist", 202, 203, 999)]
#[tokio::test]
#[serial]
async fn can_create_variant_attribute_value(
    #[case] test_name: &str,
    #[case] variant_id: i32,
    #[case] attribute_id: i32,
    #[case] attribute_value_id: i32,
) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let params = NewVariantAttributeValue::new(variant_id, attribute_id, attribute_value_id);

    let result = VariantAttributeValue::create(ctx.db(), &params).await;

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
