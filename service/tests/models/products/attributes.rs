use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use serial_test::serial;
use service::models::Attribute;
use uuid::Uuid;

use crate::{boot_test, seed_data};

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
