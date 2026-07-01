use std::borrow::Cow;

use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use service::schemas::{ChangePassword, Validator};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("auth");
        settings.set_snapshot_path("snapshots/auth");
        let _guard = settings.bind_to_scope();
    };
}

fn change_password(
    current_password: String,
    password: String,
    confirm_password: String,
) -> ChangePassword<'static> {
    ChangePassword::new(
        Cow::Owned(current_password),
        Cow::Owned(password),
        Cow::Owned(confirm_password),
    )
}

#[rstest]
#[case(
    "change_password_validation_accepts_valid_params",
    "CurrentPassword123".to_string(),
    "NewPassword123".to_string(),
    "NewPassword123".to_string()
)]
#[case(
    "change_password_validation_rejects_empty_current_password",
    String::new(),
    "NewPassword123".to_string(),
    "NewPassword123".to_string()
)]
#[case(
    "change_password_validation_rejects_mismatched_passwords",
    "CurrentPassword123".to_string(),
    "NewPassword123".to_string(),
    "DifferentPassword123".to_string()
)]
#[case(
    "change_password_validation_rejects_short_new_password",
    "CurrentPassword123".to_string(),
    "short".to_string(),
    "short".to_string()
)]
#[case(
    "change_password_validation_rejects_new_password_with_whitespace",
    "CurrentPassword123".to_string(),
    "New Password123".to_string(),
    "New Password123".to_string()
)]
#[case(
    "change_password_validation_rejects_new_password_with_comma",
    "CurrentPassword123".to_string(),
    "New,Password123".to_string(),
    "New,Password123".to_string()
)]
fn can_validate_change_password(
    #[case] test_name: &str,
    #[case] current_password: String,
    #[case] password: String,
    #[case] confirm_password: String,
) {
    configure_insta!();

    let params = change_password(current_password, password, confirm_password);
    let result = Validator::new(params)
        .validate()
        .map(|_| "valid".to_string())
        .map_err(|err| err.to_string());

    assert_debug_snapshot!(test_name, result);
}
