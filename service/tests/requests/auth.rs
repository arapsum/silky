use axum::http::HeaderValue;
use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::{App, models::User};
use uuid::Uuid;
// use service::{models::User, views::LoginResponse};

use crate::utils;

#[derive(Clone, Copy)]
enum VerificationToken {
    Valid,
    Expired,
    Missing,
    Used,
}

#[derive(Clone, Copy)]
enum ResetToken {
    Valid,
    Expired,
    Invalid,
    Missing,
}

#[derive(Clone, Copy)]
enum CurrentUserCredentials {
    AuthorizationHeader,
    AccessCookie,
    Missing,
    RefreshCookieOnly,
    InvalidAuthorizationHeader,
    InvalidAccessToken,
    UnknownUser,
}

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

#[rstest]
#[case("can_successfully_login_user", serde_json::json!({
    "email": "john.doe@acme.com",
    "password": "Password"
}))]
#[case("when_password_is_wrong_login_fails", serde_json::json!({
    "email": "john.doe@acme.com",
    "password": "Password1"
}))]
#[case("when_email_is_wrong_login_fails", serde_json::json!({
    "email": "james.doe@acme.com",
    "password": "Password1"
}))]
#[case("when_email_is_not_email_login_fails", serde_json::json!({
    "email": "james.doe:acme.com",
    "password": "Password1"
}))]
#[case("when_password_is_too_short_login_fails", serde_json::json!({
    "email": "james.doe@acme.com",
    "password": "Pas"
}))]
#[tokio::test]
#[serial]
async fn can_login_user(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server.post("/auth/login").json(&params).await;

        with_settings!({
            filters => {
                let mut filters = utils::cleanup_date().to_vec();
                filters.extend(utils::cleanup_uuid().to_vec());
                filters.extend(utils::cleanup_jwt().to_vec());
                filters.extend(utils::cleanup_headers());
                filters
            }
        },  {
            assert_debug_snapshot!(test_name, (response.status_code(), response.headers(),response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn refresh_token_reuse_is_rejected() {
    crate::request(|server, context| async move {
        crate::seed_data(context.db())
            .await
            .expect("Failed to seed data");

        let params = serde_json::json!({
            "email": "john.doe@acme.com",
            "password": "Password"
        });
        let user = utils::login_users(&server, &params).await;

        let first_response = server
            .post("/auth/refresh")
            .add_cookie(user.refresh_cookie.clone())
            .do_not_save_cookies()
            .await;
        assert_eq!(first_response.status_code(), 200);

        let second_response = server
            .post("/auth/refresh")
            .add_cookie(user.refresh_cookie)
            .do_not_save_cookies()
            .await;
        assert_eq!(second_response.status_code(), 401);
        assert_eq!(second_response.text(), "{\"error\":\"Invalid token\"}");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn logout_revokes_refresh_token_and_clears_cookies() {
    crate::request(|server, context| async move {
        crate::seed_data(context.db())
            .await
            .expect("Failed to seed data");

        let params = serde_json::json!({
            "email": "john.doe@acme.com",
            "password": "Password"
        });
        let user = utils::login_users(&server, &params).await;

        let logout_response = server
            .post("/auth/logout")
            .add_cookie(user.refresh_cookie.clone())
            .do_not_save_cookies()
            .await;

        assert_eq!(logout_response.status_code(), 200);

        let set_cookies = logout_response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|header| header.to_str().expect("set-cookie should be valid"))
            .collect::<Vec<_>>();

        assert_eq!(set_cookies.len(), 2);
        assert!(
            set_cookies
                .iter()
                .any(|cookie| cookie.starts_with("access_token=") && cookie.contains("Max-Age=0"))
        );
        assert!(
            set_cookies
                .iter()
                .any(|cookie| cookie.starts_with("refresh_token=") && cookie.contains("Max-Age=0"))
        );

        let refresh_response = server
            .post("/auth/refresh")
            .add_cookie(user.refresh_cookie)
            .do_not_save_cookies()
            .await;

        assert_eq!(refresh_response.status_code(), 401);
        assert_eq!(refresh_response.text(), "{\"error\":\"Invalid token\"}");
    })
    .await;
}

#[rstest]
#[case("when_email_is_valid_reset_token_is_sent", serde_json::json!({ "email": "john.doe@acme.com" }))]
#[case("when_email_is_invalid_validation_fails_and_no_reset_token_is_sent", serde_json::json!({ "email": "johndoe:acme.com" }))]
#[case("when_email_does_not_exist_no_reset_token_is_sent", serde_json::json!({ "email": "fake@acme.com" }))]
#[tokio::test]
#[serial]
async fn can_forgot_password(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        App::seed(ctx.db()).await.expect("Failed to seed data");

        let response = server.post("/auth/forgot-password").json(&params).await;

        with_settings!({
            filters => {
                let mut filters = utils::cleanup_date().to_vec();
                filters.extend(utils::cleanup_uuid().to_vec());
                filters.extend(utils::cleanup_jwt().to_vec());
                filters.extend(utils::cleanup_headers());
                filters
            }
        },  {
            assert_debug_snapshot!(test_name, (response.status_code(), response.headers(),response.text()))
        })
    })
    .await;
}

#[rstest]
#[case("can_successfully_verify_email", VerificationToken::Valid)]
#[case(
    "when_verification_token_is_expired_email_verification_fails",
    VerificationToken::Expired
)]
#[case(
    "when_verification_token_is_missing_email_verification_fails",
    VerificationToken::Missing
)]
#[case(
    "when_verification_token_is_already_used_email_verification_fails",
    VerificationToken::Used
)]
#[tokio::test]
#[serial]
async fn can_verify_email(#[case] test_name: &str, #[case] token_case: VerificationToken) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        App::seed(ctx.db()).await.expect("Failed to seed data");

        let mut user = User::find_by_email(ctx.db(), "john.doe@acme.com")
            .await
            .unwrap();

        let token = Uuid::new_v4().to_string();

        match token_case {
            VerificationToken::Valid => {
                user.set_verification_token(
                    ctx.db(),
                    &token,
                    ctx.config().auth().verification_token_expiry(),
                )
                .await
                .unwrap();
            }
            VerificationToken::Expired => {
                user.set_verification_token(ctx.db(), &token, -1)
                    .await
                    .unwrap();
            }
            VerificationToken::Missing => {}
            VerificationToken::Used => {
                user.set_verification_token(
                    ctx.db(),
                    &token,
                    ctx.config().auth().verification_token_expiry(),
                )
                .await
                .unwrap();

                User::verify_email(ctx.db(), &token).await.unwrap();
            }
        }

        let response = server.get(&format!("/auth/verify/{token}")).await;

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

#[rstest]
#[case(
    "can_successfully_reset_password",
    ResetToken::Valid,
    serde_json::json!({
        "password": "NewPassword123",
        "confirmPassword": "NewPassword123"
    })
)]
#[case(
    "when_reset_token_is_expired_password_reset_fails",
    ResetToken::Expired,
    serde_json::json!({
        "password": "NewPassword123",
        "confirmPassword": "NewPassword123"
    })
)]
#[case(
    "when_reset_token_is_invalid_password_reset_fails",
    ResetToken::Invalid,
    serde_json::json!({
        "password": "NewPassword123",
        "confirmPassword": "NewPassword123"
    })
)]
#[case(
    "when_reset_token_is_missing_password_reset_fails",
    ResetToken::Missing,
    serde_json::json!({
        "password": "NewPassword123",
        "confirmPassword": "NewPassword123"
    })
)]
#[case(
    "when_new_passwords_do_not_match_password_reset_fails",
    ResetToken::Valid,
    serde_json::json!({
        "password": "NewPassword123",
        "confirmPassword": "DifferentPassword123"
    })
)]
#[case(
    "when_new_password_is_under_eight_chars_password_reset_fails",
    ResetToken::Valid,
    serde_json::json!({
        "password": "short",
        "confirmPassword": "short"
    })
)]
#[case(
    "when_new_password_contains_whitespace_char_password_reset_fails",
    ResetToken::Valid,
    serde_json::json!({
        "password": "New Password123",
        "confirmPassword": "New Password123"
    })
)]
#[case(
    "when_new_password_contains_comma_char_password_reset_fails",
    ResetToken::Valid,
    serde_json::json!({
        "password": "New,Password123",
        "confirmPassword": "New,Password123"
    })
)]
#[tokio::test]
#[serial]
async fn can_reset_password(
    #[case] test_name: &str,
    #[case] token_case: ResetToken,
    #[case] mut params: serde_json::Value,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        App::seed(ctx.db()).await.expect("Failed to seed data");

        let mut user = User::find_by_email(ctx.db(), "john.doe@acme.com")
            .await
            .unwrap();

        let token = Uuid::new_v4().to_string();

        match token_case {
            ResetToken::Valid => {
                user.set_reset_token(ctx.db(), &token, ctx.config().auth().refresh_token_expiry())
                    .await
                    .unwrap();
            }
            ResetToken::Expired => {
                user.set_reset_token(ctx.db(), &token, -1).await.unwrap();
            }
            ResetToken::Invalid | ResetToken::Missing => {}
        }

        let request_token = match token_case {
            ResetToken::Invalid => "invalid-token".to_string(),
            ResetToken::Missing => Uuid::new_v4().to_string(),
            ResetToken::Valid | ResetToken::Expired => token,
        };

        params["token"] = serde_json::json!(request_token);

        let response = server
            .post("/auth/reset-password")
            .json(&params)
            .await;

        with_settings!({
            filters => {
                let mut filters = utils::cleanup_date().to_vec();
                filters.extend(utils::cleanup_uuid().to_vec());
                filters.extend(utils::cleanup_jwt().to_vec());
                filters.extend(utils::cleanup_headers());
                filters
            }
        },  {
            assert_debug_snapshot!(test_name, (response.status_code(), response.headers(), response.text()))
        })
    })
    .await;
}

#[rstest]
#[case(
    "can_get_current_user_with_authorization_header",
    CurrentUserCredentials::AuthorizationHeader
)]
#[case(
    "can_get_current_user_with_access_cookie",
    CurrentUserCredentials::AccessCookie
)]
#[case(
    "when_credentials_are_missing_current_user_fails",
    CurrentUserCredentials::Missing
)]
#[case(
    "when_only_refresh_cookie_is_sent_current_user_fails",
    CurrentUserCredentials::RefreshCookieOnly
)]
#[case(
    "when_authorization_header_is_invalid_current_user_fails",
    CurrentUserCredentials::InvalidAuthorizationHeader
)]
#[case(
    "when_access_token_is_invalid_current_user_fails",
    CurrentUserCredentials::InvalidAccessToken
)]
#[case(
    "when_token_subject_does_not_exist_current_user_fails",
    CurrentUserCredentials::UnknownUser
)]
#[tokio::test]
#[serial]
async fn can_get_current_user(
    #[case] test_name: &str,
    #[case] credentials: CurrentUserCredentials,
) {
    crate::request(|server, context| async move {
        configure_insta!();

        crate::seed_data(context.db())
            .await
            .expect("Failed to seed data");

        let params = serde_json::json!({
            "email": "john.doe@acme.com",
            "password": "Password"
        });
        let user: utils::LoggedInUser = utils::login_users(&server, &params).await;

        let mut request = server.get("/auth/me");

        match credentials {
            CurrentUserCredentials::AuthorizationHeader => {
                let (auth_header, auth_value) = utils::auth_header(user.access_token);
                request = request.add_header(auth_header, auth_value);
            }
            CurrentUserCredentials::AccessCookie => {
                request = request.add_cookie(user.access_cookie);
            }
            CurrentUserCredentials::Missing => {}
            CurrentUserCredentials::RefreshCookieOnly => {
                request = request.add_cookie(user.refresh_cookie);
            }
            CurrentUserCredentials::InvalidAuthorizationHeader => {
                let (auth_header, auth_value) =
                    utils::auth_header(HeaderValue::from_static("Basic invalid"));
                request = request.add_header(auth_header, auth_value);
            }
            CurrentUserCredentials::InvalidAccessToken => {
                let (auth_header, auth_value) =
                    utils::auth_header(HeaderValue::from_static("Bearer invalid"));
                request = request.add_header(auth_header, auth_value);
            }
            CurrentUserCredentials::UnknownUser => {
                let token = context
                    .auth()
                    .access()
                    .generate_token(&Uuid::new_v4().to_string())
                    .unwrap();
                let auth_value = HeaderValue::from_str(&format!("Bearer {token}")).unwrap();
                let (auth_header, auth_value) = utils::auth_header(auth_value);
                request = request.add_header(auth_header, auth_value);
            }
        }

        let response = request.await;

        with_settings!({
            filters => {
                let mut filters = utils::cleanup_date().to_vec();
                filters.extend(utils::cleanup_uuid().to_vec());
                filters.extend(utils::cleanup_jwt().to_vec());
                filters.extend(utils::cleanup_headers());
                filters
            }
        },  {
            assert_debug_snapshot!(test_name, (response.status_code(), response.headers(), response.text()))
        })
    })
    .await;
}
