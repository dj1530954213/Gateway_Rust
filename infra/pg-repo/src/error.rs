//! error.rs —— 仓储层错误处理
//!
//! 定义统一的仓储层错误类型，方便上层转换为API错误
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Conflict: {message}")]
    Conflict { message: String },
    
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("UUID parse error: {0}")]
    UuidParse(#[from] uuid::Error),
    
    #[error("Chrono parse error: {0}")]
    ChronoParse(#[from] chrono::ParseError),
}

impl RepoError {
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound { resource: resource.into() }
    }
    
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict { message: message.into() }
    }
    
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation { message: message.into() }
    }
}

pub type RepoResult<T> = Result<T, RepoError>;