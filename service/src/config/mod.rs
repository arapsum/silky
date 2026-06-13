use std::fmt::{self, Display};

use serde::Deserialize;

use crate::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    server: ServerConfig,
}

impl Config {
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

    pub fn server(&self) -> &ServerConfig {
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

#[derive(Debug, Clone)]
pub enum Environment {
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
