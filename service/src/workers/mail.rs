use std::sync::Arc;

use apalis::prelude::{Data, Error, Monitor, Storage as _, WorkerBuilder, WorkerFactoryFn};
use apalis_redis::{Config, ConnectionManager, RedisStorage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppContext, config::RedisConfig, mailer::AuthMailer, models::User};

#[derive(Debug, Serialize, Deserialize)]
struct MailJob {
    user_id: Uuid,
    token: String,
}

#[derive(Clone)]
pub struct MailQueue {
    welcome: RedisStorage<MailJob>,
    forgot: RedisStorage<MailJob>,
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
    /// This function will return an error if the Redis connection fails to
    /// establish.
    pub async fn init(cfg: &RedisConfig) -> crate::Result<Self> {
        let conn: ConnectionManager = apalis_redis::connect(cfg.url()).await?;

        Ok(Self {
            welcome: Self::storage(conn.clone(), "welcome-queue"),
            forgot: Self::storage(conn, "forgot-queue"),
        })
    }

    /// Queues a welcome email for asynchronous delivery.
    ///
    /// # Errors
    ///
    /// This function will return an error if the job cannot be pushed to
    /// Redis.
    pub async fn enqueue_welcome(&self, user_id: Uuid, token: String) -> crate::Result<()> {
        self.enqueue(self.welcome.clone(), user_id, token).await
    }

    /// Queues a forgot-password email for asynchronous delivery.
    ///
    /// # Errors
    ///
    /// This function will return an error if the job cannot be pushed to
    /// Redis.
    pub async fn enqueue_forgot_password(&self, user_id: Uuid, token: String) -> crate::Result<()> {
        self.enqueue(self.forgot.clone(), user_id, token).await
    }

    async fn enqueue(
        &self,
        mut queue: RedisStorage<MailJob>,
        user_id: Uuid,
        token: String,
    ) -> crate::Result<()> {
        queue.push(MailJob { user_id, token }).await?;

        Ok(())
    }

    fn storage(conn: ConnectionManager, namespace: &str) -> RedisStorage<MailJob> {
        RedisStorage::new_with_config(conn, Config::default().set_namespace(namespace))
    }
}

pub(super) fn register(monitor: Monitor, ctx: Arc<AppContext>, queue: &MailQueue) -> Monitor {
    monitor
        .register(
            WorkerBuilder::new("mail-welcome")
                .data(Arc::clone(&ctx))
                .backend(queue.welcome.clone())
                .build_fn(handle_welcome),
        )
        .register(
            WorkerBuilder::new("mail-forgot")
                .data(ctx)
                .backend(queue.forgot.clone())
                .build_fn(handle_forgot_password),
        )
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
/// - The user's verification token hash is `None`.
#[tracing::instrument(skip(ctx))]
async fn handle_welcome(job: MailJob, ctx: Data<Arc<AppContext>>) -> Result<(), Error> {
    let user = User::find_by_pid(ctx.db(), job.user_id)
        .await
        .map_err(|e| Error::Failed(Arc::new(e.into())))?;

    AuthMailer::send_welcome(&ctx, &user, &job.token)
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
#[tracing::instrument(skip(ctx))]
async fn handle_forgot_password(job: MailJob, ctx: Data<Arc<AppContext>>) -> Result<(), Error> {
    let user = User::find_by_pid(ctx.db(), job.user_id)
        .await
        .map_err(|e| Error::Failed(Arc::new(e.into())))?;

    AuthMailer::forgot_password(&ctx, &user, &job.token)
        .await
        .map_err(|e| Error::Failed(Arc::new(e.into())))?;

    Ok(())
}
