use thiserror::Error;
use url::ParseError;
use sqlx::error::Error as SqlxError;
use aptos_sdk::move_types::account_address::AccountAddressParseError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] SqlxError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(&'static str),

    #[error("Blockchain error: {0}")]
    Blockchain(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

// Helper functions for common error cases
impl AppError {
    pub fn not_found(resource: &'static str) -> Self {
        AppError::NotFound(resource.to_string())
    }

    pub fn invalid_input(reason: &'static str) -> Self {
        AppError::InvalidInput(reason.to_string())
    }

    pub fn unauthorized(reason: &'static str) -> Self {
        AppError::Unauthorized(reason)
    }

    pub fn internal(message: impl ToString) -> Self {
        AppError::Internal(message.to_string())
    }
}

// Implement conversion from anyhow::Error
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<ParseError> for AppError {
    fn from(error: ParseError) -> Self {
        AppError::Internal(error.to_string())
    }
}

impl AppError {
    pub fn connection_error(message: &str) -> Self {
        AppError::Internal(format!("Connection error: {}", message))
    }

    pub fn transaction_error(message: &str) -> Self {
        AppError::Internal(format!("Transaction error: {}", message))
    }

    pub fn rate_limit_exceeded() -> Self {
        AppError::Internal("Rate limit exceeded".to_string())
    }

    pub fn health_check_failed(message: &str) -> Self {
        AppError::Internal(format!("Node health check failed: {}", message))
    }

    pub fn no_healthy_nodes() -> Self {
        AppError::Internal("All nodes are unhealthy".to_string())
    }

    pub fn config_error(message: &str) -> Self {
        AppError::Internal(format!("Configuration error: {}", message))
    }

    pub fn database_error(message: &str) -> Self {
        AppError::Internal(format!("Database error: {}", message))
    }

    pub fn invalid_request(message: &str) -> Self {
        AppError::Internal(format!("Invalid request: {}", message))
    }

    pub fn network_error(message: &str) -> Self {
        AppError::Internal(format!("Network error: {}", message))
    }

    pub fn serialization_error(message: &str) -> Self {
        AppError::Internal(format!("Serialization error: {}", message))
    }

    pub fn deserialization_error(message: &str) -> Self {
        AppError::Internal(format!("Deserialization error: {}", message))
    }

    pub fn event_subscription_error(message: &str) -> Self {
        AppError::Internal(format!("Event subscription error: {}", message))
    }

    pub fn chain_id_mismatch(expected: u8, actual: u8) -> Self {
        AppError::Internal(format!("Chain ID mismatch: expected {}, got {}", expected, actual))
    }

    pub fn account_error(message: &str) -> Self {
        AppError::Internal(format!("Account error: {}", message))
    }

    pub fn validation_error(message: &str) -> Self {
        AppError::Internal(format!("Validation error: {}", message))
    }

    pub fn transaction_isolation_error(message: &str) -> Self {
        AppError::Internal(format!("Database transaction error: {}", message))
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Internal(format!("IO error: {}", error))
    }
}

impl From<AccountAddressParseError> for AppError {
    fn from(error: AccountAddressParseError) -> Self {
        AppError::InvalidInput(error.to_string())
    }
} 