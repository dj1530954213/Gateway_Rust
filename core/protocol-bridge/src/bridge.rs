/*!
# Protocol Bridge Abstraction

协议桥接抽象层，定义统一的桥接接口
*/

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::{BridgeError, Result};

/// 桥接类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BridgeType {
    /// OPC-UA Server桥接
    OpcUaServer,
    /// Modbus Slave桥接
    ModbusSlave,
    /// 自定义桥接
    Custom(String),
}

impl std::fmt::Display for BridgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeType::OpcUaServer => write!(f, "opcua-server"),
            BridgeType::ModbusSlave => write!(f, "modbus-slave"),
            BridgeType::Custom(name) => write!(f, "custom-{}", name),
        }
    }
}

/// 桥接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// 桥接名称
    pub name: String,
    /// 桥接类型
    pub bridge_type: BridgeType,
    /// 是否启用
    pub enabled: bool,
    /// 绑定地址
    pub bind_address: String,
    /// 绑定端口
    pub port: u16,
    /// 最大连接数
    pub max_connections: usize,
    /// 连接超时（秒）
    pub connection_timeout: u64,
    /// 扩展配置
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            name: "default-bridge".to_string(),
            bridge_type: BridgeType::Custom("unknown".to_string()),
            enabled: true,
            bind_address: "0.0.0.0".to_string(),
            port: 0,
            max_connections: 100,
            connection_timeout: 30,
            extra: HashMap::new(),
        }
    }
}

/// 桥接状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeState {
    /// 已停止
    Stopped,
    /// 正在启动
    Starting,
    /// 正在运行
    Running,
    /// 正在停止
    Stopping,
    /// 错误状态
    Error(String),
}

/// 桥接统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStats {
    /// 当前连接数
    pub connections: usize,
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
    /// 启动时间
    pub start_time: Option<SystemTime>,
    /// 最后活动时间
    pub last_activity: Option<SystemTime>,
}

impl Default for BridgeStats {
    fn default() -> Self {
        Self {
            connections: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time: 0.0,
            start_time: None,
            last_activity: None,
        }
    }
}

/// 数据点定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// 数据点ID
    pub id: String,
    /// 数据点名称
    pub name: String,
    /// 数据类型
    pub data_type: DataType,
    /// 访问权限
    pub access: AccessLevel,
    /// 当前值
    pub value: Option<DataValue>,
    /// 最后更新时间
    pub last_updated: Option<SystemTime>,
    /// 质量标记
    pub quality: Quality,
}

/// 数据类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Boolean,
    Int16,
    Int32,
    Int64,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Double,
    String,
    ByteArray,
    DateTime,
}

/// 访问级别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    /// 只读
    ReadOnly,
    /// 只写
    WriteOnly,
    /// 读写
    ReadWrite,
}

/// 数据值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataValue {
    Boolean(bool),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float(f32),
    Double(f64),
    String(String),
    ByteArray(Vec<u8>),
    DateTime(SystemTime),
}

/// 质量标记
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quality {
    /// 良好
    Good,
    /// 不确定
    Uncertain,
    /// 坏的
    Bad,
}

/// 协议桥接抽象接口
#[async_trait]
pub trait ProtocolBridge {
    /// 获取桥接配置
    fn config(&self) -> &BridgeConfig;

    /// 获取桥接状态
    async fn state(&self) -> BridgeState;

    /// 获取桥接统计信息
    async fn stats(&self) -> BridgeStats;

    /// 启动桥接
    async fn start(&self) -> Result<()>;

    /// 停止桥接
    async fn stop(&self) -> Result<()>;

    /// 重启桥接
    async fn restart(&self) -> Result<()> {
        self.stop().await?;
        self.start().await
    }

    /// 添加数据点
    async fn add_data_point(&self, data_point: DataPoint) -> Result<()>;

    /// 移除数据点
    async fn remove_data_point(&self, data_point_id: &str) -> Result<()>;

    /// 获取数据点
    async fn get_data_point(&self, data_point_id: &str) -> Result<Option<DataPoint>>;

    /// 获取所有数据点
    async fn list_data_points(&self) -> Result<Vec<DataPoint>>;

    /// 读取数据点值
    async fn read_value(&self, data_point_id: &str) -> Result<Option<DataValue>>;

    /// 写入数据点值
    async fn write_value(&self, data_point_id: &str, value: DataValue) -> Result<()>;

    /// 批量读取数据点值
    async fn read_multiple(&self, data_point_ids: &[String]) -> Result<HashMap<String, Option<DataValue>>>;

    /// 批量写入数据点值
    async fn write_multiple(&self, values: HashMap<String, DataValue>) -> Result<()>;

    /// 订阅数据点变化
    async fn subscribe(&self, data_point_ids: &[String]) -> Result<String>;

    /// 取消订阅
    async fn unsubscribe(&self, subscription_id: &str) -> Result<()>;

    /// 健康检查
    async fn health_check(&self) -> Result<bool>;

    /// 获取桥接信息
    async fn info(&self) -> Result<HashMap<String, serde_json::Value>>;
}

/// 桥接事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEvent {
    /// 桥接启动
    Started {
        bridge_name: String,
        timestamp: SystemTime,
    },
    /// 桥接停止
    Stopped {
        bridge_name: String,
        timestamp: SystemTime,
    },
    /// 客户端连接
    ClientConnected {
        bridge_name: String,
        client_id: String,
        timestamp: SystemTime,
    },
    /// 客户端断开
    ClientDisconnected {
        bridge_name: String,
        client_id: String,
        timestamp: SystemTime,
    },
    /// 数据点值变化
    DataValueChanged {
        bridge_name: String,
        data_point_id: String,
        old_value: Option<DataValue>,
        new_value: DataValue,
        timestamp: SystemTime,
    },
    /// 错误事件
    Error {
        bridge_name: String,
        error: String,
        timestamp: SystemTime,
    },
}

/// 事件监听器
#[async_trait]
pub trait BridgeEventListener {
    /// 处理桥接事件
    async fn on_event(&self, event: BridgeEvent) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_type_display() {
        assert_eq!(BridgeType::OpcUaServer.to_string(), "opcua-server");
        assert_eq!(BridgeType::ModbusSlave.to_string(), "modbus-slave");
        assert_eq!(
            BridgeType::Custom("test".to_string()).to_string(),
            "custom-test"
        );
    }

    #[test]
    fn test_bridge_config_default() {
        let config = BridgeConfig::default();
        assert_eq!(config.name, "default-bridge");
        assert!(config.enabled);
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_data_value_serialization() {
        let value = DataValue::Int32(42);
        let json = serde_json::to_string(&value).unwrap();
        let deserialized: DataValue = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            DataValue::Int32(v) => assert_eq!(v, 42),
            _ => panic!("Unexpected value type"),
        }
    }

    #[test]
    fn test_bridge_state() {
        let state = BridgeState::Running;
        assert_eq!(state, BridgeState::Running);
        
        let error_state = BridgeState::Error("test error".to_string());
        assert!(matches!(error_state, BridgeState::Error(_)));
    }
}