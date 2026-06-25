use std::borrow::Cow;

use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::{
    models::Role,
    schemas::{NewRole, UpdateRole, Validator},
};

use crate::{
    boot_test,
    utils::{cleanup_date, cleanup_id, cleanup_password, cleanup_uuid},
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
            filters.extend(cleanup_password());
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
#[case("new_role_validation_accepts_valid_params", "Manager".to_string(), Some("Department heads".to_string()))]
#[case("new_role_validation_accepts_missing_description", "Manager".to_string(), None)]
#[case("new_role_validation_rejects_empty_name", "".to_string(), Some("Department heads".to_string()))]
#[case("new_role_validation_rejects_short_name", "A".to_string(), Some("Department heads".to_string()))]
#[case("new_role_validation_rejects_long_name", "a".repeat(33), Some("Department heads".to_string()))]
#[case("new_role_validation_rejects_name_with_special_chars", "Manager+".to_string(), Some("Department heads".to_string()))]
#[case("new_role_validation_rejects_long_description", "Manager".to_string(), Some("a".repeat(257)))]
fn can_validate_new_role(
    #[case] test_name: &str,
    #[case] name: String,
    #[case] description: Option<String>,
) {
    configure_insta!();

    let params = new_role(name, description);
    let result = Validator::new(params)
        .validate()
        .map(|_| "valid".to_string())
        .map_err(|err| err.to_string());

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("update_role_validation_accepts_valid_name", Some("Manager".to_string()), None)]
#[case("update_role_validation_accepts_description_only", None, Some("Department heads".to_string()))]
#[case("update_role_validation_rejects_empty_name", Some("".to_string()), Some("Department heads".to_string()))]
#[case("update_role_validation_rejects_short_name", Some("A".to_string()), Some("Department heads".to_string()))]
#[case("update_role_validation_rejects_long_name", Some("a".repeat(33)), Some("Department heads".to_string()))]
#[case("update_role_validation_rejects_name_with_special_chars", Some("Manager+".to_string()), Some("Department heads".to_string()))]
#[case("update_role_validation_rejects_long_description", Some("Manager".to_string()), Some("a".repeat(257)))]
fn can_validate_update_role(
    #[case] test_name: &str,
    #[case] name: Option<String>,
    #[case] description: Option<String>,
) {
    configure_insta!();

    let params = update_role(name, description);
    let result = Validator::new(params)
        .validate()
        .map(|_| "valid".to_string())
        .map_err(|err| err.to_string());

    assert_debug_snapshot!(test_name, result);
}
