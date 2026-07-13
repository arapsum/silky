use std::fmt::{self, Display};

use jsonwebtoken::errors::{Error as JwtError, ErrorKind as JwtErrorKind};
use serde::Serialize;
use serde_json::Value;

use crate::models::ModelError;

mod response;

/// JSON body returned when an API request cannot be completed.
///
/// `error` is intended for people, while `code` is a stable identifier that
/// clients can use without matching display text. `field` and `details` are
/// included only when the error has actionable, structured context.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: String,
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ErrorResponse {
    #[must_use]
    pub fn new(error: impl Into<String>, code: &'static str) -> Self {
        Self {
            error: error.into(),
            code,
            field: None,
            details: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    AppenderInit(#[from] tracing_appender::rolling::InitError),
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    DirectiveParseError(#[from] tracing_subscriber::filter::ParseError),
    #[error(transparent)]
    EnvFilter(#[from] std::env::VarError),
    #[error("Expired Session")]
    ExpiredSession,
    #[error(transparent)]
    ExtensionRejection(#[from] axum::extract::rejection::ExtensionRejection),
    #[error(transparent)]
    FromEnv(#[from] tracing_subscriber::filter::FromEnvError),
    #[error("Forbidden")]
    Forbidden,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    JsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[error(transparent)]
    Jwt(JwtError),
    #[error(transparent)]
    Mailer(#[from] MailerError),
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("Unauthorised: Missing credentials")]
    MissingCredentials,
    #[error(transparent)]
    Model(#[from] ModelError),
    #[error("Non-blocking work guard already set")]
    NonBlockingWorkGuardAlreadySet,
    #[error(transparent)]
    PathRejection(#[from] axum::extract::rejection::PathRejection),
    #[error(transparent)]
    QueryRejection(#[from] axum::extract::rejection::QueryRejection),
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    TryInit(#[from] tracing_subscriber::util::TryInitError),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug)]
pub struct Report(pub color_eyre::Report);

impl<E> From<E> for Report
where
    E: Into<color_eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub type Result<T, E = Report> = std::result::Result<T, E>;

impl From<JwtError> for Error {
    fn from(err: JwtError) -> Self {
        match err.kind() {
            JwtErrorKind::ExpiredSignature => Self::ExpiredSession,
            _ => Self::InvalidToken,
        }
    }
}

impl Error {
    /// Returns a stable machine-readable code for API consumers and logs.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ExpiredSession => "session_expired",
            Self::Forbidden => "forbidden",
            Self::InvalidCredentials => "invalid_credentials",
            Self::InvalidToken | Self::Jwt(_) => "invalid_token",
            Self::MissingCredentials => "missing_credentials",
            Self::Model(error) => error.code(),
            Self::ValidationError(_) => "validation_error",
            Self::JsonRejection(_) => "invalid_json",
            Self::PathRejection(_) => "invalid_path_parameter",
            Self::QueryRejection(_) => "invalid_query_parameter",
            Self::ExtensionRejection(_)
            | Self::AppenderInit(_)
            | Self::Config(_)
            | Self::DirectiveParseError(_)
            | Self::EnvFilter(_)
            | Self::FromEnv(_)
            | Self::IO(_)
            | Self::Mailer(_)
            | Self::Migrate(_)
            | Self::NonBlockingWorkGuardAlreadySet
            | Self::Redis(_)
            | Self::Sqlx(_)
            | Self::TryInit(_) => "internal_error",
        }
    }

    /// Extracts structured validation fields when the validation error contains
    /// the JSON object produced by [`crate::schemas::validator::Validator`].
    #[must_use]
    pub fn details(&self) -> Option<Value> {
        match self {
            Self::ValidationError(error) => serde_json::from_str(error)
                .ok()
                .filter(|details: &Value| details.is_object()),
            Self::Model(error) => error.details(),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MailerError {
    #[error(transparent)]
    Address(#[from] lettre::address::AddressError),
    #[error("Input/output error")]
    IO,
    #[error("Mailer initialisation error: {0}")]
    Init(String),
    #[error(transparent)]
    Lettre(#[from] lettre::error::Error),
    #[error("Missing render variable")]
    MissingVariable,
    #[error(transparent)]
    Render(#[from] handlebars::RenderError),
    #[error(transparent)]
    Smtp(#[from] lettre::transport::smtp::Error),
    #[error(transparent)]
    Stub(#[from] lettre::transport::stub::Error),
    #[error(transparent)]
    Template(#[from] handlebars::TemplateError),
}

impl From<&'static Self> for MailerError {
    fn from(value: &'static Self) -> Self {
        Self::Init(value.to_string())
    }
}

pub type MailerResult<T, E = MailerError> = std::result::Result<T, E>;
