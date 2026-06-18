use std::{io::IsTerminal, net::SocketAddr, sync::Arc};

use axum::Router;
use clap::Parser;
use color_eyre::config::{HookBuilder, Theme};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{
    AppContext, Commands, Config, Result, config::Environment, controllers, middlewares::trace,
    models::User,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct App {
    #[arg(short, long, default_value_t = Environment::default())]
    env: Environment,

    #[command(subcommand)]
    command: Option<Commands>,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::parse()
    }

    pub async fn init(&self, config: &Config) -> Result<Arc<AppContext>> {
        let ctx = AppContext::try_from(config)?;
        ctx.init().await?;

        match self.command {
            Some(Commands::Seed) => {
                Self::seed(ctx.db()).await?;
            }
            None => {}
        }

        Ok(Arc::new(ctx))
    }

    /// Starts the application and begins serving HTTP requests.
    ///
    /// Configuration is loaded from the environment specified by this
    /// application instance. Application services are then initialised,
    /// including logging and database setup, before the HTTP server is bound
    /// to the configured address.
    ///
    /// All application routes are served under the `/api` path.
    ///
    /// This method does not return until the server shuts down or an error
    /// occurs.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The application configuration cannot be loaded.
    /// - The application context cannot be created.
    /// - Application initialization fails.
    /// - The server cannot bind to the configured address.
    /// - The HTTP server encounters an error while serving requests.
    pub async fn run(&self) -> Result<()> {
        HookBuilder::new().theme(if std::io::stderr().is_terminal() {
            Theme::dark()
        } else {
            Theme::new()
        });

        let config = Config::from_env(&self.env)?;

        let ctx = self.init(&config).await?;

        let server = ctx.config().server();

        let listener = TcpListener::bind(server.address()).await?;

        let router = Router::new().nest("/api", controllers::router()).layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::make_span_with)
                .on_response(trace::on_response)
                .on_request(trace::on_request)
                .on_failure(trace::on_failure),
        );

        tracing::info!("Listening on {}", server.url());

        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(Into::into)
    }

    pub async fn seed(db: &sqlx::PgPool) -> Result<()> {
        User::seed_data(db, "users.json").await?;

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
