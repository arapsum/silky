mod log;

use std::{
    fmt::{self, Display},
    path::Path,
    time::Duration,
};

use serde::Deserialize;
use sqlx::{Pool, Postgres, migrate::Migrator, postgres::PgPoolOptions};

use crate::Result;

pub use self::log::Logger;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    server: ServerConfig,
    database: DatabaseConfig,
    logger: Logger,
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

    #[must_use]
    pub const fn database(&self) -> &DatabaseConfig {
        &self.database
    }

    #[must_use]
    pub const fn logger(&self) -> &Logger {
        &self.logger
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    protocol: String,
    host: String,
    port: u16,
}

impl ServerConfig {
    #[must_use]
    pub fn address(&self) -> String {
        format!("{}:{}", &self.host, self.port)
    }

    #[must_use]
    pub fn url(&self) -> String {
        format!("{}://{}:{}", &self.protocol, &self.host, &self.port)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub(crate) uri: String,
    pub(crate) max_connections: u32,
    pub(crate) min_connections: u32,
    pub(crate) connection_timeout: u64,
    pub(crate) idle_timeout: u64,
    pub(crate) auto_migrate: bool,
    pub(crate) dangerously_truncate: bool,
    pub(crate) dangerously_recreate: bool,
}

impl DatabaseConfig {
    /// Creates a `PostgreSQL` connection pool from this configuration.
    ///
    /// The returned pool is lazily connected, meaning no database connection is
    /// established until the pool is first used.
    ///
    /// Pool settings such as connection limits, idle timeout, and connection
    /// acquisition timeout are configured from this [`DatabaseConfig`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the configured database URI is
    /// invalid and a connection pool cannot be created.
    pub fn pool(&self) -> Result<Pool<Postgres>> {
        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .idle_timeout(Duration::from_secs(self.idle_timeout))
            .acquire_timeout(Duration::from_secs(self.connection_timeout))
            .connect_lazy(&self.uri)
            .map_err(Into::into)
    }

    #[allow(clippy::cast_possible_wrap)]
    /// Initializes the database according to this configuration.
    ///
    /// This function loads migrations from the `migrations` directory and applies
    /// migration-related configuration options such as automatic migration and
    /// database recreation.
    ///
    /// The following behavior is performed:
    ///
    /// - If no migrations exist, the function returns immediately.
    /// - If both `dangerously_recreate` and `dangerously_truncate` are enabled,
    ///   all migrations are rolled back and then reapplied.
    /// - If only `dangerously_recreate` is enabled, all migrations are rolled
    ///   back before any further actions are taken.
    /// - If `auto_migrate` is enabled, all pending migrations are applied.
    ///
    /// # Warning
    ///
    /// Enabling `dangerously_recreate` may result in complete data loss, as all
    /// migrations are rolled back before being reapplied.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The database connection pool cannot be created.
    /// - The migration directory cannot be loaded.
    /// - A migration rollback operation fails.
    /// - A migration execution operation fails.
    pub async fn init(&self) -> Result<()> {
        let pool = self.pool()?;
        let migrator = Migrator::new(Path::new("migrations")).await?;

        let migrations = migrator.iter().count() as i64;

        if migrations == 0 {
            return Ok(());
        }

        if self.dangerously_recreate && self.dangerously_truncate {
            migrator.undo(&pool, migrations).await?;
            migrator.run(&pool).await?;
            return Ok(());
        }

        // TODO: delete all the data in the tables without dropping the tables.
        // if self.dangerously_truncate {
        // }

        if self.dangerously_recreate {
            migrator.undo(&pool, migrations).await?;
        }

        if self.auto_migrate {
            migrator.run(&pool).await?;
        }

        Ok(())
    }

    #[must_use]
    pub fn uri(&self) -> &str {
        &self.uri
    }

    #[must_use]
    pub const fn max_connections(&self) -> u32 {
        self.max_connections
    }

    #[must_use]
    pub const fn min_connections(&self) -> u32 {
        self.min_connections
    }

    #[must_use]
    pub const fn connection_timeout(&self) -> u64 {
        self.connection_timeout
    }

    #[must_use]
    pub const fn idle_timeout(&self) -> u64 {
        self.idle_timeout
    }

    #[must_use]
    pub const fn auto_migrate(&self) -> bool {
        self.auto_migrate
    }

    #[must_use]
    pub const fn dangerously_truncate(&self) -> bool {
        self.dangerously_truncate
    }

    #[must_use]
    pub const fn dangerously_recreate(&self) -> bool {
        self.dangerously_recreate
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
    #[must_use]
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
