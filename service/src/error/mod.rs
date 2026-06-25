use std::fmt::{self, Display};

use jsonwebtoken::errors::{Error as JwtError, ErrorKind as JwtErrorKind};

use crate::models::ModelError;

mod response;

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
