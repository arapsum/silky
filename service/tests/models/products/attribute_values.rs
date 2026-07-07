use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::models::AttributeValue;
use uuid::Uuid;

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("attribute_values");
        settings.set_snapshot_path("snapshots/attribute_values");
        let _guard = settings.bind_to_scope();
    };
}

#[rstest]
#[case("can_create_attribute_value_successful", 201, "Gold")]
#[case(
    "will_return_attribute_value_if_value_exists_for_attribute",
    201,
    "Black"
)]
#[case("can_create_attribute_value_and_trim_value", 201, "  Charcoal  ")]
#[tokio::test]
#[serial]
async fn can_create_attribute_value(
    #[case] test_name: &str,
    #[case] attribute_id: i32,
    #[case] value: &str,
) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let result = AttributeValue::create(ctx.db(), attribute_id, value).await;

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

#[rstest]
#[case(
    "can_find_black_attribute_value_by_pid",
    "9ae5af4e-5174-48f5-ae04-1d74e68f3201"
)]
#[case(
    "can_find_medium_attribute_value_by_pid",
    "4cd5e3c1-06e9-485e-bc62-ea9309d93205"
)]
#[case(
    "cannot_find_attribute_value_if_pid_not_exists",
    "00000000-0000-0000-0000-000000000000"
)]
#[tokio::test]
#[serial]
async fn can_find_attribute_value_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let pid = Uuid::parse_str(pid).expect("Failed to parse str to UUID");

    let result = AttributeValue::find_by_pid(ctx.db(), pid).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_black_attribute_value_by_value", "Black")]
#[case("can_find_marine_blue_attribute_value_by_value", "Marine Blue")]
#[case("can_find_attribute_value_by_trimmed_value", "  Leather  ")]
#[case("cannot_find_attribute_value_if_value_not_exists", "Magenta")]
#[tokio::test]
#[serial]
async fn can_find_attribute_value_by_value(#[case] test_name: &str, #[case] value: &str) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let result = AttributeValue::find_by_value(ctx.db(), value).await;

    assert_debug_snapshot!(test_name, result);
}
