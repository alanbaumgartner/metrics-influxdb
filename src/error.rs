use thiserror::Error;

pub type Result<T> = std::result::Result<T, InfluxError>;

#[derive(Debug, Error)]
pub enum InfluxError {
    #[error("{error}")]
    AuthorizationError { error: String },
    #[error("{error}")]
    AuthenticationError { error: String },
    #[error("{error}")]
    BadRequest { error: String },
    #[error("{error}")]
    ContentTooLarge { error: String },
    #[error("Connection error: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
