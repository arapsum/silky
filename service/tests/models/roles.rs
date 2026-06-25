use std::borrow::Cow;

use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::{
    models::Role,
    schemas::{NewRole, UpdateRole},
};
use uuid::Uuid;

use crate::{
    boot_test,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("roles");
        settings.set_snapshot_path("snapshots/roles");
        let _guard = settings.bind_to_scope();
    };
}

fn new_role(name: String, description: Option<String>) -> NewRole<'static> {
    NewRole::new(Cow::Owned(name), description.map(Cow::Owned))
}

fn update_role(name: Option<String>, description: Option<String>) -> UpdateRole<'static> {
    UpdateRole::new(name.map(Cow::Owned), description.map(Cow::Owned))
}

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("Failed to parse UUID")
}

#[rstest]
#[case(
    "can_create_role_with_description",
    "Manager".to_string(),
    Some("Department heads".to_string())
)]
#[case("can_create_role_without_description", "Support".to_string(), None)]
#[case(
    "can_create_role_and_normalize_name",
    "  Inventory Manager  ".to_string(),
    Some("Stock owners".to_string())
)]
#[tokio::test]
#[serial]
async fn can_create_role(
    #[case] test_name: &str,
    #[case] name: String,
    #[case] description: Option<String>,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();
    let params = new_role(name, description);

    let result = Role::create(ctx.db(), &params).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_id());
            filters
        }
    }, {
        assert_debug_snapshot!(test_name, result)
    })
}

#[rstest]
#[case(
    "cannot_create_role_when_name_already_exists",
    "Manager".to_string(),
    "manager".to_string()
)]
#[case(
    "cannot_create_role_when_name_differs_by_case_and_whitespace",
    "  Manager  ".to_string(),
    "MANAGER".to_string()
)]
#[tokio::test]
#[serial]
async fn cannot_create_duplicate_role(
    #[case] test_name: &str,
    #[case] first_name: String,
    #[case] second_name: String,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    let first = new_role(first_name, Some("Department heads".to_string()));
    Role::create(ctx.db(), &first).await.unwrap();

    let second = new_role(second_name, Some("Duplicate role".to_string()));
    let result = Role::create(ctx.db(), &second).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_seed_roles_from_json", "roles.json")]
#[case(
    "when_seed_file_does_not_exist_role_seeding_fails",
    "missing-roles.json"
)]
#[tokio::test]
#[serial]
async fn can_seed_roles(#[case] test_name: &str, #[case] file: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    let result = Role::seed_data(ctx.db(), file).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "can_update_role",
    "f028f910-1a4f-4b79-8619-71a8c185e221",
    None,
    Some("Both registered shoppers and guest browsers".to_string())
)]
#[case(
    "can_update_role_name_only",
    "f028f910-1a4f-4b79-8619-71a8c185e221",
    Some("Buyer".to_string()),
    None
)]
#[case(
    "can_update_role_with_name_and_description",
    "f028f910-1a4f-4b79-8619-71a8c185e221",
    Some("Wholesale Buyer".to_string()),
    Some("Bulk order customers".to_string())
)]
#[case(
    "can_update_role_and_normalize_name",
    "f028f910-1a4f-4b79-8619-71a8c185e221",
    Some("  VIP Customer  ".to_string()),
    Some("Priority shoppers".to_string())
)]
#[case(
    "can_update_role_with_same_name",
    "7d416019-34c6-4f25-a39a-fa6752f8b319",
    Some("  ADMINISTRATOR  ".to_string()),
    Some("Full system access".to_string())
)]
#[case(
    "cannot_update_role_when_name_already_exists",
    "f028f910-1a4f-4b79-8619-71a8c185e221",
    Some("Administrator".to_string()),
    Some("Duplicate role".to_string())
)]
#[case(
    "cannot_update_role_when_role_does_not_exist",
    "00000000-0000-0000-0000-000000000000",
    Some("Guest".to_string()),
    Some("Guest shoppers".to_string())
)]
#[tokio::test]
#[serial]
async fn can_update_role(
    #[case] test_name: &str,
    #[case] pid: &str,
    #[case] name: Option<String>,
    #[case] description: Option<String>,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    crate::seed_data(ctx.db())
        .await
        .expect("Failed to seed roles");

    let pid = uuid(pid);
    let params = update_role(name, description);

    let result = Role::update(ctx.db(), pid, &params).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_id());
            filters
        }
    }, {
        assert_debug_snapshot!(test_name, result)
    })
}

#[rstest]
#[case(
    "can_find_administrator_role_by_pid",
    "7d416019-34c6-4f25-a39a-fa6752f8b319"
)]
#[case(
    "can_find_customer_role_by_pid",
    "f028f910-1a4f-4b79-8619-71a8c185e221"
)]
#[tokio::test]
#[serial]
async fn can_find_role_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Role::seed_data(ctx.db(), "roles.json")
        .await
        .expect("Failed to seed roles");

    let result = Role::find_by_pid(ctx.db(), uuid(pid)).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "cannot_find_role_when_pid_does_not_exist",
    "00000000-0000-0000-0000-000000000000"
)]
#[tokio::test]
#[serial]
async fn cannot_find_role_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Role::seed_data(ctx.db(), "roles.json")
        .await
        .expect("Failed to seed roles");

    let result = Role::find_by_pid(ctx.db(), uuid(pid)).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_role_list")]
#[tokio::test]
#[serial]
async fn can_find_role_list(#[case] test_name: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Role::seed_data(ctx.db(), "roles.json")
        .await
        .expect("Failed to seed roles");

    let result = Role::find_list(ctx.db()).await;

    assert_debug_snapshot!(test_name, result);
}
