use std::sync::Arc;

use apalis_redis::Config as QueueConfig;
use redis::AsyncCommands as _;
use serial_test::serial;
use service::{
    AppContext, Config,
    config::Environment,
    workers::{
        AppWorker, ForgotPasswordMailWorker, MailQueue, Processor, WelcomeMailWorker, Workers,
    },
};
use uuid::Uuid;

const WELCOME_QUEUE: &str = "welcome-queue";
const FORGOT_QUEUE: &str = "forgot-queue";

#[tokio::test]
#[serial]
async fn welcome_mail_jobs_are_enqueued_in_the_welcome_queue() {
    let config = Config::from_env(&Environment::Testing).unwrap();
    let queue = MailQueue::init(config.redis()).await.unwrap();
    let user_id = Uuid::new_v4();
    let token = "welcome-token".to_string();

    clear_mail_queues(&config).await;

    queue.enqueue_welcome(user_id, token.clone()).await.unwrap();

    assert_eq!(active_jobs(&config, WELCOME_QUEUE).await, 1);
    assert_eq!(stored_jobs(&config, WELCOME_QUEUE).await, 1);
    assert_eq!(active_jobs(&config, FORGOT_QUEUE).await, 0);
    assert_eq!(stored_jobs(&config, FORGOT_QUEUE).await, 0);

    let job = stored_job_payload(&config, WELCOME_QUEUE).await;
    assert!(job.contains(&user_id.to_string()));
    assert!(job.contains(&token));

    clear_mail_queues(&config).await;
}

#[tokio::test]
#[serial]
async fn forgot_password_mail_jobs_are_enqueued_in_the_forgot_queue() {
    let config = Config::from_env(&Environment::Testing).unwrap();
    let queue = MailQueue::init(config.redis()).await.unwrap();
    let user_id = Uuid::new_v4();
    let token = "forgot-token".to_string();

    clear_mail_queues(&config).await;

    queue
        .enqueue_forgot_password(user_id, token.clone())
        .await
        .unwrap();

    assert_eq!(active_jobs(&config, FORGOT_QUEUE).await, 1);
    assert_eq!(stored_jobs(&config, FORGOT_QUEUE).await, 1);
    assert_eq!(active_jobs(&config, WELCOME_QUEUE).await, 0);
    assert_eq!(stored_jobs(&config, WELCOME_QUEUE).await, 0);

    let job = stored_job_payload(&config, FORGOT_QUEUE).await;
    assert!(job.contains(&user_id.to_string()));
    assert!(job.contains(&token));

    clear_mail_queues(&config).await;
}

#[tokio::test]
#[serial]
async fn workers_runtime_starts_and_shuts_down_cleanly() {
    let config = Config::from_env(&Environment::Testing).unwrap();
    let ctx = Arc::new(AppContext::try_from(&config).unwrap());

    clear_mail_queues(&config).await;

    let workers = Workers::init(&config).await.unwrap();
    ctx.set_queue(workers.mail_queue().clone());

    let mut processor = Processor::new();
    processor.register(WelcomeMailWorker::build(&ctx));
    processor.register(ForgotPasswordMailWorker::build(&ctx));

    let runtime = workers.start(processor);

    runtime.shutdown().await;

    clear_mail_queues(&config).await;
}

async fn active_jobs(config: &Config, namespace: &str) -> i64 {
    let keys = QueueKeys::new(namespace);
    let mut conn = redis_connection(config).await;

    conn.llen(keys.active_jobs).await.unwrap()
}

async fn stored_jobs(config: &Config, namespace: &str) -> i64 {
    let keys = QueueKeys::new(namespace);
    let mut conn = redis_connection(config).await;

    conn.hlen(keys.job_data).await.unwrap()
}

async fn stored_job_payload(config: &Config, namespace: &str) -> String {
    let keys = QueueKeys::new(namespace);
    let mut conn = redis_connection(config).await;
    let values: Vec<Vec<u8>> = conn.hvals(keys.job_data).await.unwrap();

    assert_eq!(values.len(), 1);

    String::from_utf8(values.into_iter().next().unwrap()).unwrap()
}

async fn clear_mail_queues(config: &Config) {
    let keys = [QueueKeys::new(WELCOME_QUEUE), QueueKeys::new(FORGOT_QUEUE)]
        .into_iter()
        .flat_map(QueueKeys::into_keys)
        .collect::<Vec<_>>();
    let mut conn = redis_connection(config).await;

    redis::cmd("DEL")
        .arg(keys)
        .query_async::<()>(&mut conn)
        .await
        .unwrap();
}

async fn redis_connection(config: &Config) -> redis::aio::MultiplexedConnection {
    config
        .redis()
        .connection()
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap()
}

struct QueueKeys {
    active_jobs: String,
    consumers: String,
    dead_jobs: String,
    done_jobs: String,
    failed_jobs: String,
    inflight_jobs: String,
    job_data: String,
    scheduled_jobs: String,
    signal: String,
}

impl QueueKeys {
    fn new(namespace: &str) -> Self {
        let config = QueueConfig::default().set_namespace(namespace);

        Self {
            active_jobs: config.active_jobs_list(),
            consumers: config.consumers_set(),
            dead_jobs: config.dead_jobs_set(),
            done_jobs: config.done_jobs_set(),
            failed_jobs: config.failed_jobs_set(),
            inflight_jobs: config.inflight_jobs_set(),
            job_data: config.job_data_hash(),
            scheduled_jobs: config.scheduled_jobs_set(),
            signal: config.signal_list(),
        }
    }

    fn into_keys(self) -> [String; 9] {
        [
            self.active_jobs,
            self.consumers,
            self.dead_jobs,
            self.done_jobs,
            self.failed_jobs,
            self.inflight_jobs,
            self.job_data,
            self.scheduled_jobs,
            self.signal,
        ]
    }
}
