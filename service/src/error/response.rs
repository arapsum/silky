use crate::models::ModelError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::{Error, ErrorResponse, Report};

impl IntoResponse for Report {
    fn into_response(self) -> Response {
        let report = self.0;

        if let Some(error) = report.downcast_ref::<Error>() {
            let status = error.response_body().0;
            log_report(&report, status, error.code());
            return error.response();
        } else if let Some(error) = report.downcast_ref::<ModelError>() {
            let status = error.response_body().0;
            log_report(&report, status, error.code());
            return error.response();
        }

        log_report(&report, StatusCode::INTERNAL_SERVER_ERROR, "internal_error");
        let body = Json(ErrorResponse::new(
            "An internal server error has occurred",
            "internal_error",
        ));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

fn log_report(report: &color_eyre::Report, status: StatusCode, code: &'static str) {
    if status.is_server_error() {
        tracing::error!(
            error = %report,
            error_debug = ?report,
            %status,
            code,
            "Request failed"
        );
    } else {
        tracing::warn!(error = %report, %status, code, "Request rejected");
    }
}

impl Error {
    #[must_use]
    pub fn response_body(&self) -> (StatusCode, String) {
        let (status, message) = match self {
            Self::InvalidToken | Self::Jwt(_) => (
                StatusCode::UNAUTHORIZED,
                "Your authentication token is invalid. Please sign in again.".to_string(),
            ),
            Self::ExpiredSession => (
                StatusCode::UNAUTHORIZED,
                "Your session has expired. Please sign in again.".to_string(),
            ),
            Self::MissingCredentials => (
                StatusCode::UNAUTHORIZED,
                "Authentication is required. Please sign in and try again.".to_string(),
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "You do not have permission to perform this action.".to_string(),
            ),
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "The email address or password is incorrect.".to_string(),
            ),
            Self::Model(model_error) => model_error.response_body(),
            Self::ValidationError(val_error) => (StatusCode::BAD_REQUEST, val_error.clone()),
            Self::JsonRejection(json_rejection) => {
                (json_rejection.status(), json_rejection.body_text())
            }
            Self::PathRejection(path_rejection) => (
                path_rejection.status(),
                "The supplied resource identifier is invalid.".to_string(),
            ),
            Self::QueryRejection(query_rejection) => {
                (query_rejection.status(), query_rejection.body_text())
            }
            Self::ExtensionRejection(extension_rejection) => (
                extension_rejection.status(),
                extension_rejection.body_text(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, message)
    }

    #[must_use]
    pub fn response(&self) -> Response {
        if let Self::Model(model_error) = self {
            return model_error.response();
        }

        let (status, mut message) = self.response_body();
        let details = self.details();
        if details.is_some() && matches!(self, Self::ValidationError(_)) {
            message = "One or more fields failed validation".to_string();
        }

        let mut body = ErrorResponse::new(message, self.code());
        body.field = self.field();
        body.details = details;

        (status, Json(body)).into_response()
    }
}
