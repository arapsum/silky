#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
