use std::{io::IsTerminal, net::SocketAddr};

use axum::Router;
use clap::Parser;
use color_eyre::config::{HookBuilder, Theme};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{Config, Result, config::Environment, controllers, middlewares::trace};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct App {
    #[arg(short, long, default_value_t = Environment::default())]
    env: Environment,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::parse()
    }

    /// Starts the HTTP server and begins serving requests.
    ///
    /// Configuration is loaded from the environment specified by this
    /// application instance. The server then binds to the configured address
    /// and serves the application's routes under the `/api` path.
    ///
    /// This method does not return until the server shuts down or an error
    /// occurs.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The application configuration cannot be loaded.
    /// - The tracing setup fails.
    /// - The server cannot bind to the configured address.
    /// - The HTTP server encounters an error while serving requests.
    pub async fn run(&self) -> Result<()> {
        HookBuilder::new().theme(if std::io::stderr().is_terminal() {
            Theme::dark()
        } else {
            Theme::new()
        });

        let config = Config::from_env(&self.env)?;

        config.logger().setup()?;

        let listener = TcpListener::bind(config.server().address()).await?;

        let router = Router::new().nest("/api", controllers::router()).layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::make_span_with)
                .on_response(trace::on_response)
                .on_request(trace::on_request)
                .on_failure(trace::on_failure),
        );

        tracing::info!("Listening on {}", config.server().url());

        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(Into::into)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
