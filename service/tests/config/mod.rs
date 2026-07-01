use std::{env, ffi::OsString};

use serial_test::serial;
use service::{
    Config,
    config::{Environment, RedisConfig},
};

struct EnvGuard {
    previous: Vec<(&'static str, Option<OsString>)>,
}

impl EnvGuard {
    fn set(vars: &[(&'static str, &'static str)]) -> Self {
        let previous = vars
            .iter()
            .map(|(key, _value)| (*key, env::var_os(key)))
            .collect::<Vec<_>>();

        for (key, value) in vars {
            // SAFETY: This test is marked serial and restores every modified
            // variable before returning.
            unsafe {
                env::set_var(key, value);
            }
        }

        Self { previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.previous {
            // SAFETY: This guard restores process environment variables that
            // were changed by a serial test.
            unsafe {
                match value {
                    Some(value) => env::set_var(key, value),
                    None => env::remove_var(key),
                }
            }
        }
    }
}

#[test]
fn environment_parses_known_aliases_and_displays() {
    assert_eq!(Environment::from("development"), Environment::Development);
    assert_eq!(Environment::from("dev"), Environment::Development);
    assert_eq!(Environment::from(" production "), Environment::Production);
    assert_eq!(Environment::from("prod"), Environment::Production);
    assert_eq!(Environment::from("testing"), Environment::Testing);
    assert_eq!(Environment::from("test"), Environment::Testing);

    let other = Environment::from("staging");

    assert_eq!(other.as_str(), "staging");
    assert_eq!(other.to_string(), "staging");
}

#[test]
#[serial]
fn loads_testing_config_from_yaml() {
    let config = Config::from_env(&Environment::Testing).unwrap();

    assert_eq!(config.server().address(), "127.0.0.1:7175");
    assert_eq!(config.server().url(), "http://127.0.0.1:7175");

    let database = config.database();
    assert_eq!(
        database.uri(),
        "postgresql://username:password@localhost:5432/database"
    );
    assert_eq!(database.max_connections(), 1);
    assert_eq!(database.min_connections(), 1);
    assert_eq!(database.connection_timeout(), 5);
    assert_eq!(database.idle_timeout(), 5);
    assert!(database.auto_migrate());
    assert!(!database.dangerously_truncate());
    assert!(database.dangerously_recreate());

    assert_eq!(config.redis().url(), "redis://127.0.0.1:6379/");

    let logger = config.logger();
    assert_eq!(logger.level().to_string(), "trace");
    assert_eq!(logger.format().to_string(), "full");
    assert_eq!(
        logger
            .directives()
            .unwrap()
            .into_iter()
            .map(|directive| directive.to_string())
            .collect::<Vec<_>>(),
        vec!["axum=trace", "service=trace"]
    );

    let smtp = config.mailer().smtp();
    assert!(smtp.enable());
    assert_eq!(smtp.host(), "localhost");
    assert_eq!(smtp.port(), 1025);
    assert!(!smtp.secure());
    assert!(smtp.auth().is_none());

    let auth = config.auth();
    assert_eq!(
        auth.access().public_key().to_string_lossy(),
        "secrets/keys/test/access_key_pub.pem"
    );
    assert_eq!(
        auth.access().private_key().to_string_lossy(),
        "secrets/keys/test/access_key.pem"
    );
    assert_eq!(auth.access().maxage(), 900);
    assert_eq!(
        auth.refresh().public_key().to_string_lossy(),
        "secrets/keys/test/refresh_key_pub.pem"
    );
    assert_eq!(
        auth.refresh().private_key().to_string_lossy(),
        "secrets/keys/test/refresh_key.pem"
    );
    assert_eq!(auth.refresh().maxage(), 604_800);
    assert_eq!(auth.verification_token_expiry(), 86_400);
    assert_eq!(auth.refresh_token_expiry(), 900);
    assert!(auth.access().encoding_key().is_ok());
    assert!(auth.access().decoding_key().is_ok());
    assert!(auth.refresh().encoding_key().is_ok());
    assert!(auth.refresh().decoding_key().is_ok());
}

#[test]
#[serial]
fn loads_development_config_from_yaml() {
    let config = Config::from_env(&Environment::Development).unwrap();

    assert_eq!(config.server().address(), "127.0.0.1:7150");
    assert_eq!(config.server().url(), "http://127.0.0.1:7150");
    assert_eq!(config.database().max_connections(), 10);
    assert_eq!(config.database().min_connections(), 1);
    assert!(config.database().auto_migrate());
    assert!(!config.database().dangerously_recreate());
    assert_eq!(config.redis().url(), "redis://localhost:6379");
    assert_eq!(config.logger().level().to_string(), "debug");
    assert_eq!(config.logger().format().to_string(), "pretty");
    assert_eq!(
        config.auth().access().public_key().to_string_lossy(),
        "secrets/keys/dev/access_key_pub.pem"
    );
    assert_eq!(
        config.auth().refresh().private_key().to_string_lossy(),
        "secrets/keys/dev/refresh_key.pem"
    );
}

#[test]
#[serial]
fn environment_variables_override_yaml_config_values() {
    let _guard = EnvGuard::set(&[
        ("APP_SERVER_PORT", "8181"),
        (
            "APP_DATABASE_URI",
            "postgresql://user:pass@localhost:5432/override",
        ),
        ("APP_REDIS_URL", "redis://example.com:6380/"),
        ("APP_LOGGER_LEVEL", "warn"),
        ("APP_MAILER_SMTP_PORT", "2525"),
        ("APP_AUTH_ACCESS_MAXAGE", "123"),
    ]);

    let config = Config::from_env(&Environment::Testing).unwrap();

    assert_eq!(config.server().address(), "127.0.0.1:8181");
    assert_eq!(
        config.database().uri(),
        "postgresql://user:pass@localhost:5432/override"
    );
    assert_eq!(config.redis().url(), "redis://example.com:6380/");
    assert_eq!(config.logger().level().to_string(), "warn");
    assert_eq!(
        config
            .logger()
            .directives()
            .unwrap()
            .into_iter()
            .map(|directive| directive.to_string())
            .collect::<Vec<_>>(),
        vec!["axum=warn", "service=warn"]
    );
    assert_eq!(config.mailer().smtp().port(), 2525);
    assert_eq!(config.auth().access().maxage(), 123);
}

#[test]
#[serial]
fn missing_environment_file_returns_error() {
    let result = Config::from_env(&Environment::Other("missing-config".to_string()));

    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn database_pool_is_created_lazily_from_config() {
    let config = Config::from_env(&Environment::Testing).unwrap();
    let pool = config.database().pool().unwrap();

    assert_eq!(pool.size(), 0);

    pool.close().await;
}

#[test]
#[serial]
fn redis_connection_accepts_valid_urls_and_rejects_invalid_urls() {
    let config = Config::from_env(&Environment::Testing).unwrap();

    assert!(config.redis().connection().is_ok());
    assert!(
        RedisConfig {
            url: "not-a-redis-url".to_string(),
        }
        .connection()
        .is_err()
    );
}
