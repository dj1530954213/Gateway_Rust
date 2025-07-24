//! 配置文件结构定义

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// endpoints.yml - 端点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointsConfig {
    /// 端点定义
    pub endpoints: HashMap<String, EndpointCfg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointCfg {
    /// 连接URL (tcp://host:port, tls://host:port, serial:///dev/ttyUSB0)
    pub url: String,
    
    /// 连接描述
    #[serde(default)]
    pub description: String,
    
    /// 连接超时
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub timeout: Duration,
    
    /// 连接池配置
    #[serde(default)]
    pub pool: PoolCfg,
    
    /// TLS配置
    #[serde(default)]
    pub tls: Option<TlsCfg>,
    
    /// 串口配置
    #[serde(default)]
    pub serial: Option<SerialCfg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolCfg {
    /// 最小连接数
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    
    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    
    /// 连接空闲超时
    #[serde(default = "default_idle_timeout", with = "humantime_serde")]
    pub idle_timeout: Duration,
    
    /// 连接最大生命周期
    #[serde(default = "default_max_lifetime", with = "humantime_serde")]
    pub max_lifetime: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsCfg {
    /// 服务器名称（SNI）
    #[serde(default)]
    pub server_name: String,
    
    /// 客户端证书路径
    #[serde(default)]
    pub cert_path: String,
    
    /// 客户端私钥路径
    #[serde(default)]
    pub key_path: String,
    
    /// CA证书路径
    #[serde(default)]
    pub ca_path: String,
    
    /// 是否验证服务器证书
    #[serde(default = "default_verify_cert")]
    pub verify_cert: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialCfg {
    /// 波特率
    #[serde(default = "default_baud_rate")]
    pub baud_rate: u32,
    
    /// 数据位
    #[serde(default = "default_data_bits")]
    pub data_bits: u8,
    
    /// 停止位
    #[serde(default = "default_stop_bits")]
    pub stop_bits: u8,
    
    /// 校验位
    #[serde(default)]
    pub parity: Parity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Parity {
    None,
    Odd,
    Even,
}

/// drivers.yml - 驱动配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriversConfig {
    /// 驱动实例定义
    pub drivers: HashMap<String, DriverCfg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverCfg {
    /// 驱动类型
    pub driver_type: String,
    
    /// 关联的端点
    pub endpoint: String,
    
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// 轮询间隔
    #[serde(default = "default_polling", with = "humantime_serde")]
    pub polling: Duration,
    
    /// 重试次数
    #[serde(default = "default_retry")]
    pub retry: u32,
    
    /// 驱动特定配置
    #[serde(flatten)]
    pub config: serde_yaml::Value,
}

/// variables.yml - 变量配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariablesConfig {
    /// 变量定义
    pub variables: HashMap<String, VariableCfg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableCfg {
    /// 变量描述
    #[serde(default)]
    pub description: String,
    
    /// 关联的驱动
    pub driver: String,
    
    /// 数据类型
    pub data_type: DataType,
    
    /// 地址或标识符
    pub address: String,
    
    /// 访问权限
    #[serde(default)]
    pub access: Access,
    
    /// 缩放表达式
    #[serde(default)]
    pub scale: Option<String>,
    
    /// 单位
    #[serde(default)]
    pub unit: String,
    
    /// 报警配置
    #[serde(default)]
    pub alarms: Vec<AlarmCfg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Bool,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Uint64,
    Float32,
    Float64,
    String,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Access {
    #[serde(rename = "r")]
    Read,
    #[serde(rename = "w")]
    Write,
    #[serde(rename = "rw")]
    ReadWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmCfg {
    /// 报警类型
    pub alarm_type: AlarmType,
    
    /// 比较值
    pub value: serde_yaml::Value,
    
    /// 报警级别
    #[serde(default)]
    pub level: AlarmLevel,
    
    /// 报警消息
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlarmType {
    High,
    Low,
    HighHigh,
    LowLow,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlarmLevel {
    Info,
    Warning,
    Critical,
}

// 默认值函数
fn default_timeout() -> Duration { Duration::from_secs(10) }
fn default_min_connections() -> u32 { 1 }
fn default_max_connections() -> u32 { 10 }
fn default_idle_timeout() -> Duration { Duration::from_secs(300) }
fn default_max_lifetime() -> Duration { Duration::from_secs(3600) }
fn default_verify_cert() -> bool { true }
fn default_baud_rate() -> u32 { 9600 }
fn default_data_bits() -> u8 { 8 }
fn default_stop_bits() -> u8 { 1 }
fn default_enabled() -> bool { true }
fn default_polling() -> Duration { Duration::from_secs(1) }
fn default_retry() -> u32 { 3 }

impl Default for PoolCfg {
    fn default() -> Self {
        Self {
            min_connections: default_min_connections(),
            max_connections: default_max_connections(),
            idle_timeout: default_idle_timeout(),
            max_lifetime: default_max_lifetime(),
        }
    }
}

impl Default for Parity {
    fn default() -> Self {
        Parity::None
    }
}

impl Default for Access {
    fn default() -> Self {
        Access::Read
    }
}

impl Default for AlarmLevel {
    fn default() -> Self {
        AlarmLevel::Info
    }
}