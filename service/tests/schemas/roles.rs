use std::borrow::Cow;

use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use serde_json::json;
use service::schemas::{AssignRole, NewRole, UpdateRole, Validator};

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

fn assign_role(user_id: i32, role_id: i32) -> AssignRole {
    serde_json::from_value(json!({
        "userId": user_id,
        "roleId": role_id
    }))
    .expect("Failed to parse role assignment")
}

#[rstest]
#[case(
    "new_role_validation_accepts_valid_params",
    "Manager".to_string(),
    Some("Department heads".to_string())
)]
#[case(
    "new_role_validation_accepts_missing_description",
    "Manager".to_string(),
    None
)]
#[case(
    "new_role_validation_rejects_empty_name",
    "".to_string(),
    Some("Department heads".to_string())
)]
#[case(
    "new_role_validation_rejects_short_name",
    "A".to_string(),
    Some("Department heads".to_string())
)]
#[case(
    "new_role_validation_rejects_long_name",
    "a".repeat(33),
    Some("Department heads".to_string())
)]
#[case(
    "new_role_validation_rejects_name_with_special_chars",
    "Manager+".to_string(),
    Some("Department heads".to_string())
)]
#[case(
    "new_role_validation_rejects_long_description",
    "Manager".to_string(),
    Some("a".repeat(257))
)]
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
#[case(
    "update_role_validation_accepts_valid_name",
    Some("Manager".to_string()),
    None
)]
#[case(
    "update_role_validation_accepts_description_only",
    None,
    Some("Department heads".to_string())
)]
#[case(
    "update_role_validation_rejects_empty_name",
    Some("".to_string()),
    Some("Department heads".to_string())
)]
#[case(
    "update_role_validation_rejects_short_name",
    Some("A".to_string()),
    Some("Department heads".to_string())
)]
#[case(
    "update_role_validation_rejects_long_name",
    Some("a".repeat(33)),
    Some("Department heads".to_string())
)]
#[case(
    "update_role_validation_rejects_name_with_special_chars",
    Some("Manager+".to_string()),
    Some("Department heads".to_string())
)]
#[case(
    "update_role_validation_rejects_long_description",
    Some("Manager".to_string()),
    Some("a".repeat(257))
)]
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

#[rstest]
#[case("assign_role_validation_accepts_valid_params", 11, 22)]
#[case("assign_role_validation_rejects_zero_user_id", 0, 22)]
#[case("assign_role_validation_rejects_zero_role_id", 11, 0)]
#[case("assign_role_validation_rejects_negative_params", -1, -2)]
fn can_validate_assign_role(#[case] test_name: &str, #[case] user_id: i32, #[case] role_id: i32) {
    configure_insta!();

    let params = assign_role(user_id, role_id);
    let result = Validator::new(params)
        .validate()
        .map(|_| "valid".to_string())
        .map_err(|err| err.to_string());

    assert_debug_snapshot!(test_name, result);
}
