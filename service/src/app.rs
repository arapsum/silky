use axum::Router;
use tokio::net::TcpListener;

use crate::{Config, Result, config::Environment, controllers};

pub struct App;

impl App {
    pub async fn run() -> Result<()> {
        let env = Environment::Development;

        let config = Config::from_env(&env)?;

        let listener = TcpListener::bind(config.server().address()).await?;

        let router = Router::new().nest("/api", controllers::router());

        println!("Listening on {}", config.server().url());

        axum::serve(listener, router).await.map_err(Into::into)
    }
}
