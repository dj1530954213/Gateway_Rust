//! error.rs —— 报警引擎错误类型定义
//!
//! 统一的错误处理，包括：
//! - 配置错误
//! - 数据库错误  
//! - 通知发送错误
//! - 规则评估错误
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use thiserror::Error;

/// 报警引擎错误类型
#[derive(Debug, Error)]
pub enum AlertError {
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Rule evaluation error: {rule_id}")]
    RuleEvaluation { rule_id: uuid::Uuid },
    
    #[error("Notification error: {notifier} - {message}")]
    Notification { notifier: String, message: String },
    
    #[error("Email sending failed: {0}")]
    Email(#[from] lettre::error::Error),
    
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Frame bus error: {message}")]
    FrameBus { message: String },
    
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },
    
    #[error("Invalid threshold: {value}")]
    InvalidThreshold { value: String },
    
    #[error("Rule not found: {rule_id}")]
    RuleNotFound { rule_id: uuid::Uuid },
    
    #[error("Duplicate rule name: {name}")]
    DuplicateRuleName { name: String },
    
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl AlertError {
    /// 创建配置错误
    pub fn config_error(message: impl Into<String>) -> Self {
        AlertError::Config { message: message.into() }
    }
    
    /// 创建通知错误
    pub fn notification_error(notifier: impl Into<String>, message: impl Into<String>) -> Self {
        AlertError::Notification { 
            notifier: notifier.into(), 
            message: message.into() 
        }
    }
    
    /// 创建frame-bus错误
    pub fn frame_bus_error(message: impl Into<String>) -> Self {
        AlertError::FrameBus { message: message.into() }
    }
    
    /// 创建内部错误
    pub fn internal_error(message: impl Into<String>) -> Self {
        AlertError::Internal { message: message.into() }
    }
    
    /// 检查是否为临时错误（可重试）
    pub fn is_temporary(&self) -> bool {
        matches!(self, 
            AlertError::Http(_) |
            AlertError::Email(_) |
            AlertError::ServiceUnavailable { .. } |
            AlertError::FrameBus { .. }
        )
    }
    
    /// 检查是否为致命错误（需要停止服务）
    pub fn is_fatal(&self) -> bool {
        matches!(self, 
            AlertError::Config { .. } |
            AlertError::Database(_)
        )
    }
}

pub type AlertResult<T> = Result<T, AlertError>;