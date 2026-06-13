use axum::Router;
use tokio::net::TcpListener;

use crate::controllers;

pub struct App;

impl App {
    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:5150").await?;

        let router = Router::new().nest("/api", controllers::router());

        println!("Listening on http://127.0.0.1:5150/api");

        axum::serve(listener, router).await.map_err(Into::into)
    }
}
