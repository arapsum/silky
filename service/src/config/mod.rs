use std::fmt::{self, Display};

use serde::Deserialize;

use crate::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    server: ServerConfig,
}

impl Config {
    /// Loads application configuration from a YAML file and environment variables.
    ///
    /// The configuration file is resolved from the `config` directory in the
    /// current working directory using the provided environment name
    /// (for example, `config/development.yaml` or `config/production.yaml`).
    ///
    /// Environment variables prefixed with `APP_` are also loaded and will
    /// override values from the configuration file where applicable.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The current working directory cannot be determined.
    /// - The configuration file does not exist or cannot be read.
    /// - The configuration file contains invalid configuration data.
    /// - The configuration cannot be deserialized into `Self`.
    pub fn from_env(environment: &Environment) -> Result<Self> {
        let base_path = std::env::current_dir()?;
        let config_dir = base_path.join("config");

        let filename = config_dir.join(format!("{}.yaml", environment.as_str()));

        let config = config::Config::builder()
            .add_source(config::File::from(filename))
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("_")
                    .prefix_separator("_"),
            )
            .build()?;

        config.try_deserialize::<Self>().map_err(Into::into)
    }

    #[must_use]
    pub const fn server(&self) -> &ServerConfig {
        &self.server
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    protocol: String,
    host: String,
    port: u16,
}

impl ServerConfig {
    pub fn address(&self) -> String {
        format!("{}:{}", &self.host, self.port)
    }

    pub fn url(&self) -> String {
        format!("{}://{}:{}", &self.protocol, &self.host, &self.port)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Environment {
    #[default]
    Development,
    Production,
    Testing,
    Other(String),
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
            Self::Testing => "testing",
            Self::Other(env) => env,
        }
    }
}

impl From<&str> for Environment {
    fn from(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "development" | "dev" => Self::Development,
            "production" | "prod" => Self::Production,
            "testing" | "test" => Self::Testing,
            _ => Self::Other(s.to_string()),
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
