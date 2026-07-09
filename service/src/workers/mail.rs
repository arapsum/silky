use std::sync::Arc;

use apalis::prelude::{Error, Storage as _};
use apalis_redis::{Config, ConnectionManager, RedisStorage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppContext,
    config::RedisConfig,
    mailer::AuthMailer,
    models::User,
    workers::{AppWorker, Result as WorkerResult, Worker},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct MailJob {
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

    pub(super) fn welcome_storage(&self) -> RedisStorage<MailJob> {
        self.welcome.clone()
    }

    pub(super) fn forgot_password_storage(&self) -> RedisStorage<MailJob> {
        self.forgot.clone()
    }
}

/// Sends account verification emails after registration.
#[derive(Clone)]
pub struct WelcomeMailWorker {
    pub ctx: AppContext,
}

impl AppWorker<MailJob> for WelcomeMailWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    fn name(&self) -> &'static str {
        "mail-welcome"
    }

    fn backend(&self) -> RedisStorage<MailJob> {
        self.ctx
            .queue()
            .get()
            .expect("mail queue must be initialised before registering workers")
            .welcome_storage()
    }
}

impl Worker<MailJob> for WelcomeMailWorker {
    async fn perform(&self, job: MailJob) -> WorkerResult {
        let user = User::find_by_pid(self.ctx.db(), job.user_id)
            .await
            .map_err(worker_error)?;

        AuthMailer::send_welcome(&self.ctx, &user, &job.token)
            .await
            .map_err(worker_error)?;

        Ok(())
    }
}

/// Sends forgot-password reset emails.
#[derive(Clone)]
pub struct ForgotPasswordMailWorker {
    pub ctx: AppContext,
}

impl AppWorker<MailJob> for ForgotPasswordMailWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    fn name(&self) -> &'static str {
        "mail-forgot"
    }

    fn backend(&self) -> RedisStorage<MailJob> {
        self.ctx
            .queue()
            .get()
            .expect("mail queue must be initialised before registering workers")
            .forgot_password_storage()
    }
}

impl Worker<MailJob> for ForgotPasswordMailWorker {
    async fn perform(&self, job: MailJob) -> WorkerResult {
        let user = User::find_by_pid(self.ctx.db(), job.user_id)
            .await
            .map_err(worker_error)?;

        AuthMailer::forgot_password(&self.ctx, &user, &job.token)
            .await
            .map_err(worker_error)?;

        Ok(())
    }
}

fn worker_error(error: impl Into<apalis::prelude::BoxDynError>) -> Error {
    Error::Failed(Arc::new(error.into()))
}
