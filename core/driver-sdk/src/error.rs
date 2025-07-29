//! error.rs —— 驱动错误类型定义
//!
//! 定义驱动开发和运行时错误类型
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use thiserror::Error;

/// 驱动操作结果类型
pub type DriverResult<T> = Result<T, DriverError>;

/// 驱动错误枚举
#[derive(Error, Debug)]
pub enum DriverError {
    #[error("Driver initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Device connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Device {device_id} not found")]
    DeviceNotFound { device_id: uuid::Uuid },

    #[error("Tag address '{address}' not found on device {device_id}")]
    TagNotFound {
        device_id: uuid::Uuid,
        address: String,
    },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Communication timeout: {0}")]
    Timeout(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Data parsing error: {0}")]
    DataParsingError(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Driver internal error: {0}")]
    InternalError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl DriverError {
    /// 创建连接失败错误
    pub fn connection_failed<S: Into<String>>(msg: S) -> Self {
        Self::ConnectionFailed(msg.into())
    }

    /// 创建协议错误
    pub fn protocol_error<S: Into<String>>(msg: S) -> Self {
        Self::ProtocolError(msg.into())
    }

    /// 创建超时错误
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Self::Timeout(msg.into())
    }

    /// 创建配置错误
    pub fn invalid_config<S: Into<String>>(msg: S) -> Self {
        Self::InvalidConfig(msg.into())
    }

    /// 创建内部错误
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::InternalError(msg.into())
    }
}