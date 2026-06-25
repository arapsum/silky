use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use serial_test::serial;
use service::models::Permission;
use uuid::Uuid;

use crate::boot_test;

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("permissions");
        settings.set_snapshot_path("snapshots/permissions");
        let _guard = settings.bind_to_scope();
    };
}

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("Failed to parse UUID")
}

#[rstest]
#[case("can_seed_permissions_from_json", "permissions.json")]
#[case(
    "when_seed_file_does_not_exist_permission_seeding_fails",
    "missing-permissions.json"
)]
#[tokio::test]
#[serial]
async fn can_seed_permissions(#[case] test_name: &str, #[case] file: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    let result = Permission::seed_data(ctx.db(), file).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "can_find_roles_read_permission_by_pid",
    "9e230b11-cb47-4fe8-8bc0-5185fd9f9bb2"
)]
#[case(
    "can_find_roles_write_permission_by_pid",
    "3fdbf302-c05a-4f58-82d8-1f7efa9ea8a6"
)]
#[case(
    "can_find_permissions_read_permission_by_pid",
    "a0199d51-0147-477f-8778-070785ee81f3"
)]
#[tokio::test]
#[serial]
async fn can_find_permission_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Permission::seed_data(ctx.db(), "permissions.json")
        .await
        .expect("Failed to seed permissions");

    let result = Permission::find_by_pid(ctx.db(), uuid(pid)).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "cannot_find_permission_when_pid_does_not_exist",
    "00000000-0000-0000-0000-000000000000"
)]
#[tokio::test]
#[serial]
async fn cannot_find_permission_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Permission::seed_data(ctx.db(), "permissions.json")
        .await
        .expect("Failed to seed permissions");

    let result = Permission::find_by_pid(ctx.db(), uuid(pid)).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_permission_list")]
#[tokio::test]
#[serial]
async fn can_find_permission_list(#[case] test_name: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Permission::seed_data(ctx.db(), "permissions.json")
        .await
        .expect("Failed to seed permissions");

    let result = Permission::find_list(ctx.db()).await;

    assert_debug_snapshot!(test_name, result);
}
