/*!
# Protocol Bridge Error Types

协议桥接系统的错误类型定义
*/

use thiserror::Error;

/// 协议桥接错误类型
#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Plugin load failed: {0}")]
    PluginLoadFailed(String),

    #[error("Plugin ABI mismatch: expected {expected}, got {actual}")]
    ABIMismatch { expected: String, actual: String },

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("OPC-UA error: {0}")]
    OpcUa(String),

    #[error("Modbus error: {0}")]
    Modbus(String),

    #[error("Protocol not supported: {0}")]
    UnsupportedProtocol(String),

    #[error("Bridge not found: {0}")]
    BridgeNotFound(String),

    #[error("Bridge already exists: {0}")]
    BridgeExists(String),

    #[error("Bridge state error: {0}")]
    BridgeState(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Dynamic library error: {0}")]
    DynamicLib(String),

    #[error("Memory mapping error: {0}")]
    MemoryMapping(String),

    #[error("Timeout error: operation timed out")]
    Timeout,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl BridgeError {
    /// 创建插件错误
    pub fn plugin<S: Into<String>>(msg: S) -> Self {
        Self::Plugin(msg.into())
    }

    /// 创建运行时错误
    pub fn runtime<S: Into<String>>(msg: S) -> Self {
        Self::Runtime(msg.into())
    }

    /// 创建配置错误
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    /// 创建OPC-UA错误
    pub fn opcua<S: Into<String>>(msg: S) -> Self {
        Self::OpcUa(msg.into())
    }

    /// 创建Modbus错误
    pub fn modbus<S: Into<String>>(msg: S) -> Self {
        Self::Modbus(msg.into())
    }

    /// 创建内部错误
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }

    /// 检查是否为重试错误
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Timeout | Self::Io(_) | Self::ResourceExhausted(_)
        )
    }

    /// 检查是否为配置错误
    pub fn is_config_error(&self) -> bool {
        matches!(self, Self::Config(_) | Self::ABIMismatch { .. })
    }

    /// 检查是否为插件错误
    pub fn is_plugin_error(&self) -> bool {
        matches!(
            self,
            Self::Plugin(_) | Self::PluginNotFound(_) | Self::PluginLoadFailed(_) | Self::ABIMismatch { .. }
        )
    }
}

/// 用于简化错误处理的扩展trait
pub trait BridgeErrorExt<T> {
    /// 将错误转换为BridgeError::Plugin
    fn map_plugin_err<S: Into<String>>(self, msg: S) -> Result<T, BridgeError>;

    /// 将错误转换为BridgeError::Runtime
    fn map_runtime_err<S: Into<String>>(self, msg: S) -> Result<T, BridgeError>;

    /// 将错误转换为BridgeError::Config
    fn map_config_err<S: Into<String>>(self, msg: S) -> Result<T, BridgeError>;
}

impl<T, E> BridgeErrorExt<T> for Result<T, E>
where
    E: std::error::Error,
{
    fn map_plugin_err<S: Into<String>>(self, msg: S) -> Result<T, BridgeError> {
        self.map_err(|_| BridgeError::plugin(msg))
    }

    fn map_runtime_err<S: Into<String>>(self, msg: S) -> Result<T, BridgeError> {
        self.map_err(|_| BridgeError::runtime(msg))
    }

    fn map_config_err<S: Into<String>>(self, msg: S) -> Result<T, BridgeError> {
        self.map_err(|_| BridgeError::config(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = BridgeError::plugin("test error");
        assert!(matches!(err, BridgeError::Plugin(_)));
        assert!(err.is_plugin_error());
    }

    #[test]
    fn test_error_properties() {
        let timeout_err = BridgeError::Timeout;
        assert!(timeout_err.is_retryable());

        let config_err = BridgeError::config("bad config");
        assert!(config_err.is_config_error());
    }

    #[test]
    fn test_error_ext() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));

        let bridge_result = result.map_plugin_err("plugin load failed");
        assert!(bridge_result.is_err());
        assert!(bridge_result.unwrap_err().is_plugin_error());
    }
}