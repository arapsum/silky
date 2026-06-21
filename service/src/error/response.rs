use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::models::ModelError;

use super::{Error, Report};

impl IntoResponse for Report {
    fn into_response(self) -> Response {
        let err = self.0;
        let err_string = format!("{err}");

        tracing::error!("[error]: {}", &err_string);

        if let Some(error) = err.downcast_ref::<Error>() {
            return error.response();
        } else if let Some(error) = err.downcast_ref::<ModelError>() {
            return error.response();
        }

        // fallback error
        let body = Json(json!({"error": "An internal server error has occurred!"}));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl Error {
    #[must_use]
    pub fn response_body(&self) -> (StatusCode, String) {
        let (status, message) = match self {
            Self::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            Self::ExpiredSession => (StatusCode::UNAUTHORIZED, "Expired session".to_string()),
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid email or password".to_string(),
            ),
            Self::Model(model_error) => model_error.response_body(),
            Self::ValidationError(val_error) => (StatusCode::BAD_REQUEST, val_error.clone()),
            Self::JsonRejection(json_rejection) => {
                (json_rejection.status(), json_rejection.body_text())
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, message)
    }

    #[must_use]
    pub fn response(&self) -> Response {
        let (status, message) = self.response_body();
        (status, Json(json!({ "error": message }))).into_response()
    }
}
