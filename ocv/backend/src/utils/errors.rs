use thiserror::Error;
use async_graphql::ErrorExtensions;

/// Application-specific errors
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Internal error: {0}")]
    InternalError(#[from] anyhow::Error),
}

impl ErrorExtensions for AppError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_, e| {
            match self {
                AppError::NotFound(_) => e.set("code", "NOT_FOUND"),
                AppError::Unauthorized(_) => e.set("code", "UNAUTHORIZED"),
                AppError::ValidationError(_) => e.set("code", "VALIDATION_ERROR"),
                AppError::DatabaseError(_) => e.set("code", "DATABASE_ERROR"),
                AppError::EncryptionError(_) => e.set("code", "ENCRYPTION_ERROR"),
                AppError::InternalError(_) => e.set("code", "INTERNAL_ERROR"),
            };
        })
    }
}
