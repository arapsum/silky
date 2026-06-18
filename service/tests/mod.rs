use std::sync::Arc;

use service::{AppContext, Config, Result, config::Environment};

mod models;
pub mod utils;

pub async fn boot_test() -> Result<Arc<AppContext>> {
    let config = Config::from_env(&Environment::Testing)?;

    let ctx = Arc::new(AppContext::try_from(&config)?);

    ctx.init().await?;

    Ok(ctx)
}
