use std::{future, io::IsTerminal, net::SocketAddr, sync::Arc};

use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE, SET_COOKIE},
    },
};
use clap::Parser;
use color_eyre::config::{HookBuilder, Theme};
use dotenvy::dotenv;
use tokio::{net::TcpListener, signal};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    AppContext, Commands, Config, Result,
    config::Environment,
    controllers,
    middlewares::trace,
    models::{
        Address, Attribute, AttributeValue, Category, CategoryAttributeLink, Order, OrderDetail,
        Permission, Picture, Product, ProductOption, ProductVariant, Role, RolePermission, User,
        UserRole, VariantAttributeValue,
    },
    workers::{AppWorker, ForgotPasswordMailWorker, Processor, WelcomeMailWorker, Workers},
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

        dotenv()?;

        let config = Config::from_env(&self.env)?;

        let ctx = self.init(&config).await?;

        let workers = Workers::init(&config).await?;
        ctx.set_queue(workers.mail_queue().clone());
        let mut processor = Processor::new();
        Self::connect_workers(&mut processor, &ctx);
        let workers = workers.start(processor);

        let allowed_origins = config
            .cors()
            .allowed_origin()
            .iter()
            .map(|origin| origin.parse::<HeaderValue>())
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let cors_layer = CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::OPTIONS,
                Method::PATCH,
                Method::PUT,
                Method::DELETE,
            ])
            .allow_credentials(true)
            .allow_headers([CONTENT_TYPE, ACCEPT, COOKIE])
            .expose_headers([AUTHORIZATION, SET_COOKIE]);

        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(trace::make_span_with)
            .on_response(trace::on_response)
            .on_request(trace::on_request)
            .on_failure(trace::on_failure);

        let server = ctx.config().server();

        let listener = TcpListener::bind(server.address()).await?;

        let router = Router::new()
            .nest("/api", controllers::router(&ctx))
            .layer(trace_layer)
            .layer(cors_layer);

        tracing::info!("Listening on {}", server.url());

        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await?;

        tracing::info!("HTTP server stopped");

        workers.shutdown().await;

        ctx.db().close().await;
        tracing::info!("Database pool closed");

        Ok(())
    }

    /// Seeds the database with initial application data.
    ///
    /// This method loads seed data from the `users.json`, `roles.json`, etc
    /// files and inserts it into the database using the
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
        RolePermission::seed_data(db, "rolesPermission.json").await?;
        Permission::assign_all_to_role(db, "Administrator").await?;
        Category::seed_data(db, "categories.json").await?;
        Product::seed_data(db, "products.json").await?;
        Attribute::seed_data(db, "attributes.json").await?;
        CategoryAttributeLink::seed_data(db, "categoryAttributes.json").await?;
        AttributeValue::seed_data(db, "attributeValues.json").await?;
        ProductOption::seed_data(db, "productOptions.json").await?;
        ProductVariant::seed_data(db, "productVariants.json").await?;
        VariantAttributeValue::seed_data(db, "variantAttributeValues.json").await?;
        Picture::seed_data(db, "pictures.json").await?;
        UserRole::seed_data(db, "userRoles.json").await?;
        Address::seed_data(db, "addresses.json").await?;
        Order::seed_data(db, "orders.json").await?;
        OrderDetail::seed_data(db, "orderDetails.json").await?;

        Ok(())
    }

    fn connect_workers(processor: &mut Processor, ctx: &AppContext) {
        processor.register(WelcomeMailWorker::build(ctx));
        processor.register(ForgotPasswordMailWorker::build(ctx));
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = signal::ctrl_c().await {
            tracing::error!(error = ?err, "Failed to install Ctrl+C shutdown handler");
            future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(err) => {
                tracing::error!(error = ?err, "Failed to install SIGTERM shutdown handler");
                future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            tracing::info!("Received Ctrl+C shutdown signal");
        }
        () = terminate => {
            tracing::info!("Received terminate shutdown signal");
        }
    }
}
