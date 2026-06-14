#![allow(clippy::unused_self)]

use std::{
    env::VarError,
    error::Error as _,
    fmt::{self, Display},
    io::IsTerminal,
    str::FromStr,
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    EnvFilter, Layer,
    filter::Directive,
    fmt::{
        Layer as FmtLayer,
        format::{DefaultFields, Format as FmtFormat},
        writer::BoxMakeWriter,
    },
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
};

use crate::{Error, Result};

type TracingFmtLayer<S> = FmtLayer<S, DefaultFields, FmtFormat, BoxMakeWriter>;

static NONBLOCKING_WORK_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum Level {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "trace")]
    Trace,
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    #[default]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

impl Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Off => "off",
                Self::Trace => "trace",
                Self::Debug => "debug",
                Self::Info => "info",
                Self::Warn => "warn",
                Self::Error => "error",
            }
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum Format {
    #[serde(rename = "compact")]
    Compact,
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "pretty")]
    #[default]
    Pretty,
}

impl Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Compact => "compact",
                Self::Full => "full",
                Self::Json => "json",
                Self::Pretty => "pretty",
            }
        )
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Logger {
    level: Level,
    format: Format,
    crates: Vec<String>,
    file_appender: Option<LoggerFileAppender>,
}

impl Logger {
    pub fn setup(&self) -> Result<()> {
        let env_filter_layer = self.env_filter()?;
        let registry = tracing_subscriber::registry()
            .with(env_filter_layer)
            .with(ErrorLayer::default());

        let result = match self.format {
            Format::Compact => registry.with(self.compact_fmt_layer()).try_init(),
            Format::Full => registry.with(self.base_fmt_layer()).try_init(),
            Format::Json => registry.with(self.json_fmt_layer()).try_init(),
            Format::Pretty => registry.with(self.pretty_fmt_layer()).try_init(),
        };

        if let Err(e) = result {
            let msg = e.to_string();

            if !msg.contains("a global default trace dispatcher has already been set") {
                return Err(e.into());
            }
        }

        Ok(())
    }

    fn env_filter(&self) -> Result<EnvFilter> {
        let mut env_filter = match EnvFilter::try_from_default_env() {
            Ok(env_filter) => env_filter,
            Err(from_env_err) => {
                if let Some(err) = from_env_err.source() {
                    match err.downcast_ref::<VarError>() {
                        Some(VarError::NotPresent) => (),
                        Some(other) => return Err(Error::EnvFilter(other.clone())), // Converts into crate::Report
                        _ => return Err(Error::FromEnv(from_env_err)),
                    }
                }

                if self.crates.is_empty() {
                    EnvFilter::try_new(format!("{}={}", env!("CARGO_PKG_NAME"), &self.level))?
                } else {
                    EnvFilter::try_new("")?
                }
            }
        };

        let directives = self.directives()?;

        for directive in directives {
            env_filter = env_filter.add_directive(directive);
        }

        Ok(env_filter)
    }

    fn writer(&self) -> BoxMakeWriter {
        self.file_appender().map_or_else(
            || BoxMakeWriter::new(std::io::stderr),
            |file_appender| {
                file_appender
                    .writer()
                    .unwrap_or_else(|_| BoxMakeWriter::new(std::io::stderr))
            },
        )
    }

    fn base_fmt_layer<S>(&self) -> TracingFmtLayer<S>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        FmtLayer::new()
            .with_ansi(std::io::stderr().is_terminal())
            .with_writer(self.writer())
    }

    fn pretty_fmt_layer<S>(&self) -> impl Layer<S>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        self.base_fmt_layer().pretty()
    }

    fn json_fmt_layer<S>(&self) -> impl Layer<S>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        self.base_fmt_layer().json()
    }

    fn compact_fmt_layer<S>(&self) -> impl Layer<S>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        self.base_fmt_layer()
            .compact()
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
    }

    #[must_use]
    pub const fn level(&self) -> &Level {
        &self.level
    }

    #[must_use]
    pub const fn format(&self) -> &Format {
        &self.format
    }

    pub fn directives(&self) -> Result<Vec<Directive>> {
        self.crates
            .iter()
            .map(|c| -> Result<Directive> {
                let str_directive = format!("{}={}", c, &self.level);
                Ok(Directive::from_str(&str_directive)?)
            })
            .collect()
    }

    #[must_use]
    pub const fn file_appender(&self) -> Option<&LoggerFileAppender> {
        self.file_appender.as_ref()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum Rotation {
    #[serde(rename = "minutely")]
    Minutely,
    #[serde(rename = "hourly")]
    #[default]
    Hourly,
    #[serde(rename = "daily")]
    Daily,
    #[serde(rename = "weekly")]
    Weekly,
}

impl Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Minutely => write!(f, "minutely"),
            Self::Hourly => write!(f, "hourly"),
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LoggerFileAppender {
    pub enable: bool,
    pub non_blocking: bool,
    pub rotation: Rotation,
    /// Directory where log files will be written. Defaults to `./logs`.
    pub directory: Option<String>,
    /// Filename prefix for log files.
    pub filename_prefix: Option<String>,
    /// Filename suffix for log files.
    pub filename_suffix: Option<String>,
    pub max_log_files: usize,
}

impl LoggerFileAppender {
    fn writer(&self) -> Result<BoxMakeWriter> {
        if self.enable {
            let dir = self
                .directory
                .as_ref()
                .map_or_else(|| "./logs".into(), ToString::to_string);

            let mut rolling_builder =
                tracing_appender::rolling::Builder::default().max_log_files(self.max_log_files);

            rolling_builder = match self.rotation {
                Rotation::Minutely => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::MINUTELY)
                }
                Rotation::Hourly => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::HOURLY)
                }
                Rotation::Daily => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::DAILY)
                }
                Rotation::Weekly => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::WEEKLY)
                }
            };

            let rolling_file_appender = rolling_builder
                .filename_prefix(
                    self.filename_prefix
                        .as_ref()
                        .map_or_else(String::new, ToString::to_string),
                )
                .filename_suffix(
                    self.filename_suffix
                        .as_ref()
                        .map_or_else(String::new, ToString::to_string),
                )
                .build(dir)?;

            if self.non_blocking {
                let (non_blocking, work_guard) =
                    tracing_appender::non_blocking(rolling_file_appender);

                NONBLOCKING_WORK_GUARD
                    .set(work_guard)
                    .map_err(|_e| Error::NonBlockingWorkGuardAlreadySet)?;

                Ok(BoxMakeWriter::new(non_blocking))
            } else {
                Ok(BoxMakeWriter::new(rolling_file_appender))
            }
        } else {
            Ok(BoxMakeWriter::new(std::io::stderr))
        }
    }
}
