mod mail;

use std::{io, sync::Arc, time::Duration};

use apalis::prelude::Monitor;
use tokio::{sync::oneshot, task::JoinHandle};

use crate::{AppContext, Config, Result};

pub use self::mail::MailQueue;

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Workers {
    ctx: Arc<AppContext>,
    mail: MailQueue,
}

impl Workers {
    /// Initialises all background worker queues and shared worker state.
    ///
    /// Add new worker modules here by initialising their queue state and then
    /// registering them in [`Self::monitor`].
    ///
    /// # Errors
    ///
    /// This function will return an error if a worker queue cannot connect to
    /// its backing service.
    pub async fn init(config: &Config, ctx: Arc<AppContext>) -> Result<Self> {
        Ok(Self {
            ctx,
            mail: MailQueue::init(config.redis()).await?,
        })
    }

    #[must_use]
    pub const fn mail_queue(&self) -> &MailQueue {
        &self.mail
    }

    #[must_use]
    pub fn start(self) -> WorkerRuntime {
        WorkerRuntime::spawn(self.monitor())
    }

    fn monitor(self) -> Monitor {
        let monitor = Monitor::new();

        mail::register(monitor, self.ctx, &self.mail)
    }
}

pub struct WorkerRuntime {
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}

impl WorkerRuntime {
    fn spawn(monitor: Monitor) -> Self {
        let (shutdown, shutdown_rx) = oneshot::channel();

        let task = tokio::spawn(async move {
            tracing::info!("Worker monitor started");

            let shutdown_signal = async move {
                let _ = shutdown_rx.await;
                Ok::<(), io::Error>(())
            };

            if let Err(err) = monitor.run_with_signal(shutdown_signal).await {
                tracing::error!(error = ?err, "Worker monitor crashed");
            }

            tracing::info!("Worker monitor stopped");
        });

        Self {
            shutdown: Some(shutdown),
            task,
        }
    }

    pub async fn shutdown(mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }

        match tokio::time::timeout(SHUTDOWN_TIMEOUT, &mut self.task).await {
            Ok(Ok(())) => tracing::info!("Workers stopped"),
            Ok(Err(err)) => tracing::error!(error = ?err, "Worker task failed while shutting down"),
            Err(_) => {
                tracing::warn!(
                    timeout = ?SHUTDOWN_TIMEOUT,
                    "Worker shutdown timed out; aborting worker task"
                );

                self.task.abort();

                if let Err(err) = self.task.await {
                    if err.is_cancelled() {
                        tracing::info!("Worker task aborted");
                    } else {
                        tracing::error!(error = ?err, "Worker task failed after abort");
                    }
                }
            }
        }
    }
}
