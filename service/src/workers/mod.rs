mod mail;

use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use apalis::prelude::{
    Backend, BoxDynError, Error, Monitor, Request, WorkerBuilder, WorkerFactory,
};
use apalis_redis::{RedisContext, RedisStorage};
use serde::{Serialize, de::DeserializeOwned};
use tokio::{sync::oneshot, task::JoinHandle};
use tower::{Layer, Service};

use crate::{AppContext, Config, Result as AppResult};

pub use self::mail::{ForgotPasswordMailWorker, MailQueue, WelcomeMailWorker};

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

pub type Result<T = ()> = std::result::Result<T, Error>;

pub trait Worker<Args>: Clone + Send + Sync + 'static {
    fn perform(&self, args: Args) -> impl Future<Output = Result> + Send + '_;
}

pub trait AppWorker<Args>: Worker<Args>
where
    Args: Serialize + DeserializeOwned + Sync + Send + Unpin + 'static,
{
    fn build(ctx: &AppContext) -> Self;

    fn name(&self) -> &'static str;

    fn backend(&self) -> RedisStorage<Args>;
}

pub struct Processor {
    monitor: Monitor,
}

impl Processor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            monitor: Monitor::new(),
        }
    }

    pub fn register<W, Args>(&mut self, worker: W)
    where
        W: AppWorker<Args>,
        Args: Serialize + DeserializeOwned + Sync + Send + Unpin + 'static,
        RedisStorage<Args>: Backend<Request<Args, RedisContext>>,
        <RedisStorage<Args> as Backend<Request<Args, RedisContext>>>::Stream:
            Unpin + Send + 'static,
        <RedisStorage<Args> as Backend<Request<Args, RedisContext>>>::Layer:
            Layer<WorkerService<W>> + Send,
        <<RedisStorage<Args> as Backend<Request<Args, RedisContext>>>::Layer as Layer<
            WorkerService<W>,
        >>::Service: Service<Request<Args, RedisContext>> + Send,
        <<<RedisStorage<Args> as Backend<Request<Args, RedisContext>>>::Layer as Layer<
            WorkerService<W>,
        >>::Service as Service<Request<Args, RedisContext>>>::Future: Send,
        <<<RedisStorage<Args> as Backend<Request<Args, RedisContext>>>::Layer as Layer<
            WorkerService<W>,
        >>::Service as Service<Request<Args, RedisContext>>>::Error:
            Send + Sync + Into<BoxDynError>,
    {
        let monitor = std::mem::replace(&mut self.monitor, Monitor::new());
        self.monitor = monitor.register(
            WorkerBuilder::new(worker.name())
                .backend(worker.backend())
                .build(WorkerService::new(worker)),
        );
    }

    #[must_use]
    fn into_monitor(self) -> Monitor {
        self.monitor
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct WorkerService<W> {
    worker: W,
}

impl<W> WorkerService<W> {
    const fn new(worker: W) -> Self {
        Self { worker }
    }
}

impl<W, Args> Service<Request<Args, RedisContext>> for WorkerService<W>
where
    W: Worker<Args>,
    Args: Send + 'static,
{
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Args, RedisContext>) -> Self::Future {
        let worker = self.worker.clone();

        Box::pin(async move { worker.perform(req.args).await })
    }
}

pub struct Workers {
    mail: MailQueue,
}

impl Workers {
    /// Initialises all background worker queues and shared worker state.
    ///
    /// Add new worker modules by initialising their queue state here and then
    /// registering their worker structs on a [`Processor`].
    ///
    /// # Errors
    ///
    /// This function will return an error if a worker queue cannot connect to
    /// its backing service.
    pub async fn init(config: &Config) -> AppResult<Self> {
        Ok(Self {
            mail: MailQueue::init(config.redis()).await?,
        })
    }

    #[must_use]
    pub const fn mail_queue(&self) -> &MailQueue {
        &self.mail
    }

    #[must_use]
    pub fn start(self, processor: Processor) -> WorkerRuntime {
        WorkerRuntime::spawn(processor.into_monitor())
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
