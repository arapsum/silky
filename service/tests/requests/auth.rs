use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
// use service::{models::User, views::LoginResponse};

use crate::utils;

macro_rules! configure_insta {
    ($(expr:expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/auth");
        settings.set_snapshot_suffix("auth");
        let _guard = settings.bind_to_scope();
    };
}

#[rstest]
#[case(
    "can_successfully_register_user",
    serde_json::json!({
        "email":  "test1@example.com",
        "name": "Test User1",
        "password": "SafePassWord11!",
        "confirmPassword": "SafePassWord11!"
    })
)]
#[case(
    "when_passwords_do_not_match_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "name": "Test User1",
        "password": "SafePassWord11!",
        "confirmPassword": "SafePassWord11"
    })
)]
#[case(
    "when_password_is_under_eight_chars_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "name": "Test User1",
        "password": "Safe",
        "confirmPassword": "Safe"
    })
)]
#[case(
    "when_password_contains_whitespace_char_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "name": "Test User1",
        "password": "ChangeMe 123!",
        "confirmPassword": "ChangeMe 123!"
    })
)]
#[case(
    "when_password_contains_comma_char_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "name": "Test User1",
        "password": "ChangeMe,123!",
        "confirmPassword": "ChangeMe,123!"
    })
)]
#[case(
    "when_email_is_not_an_email_registration_fails",
    serde_json::json!({
        "email":  "test1:example.com",
        "name": "Test User1",
        "password": "Password123",
        "confirmPassword": "Password123"
    })
)]
#[case(
    "when_name_is_under_six_chars_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "name": "Test",
        "password": "Password123",
        "confirmPassword": "Password123"
    })
)]
#[case(
    "when_name_contains_non_alphanumeric_chars_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "name": "test user;+",
        "password": "Password123",
        "confirmPassword": "Password123"
    })
)]
#[tokio::test]
#[serial]
async fn can_register_user(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server.post("/auth/register").json(&params).await;

        with_settings!({
            filters => {
                let mut filters = utils::cleanup_date().to_vec();
                filters.extend(utils::cleanup_uuid().to_vec());
                filters
            }
        },  {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        })
    })
    .await;
}
