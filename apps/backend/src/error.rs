use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Node connection error: {0}")]
    ConnectionError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Node health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("All nodes are unhealthy")]
    NoHealthyNodes,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not authorized: {0}")]
    NotAuthorized(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<ParseError> for ClientError {
    fn from(error: ParseError) -> Self {
        ClientError::ConfigError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ClientError>; 