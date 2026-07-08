use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::models::Attribute;
use uuid::Uuid;

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("attributes");
        settings.set_snapshot_path("snapshots/attributes");
        let _guard = settings.bind_to_scope();
    };
}

#[rstest]
#[case("can_create_attribute_successful", "texture")]
#[case("cannot_create_attribute_if_name_exists", "colour")]
#[case("cannot_create_attribute_if_name_is_empty", "   ")]
#[tokio::test]
#[serial]
async fn can_create_attribute(#[case] test_name: &str, #[case] name: &str) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let result = Attribute::create(ctx.db(), name).await;

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
    "can_update_attribute_successful",
    "757aeb5b-0a44-4ef7-9d7f-eb4f8ea23103",
    "texture"
)]
#[case(
    "cannot_update_attribute_if_name_exists",
    "757aeb5b-0a44-4ef7-9d7f-eb4f8ea23103",
    "colour"
)]
#[case(
    "cannot_update_attribute_if_pid_not_exists",
    "00000000-0000-0000-0000-000000000000",
    "material"
)]
#[tokio::test]
#[serial]
async fn can_update_attribute(#[case] test_name: &str, #[case] pid: &str, #[case] name: &str) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let pid = Uuid::parse_str(pid).expect("Failed to parse str to UUID");

    let result = Attribute::update(ctx.db(), pid, name).await;

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
#[case("can_find_colour_by_pid", "3516b141-5ffc-402a-8f7c-557d277b3101")]
#[case("can_find_size_by_pid", "23d5df44-6acf-41a7-96fc-bf0160653102")]
#[case(
    "cannot_find_attribute_if_pid_not_exists",
    "00000000-0000-0000-0000-000000000000"
)]
#[tokio::test]
#[serial]
async fn can_find_attribute_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let pid = Uuid::parse_str(pid).expect("Failed to parse str to UUID");

    let result = Attribute::find_by_pid(ctx.db(), pid).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_colour_by_name", "colour")]
#[case("can_find_size_by_name", "size")]
#[case("cannot_find_attribute_if_name_not_exists", "texture")]
#[tokio::test]
#[serial]
async fn can_find_attribute_by_name(#[case] test_name: &str, #[case] name: &str) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let result = Attribute::find_by_name(ctx.db(), name).await;

    assert_debug_snapshot!(test_name, result);
}

#[tokio::test]
#[serial]
async fn can_find_all_attributes() {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let result = Attribute::find_all(ctx.db()).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_id().to_vec());
            filters
        }
    }, {
            assert_debug_snapshot!("can_find_all_attributes", result)
    });
}

#[tokio::test]
#[serial]
async fn can_find_all_attributes_with_values() {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let result = Attribute::find_all_with_values(ctx.db()).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_id().to_vec());
            filters
        }
    }, {
            assert_debug_snapshot!("can_find_all_attributes_with_values", result)
    });
}
