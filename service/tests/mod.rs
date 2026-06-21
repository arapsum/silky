use std::sync::Arc;

use axum_test::{TestServer, TestServerConfig};
use service::{App, AppContext, AppState, Config, Result, config::Environment, controllers};
use sqlx::PgPool;

mod models;
mod requests;
pub mod utils;

pub async fn boot_test() -> Result<AppState> {
    let config = Config::from_env(&Environment::Testing)?;

    let ctx = Arc::new(AppContext::try_from(&config)?);

    ctx.init().await?;

    Ok(ctx)
}

pub async fn request<F, Fut>(f: F)
where
    F: FnOnce(TestServer, AppState) -> Fut,
    Fut: Future<Output = ()>,
{
    let ctx = boot_test().await.unwrap();

    let cfg = TestServerConfig {
        default_content_type: Some("application/json".into()),
        save_cookies: true,
        ..Default::default()
    };

    let server = TestServer::new_with_config(controllers::router(&ctx), cfg);

    f(server, ctx).await
}

pub async fn seed_data(db: &PgPool) -> Result<()> {
    App::seed(db).await
}
