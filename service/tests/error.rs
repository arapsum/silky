use axum::{body::to_bytes, http::StatusCode};
use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use serde_json::Value;
use service::Error;

macro_rules! configure_insta {
    () => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/error");
        settings.set_snapshot_suffix("error");
        let _guard = settings.bind_to_scope();
    };
}

#[rstest]
#[case::forbidden("forbidden", Error::Forbidden, StatusCode::FORBIDDEN)]
#[case::invalid_token("invalid_token", Error::InvalidToken, StatusCode::UNAUTHORIZED)]
#[case::expired_session("expired_session", Error::ExpiredSession, StatusCode::UNAUTHORIZED)]
#[case::missing_credentials(
    "missing_credentials",
    Error::MissingCredentials,
    StatusCode::UNAUTHORIZED
)]
#[tokio::test]
async fn application_errors_have_stable_response_bodies(
    #[case] test_name: &str,
    #[case] error: Error,
    #[case] expected_status: StatusCode,
) {
    configure_insta!();

    let response = error.response();
    assert_eq!(response.status(), expected_status);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("error response body should be readable");
    let body: Value = serde_json::from_slice(&body).expect("error response should contain JSON");

    assert_debug_snapshot!(test_name, body);
}

#[tokio::test]
async fn validation_errors_expose_field_details() {
    configure_insta!();

    let response = Error::ValidationError(
        serde_json::json!({
            "email": "Invalid email address",
            "items[0].quantity": "Quantity must be at least one",
        })
        .to_string(),
    )
    .response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("validation response body should be readable");
    let body: Value =
        serde_json::from_slice(&body).expect("validation response should contain JSON");

    assert_debug_snapshot!("validation_error_details", body);
}
