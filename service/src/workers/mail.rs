use std::sync::Arc;

use apalis::prelude::*;
use apalis_redis::{Config, ConnectionManager, RedisStorage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppContext, config::RedisConfig, mailer::AuthMailer, models::User};

#[derive(Debug, Serialize, Deserialize)]
pub struct MailJob {
    pub user_id: Uuid,
}

pub struct MailQueue {
    pub welcome: RedisStorage<MailJob>,
    pub forgot: RedisStorage<MailJob>,
}

impl MailQueue {
    /// Initializes the mail queue with the given Redis configuration.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - The mail queue was initialized successfully.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The Redis connection fails to establish.
    /// - The `MAILER_TEMPLATES` lazy lock fails to initialize.
    /// - The `HandlebarsTemplate` fails to clone.
    /// - The user's reset token hash is `None`.
    pub async fn init(cfg: &RedisConfig) -> crate::Result<Self> {
        let conn: ConnectionManager = apalis_redis::connect(cfg.url()).await?;

        Ok(Self {
            welcome: RedisStorage::new_with_config(
                conn.clone(),
                Config::default().set_namespace("welcome-queue"),
            ),
            forgot: RedisStorage::new_with_config(
                conn,
                Config::default().set_namespace("forgot-queue"),
            ),
        })
    }
}

/// Handles the welcome email job by sending a welcome email to the user.
///
/// # Returns
///
/// * `Ok(())` - The email was sent successfully.
///
/// # Errors
///
/// This function will return an error if:
/// - The `MAILER_TEMPLATES` lazy lock fails to initialize.
/// - The `HandlebarsTemplate` fails to clone.
/// - The user's reset token hash is `None`.
pub async fn handle_welcome(job: MailJob, ctx: Data<Arc<AppContext>>) -> Result<(), Error> {
    let user = User::find_by_pid(ctx.db(), job.user_id)
        .await
        .map_err(|e| Error::Failed(Arc::new(e.into())))?;

    AuthMailer::send_welcome(&ctx, &user)
        .await
        .map_err(|e| Error::Failed(Arc::new(e.into())))?;

    Ok(())
}

/// Handles the forgot password email job by sending a forgot password email to the user.
///
/// # Returns
///
/// * `Ok(())` - The email was sent successfully.
///
/// # Errors
///
/// This function will return an error if:
/// - The `MAILER_TEMPLATES` lazy lock fails to initialize.
/// - The `HandlebarsTemplate` fails to clone.
/// - The user's reset token hash is `None`.
pub async fn handle_forgot_password(job: MailJob, ctx: Data<Arc<AppContext>>) -> Result<(), Error> {
    let user = User::find_by_pid(ctx.db(), job.user_id)
        .await
        .map_err(|e| Error::Failed(Arc::new(e.into())))?;

    AuthMailer::forgot_password(&ctx, &user)
        .await
        .map_err(|e| Error::Failed(Arc::new(e.into())))?;

    Ok(())
}
