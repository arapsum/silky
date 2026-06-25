use std::{io::IsTerminal, net::SocketAddr, sync::Arc};

use apalis::prelude::{Monitor, WorkerBuilder, WorkerFactoryFn};
use axum::Router;
use clap::Parser;
use color_eyre::config::{HookBuilder, Theme};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{
    AppContext, Commands, Config, Result,
    config::Environment,
    controllers,
    middlewares::trace,
    models::{Permission, Role, User},
    workers::{self, MailQueue},
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

    /// Initialises the application context and executes any startup commands.
    ///
    /// An [`AppContext`] is created from the provided configuration and all
    /// application services are initialised, including logging, database
    /// connections, and any other configured resources.
    ///
    /// If the application was started with a supported subcommand, the
    /// corresponding action is executed after initialisation. For example,
    /// the `seed` command populates the database with initial data.
    ///
    /// Returns a shared [`Arc`] containing the fully initialised application
    /// context.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The application context cannot be created from the configuration.
    /// - Application service initialisation fails.
    /// - Execution of a startup command fail
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
    /// - The mail queue fails to initialise.
    pub async fn run(&self) -> Result<()> {
        HookBuilder::new().theme(if std::io::stderr().is_terminal() {
            Theme::dark()
        } else {
            Theme::new()
        });

        let config = Config::from_env(&self.env)?;

        let ctx = self.init(&config).await?;

        let queue = MailQueue::init(config.redis()).await?;
        let welcome_backend = queue.welcome.clone();
        let forgot_backend = queue.forgot.clone();

        ctx.set_queue(queue);

        let ctx_worker = Arc::clone(&ctx);

        tokio::spawn(async move {
            tracing::info!("Worker started");
            Monitor::new()
                .register(
                    WorkerBuilder::new("mail-welcome")
                        .data(ctx_worker.clone())
                        .backend(welcome_backend)
                        .build_fn(workers::handle_welcome),
                )
                .register(
                    WorkerBuilder::new("mail-forgot")
                        .data(ctx_worker.clone())
                        .backend(forgot_backend)
                        .build_fn(workers::handle_forgot_password),
                )
                .run()
                .await
                .unwrap_or_else(|e| tracing::error!(error = ?e, "Queue monitor crashed" ));
        });

        let server = ctx.config().server();

        let listener = TcpListener::bind(server.address()).await?;

        let router = Router::new().nest("/api", controllers::router(&ctx)).layer(
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

    /// Seeds the database with initial application data.
    ///
    /// This method loads seed data from the `users.json`, `roles.json`, and
    /// `permissions.json` files and inserts it into the database using the
    /// application's seed routines.
    ///
    /// The operation is intended for development, testing, or bootstrapping
    /// environments where a predefined set of records is required.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The seed data file cannot be found or read.
    /// - The seed data is invalid or cannot be deserialised.
    /// - Database operations required to insert the seed data fail.
    pub async fn seed(db: &sqlx::PgPool) -> Result<()> {
        User::seed_data(db, "users.json").await?;
        Role::seed_data(db, "roles.json").await?;
        Permission::seed_data(db, "permissions.json").await?;

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
