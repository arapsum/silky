use axum::Router;
use clap::Parser;
use tokio::net::TcpListener;

use crate::{Config, Result, config::Environment, controllers};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct App {
    #[arg(short, long, default_value_t = Environment::default())]
    env: Environment,
}

impl App {
    pub fn new() -> Self {
        Self::parse()
    }
    pub async fn run(&self) -> Result<()> {
        let config = Config::from_env(&self.env)?;

        let listener = TcpListener::bind(config.server().address()).await?;

        let router = Router::new().nest("/api", controllers::router());

        println!("Listening on {}", config.server().url());

        axum::serve(listener, router).await.map_err(Into::into)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
