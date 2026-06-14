use sqlx::PgPool;

use crate::{Config, Error};

#[derive(Clone)]
pub struct AppContext {
    config: Config,
    db: PgPool,
}

impl AppContext {
    /// Initialises application services and infrastructure.
    ///
    /// This method performs application startup tasks, including configuring
    /// logging and executing any database initialisation required by the current
    /// configuration.
    ///
    /// Startup operations are performed in the following order:
    ///
    /// 1. Configure and initialise the application's logger.
    /// 2. Initialise the database and run any configured migration actions.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The logger cannot be initialised.
    /// - Database initialisation fails.
    /// - Any configured migration operation fails.
    pub async fn init(&self) -> crate::Result<()> {
        self.config.logger().setup()?;
        self.config.database().init().await?;

        Ok(())
    }

    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub const fn db(&self) -> &PgPool {
        &self.db
    }
}

impl TryFrom<&Config> for AppContext {
    type Error = Error;

    fn try_from(cfg: &Config) -> Result<Self, Self::Error> {
        Ok(Self {
            db: cfg.database().pool()?,
            config: cfg.clone(),
        })
    }
}
