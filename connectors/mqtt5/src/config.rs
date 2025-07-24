//! MQTT5连接器配置

use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttCfg {
    /// 代理地址 (tcp://host:port, tls://host:port, ws://host:port)
    pub broker: String,
    
    /// 客户端ID，为空时自动生成UUID
    #[serde(default)]
    pub client_id: String,
    
    /// 用户名
    #[serde(default)]
    pub username: String,
    
    /// 密码
    #[serde(default)]
    pub password: String,
    
    /// QoS等级
    #[serde(default = "default_qos")]
    pub qos: u8,
    
    /// 主题前缀
    #[serde(default = "default_topic_prefix")]
    pub topic_prefix: String,
    
    /// 保持连接间隔
    #[serde(default = "default_keep_alive", with = "humantime_serde")]
    pub keep_alive: Duration,
    
    /// 连接超时
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub timeout: Duration,
    
    /// 重连间隔
    #[serde(default = "default_reconnect", with = "humantime_serde")]
    pub reconnect: Duration,
    
    /// 批量发送配置
    #[serde(default)]
    pub batch: BatchCfg,
    
    /// 压缩配置
    #[serde(default)]
    pub compression: CompressionCfg,
    
    /// 断网续传缓冲区大小
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    
    /// TLS配置
    #[serde(default)]
    pub tls: TlsCfg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCfg {
    /// 批量大小
    #[serde(default = "default_batch_size")]
    pub size: usize,
    
    /// 批量超时
    #[serde(default = "default_batch_timeout", with = "humantime_serde")]
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionCfg {
    /// 是否启用压缩
    #[serde(default)]
    pub enabled: bool,
    
    /// 压缩级别 (1-22)
    #[serde(default = "default_compression_level")]
    pub level: i32,
    
    /// 压缩阈值（字节）
    #[serde(default = "default_compression_threshold")]
    pub threshold: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsCfg {
    /// 是否启用TLS
    #[serde(default)]
    pub enabled: bool,
    
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

// 默认值函数
fn default_qos() -> u8 { 2 }
fn default_topic_prefix() -> String { "gateway".to_string() }
fn default_keep_alive() -> Duration { Duration::from_secs(60) }
fn default_timeout() -> Duration { Duration::from_secs(10) }
fn default_reconnect() -> Duration { Duration::from_secs(5) }
fn default_buffer_size() -> usize { 10000 }
fn default_batch_size() -> usize { 100 }
fn default_batch_timeout() -> Duration { Duration::from_millis(500) }
fn default_compression_level() -> i32 { 3 }
fn default_compression_threshold() -> usize { 1024 }
fn default_verify_cert() -> bool { true }

impl Default for MqttCfg {
    fn default() -> Self {
        Self {
            broker: "tcp://localhost:1883".to_string(),
            client_id: String::new(),
            username: String::new(),
            password: String::new(),
            qos: default_qos(),
            topic_prefix: default_topic_prefix(),
            keep_alive: default_keep_alive(),
            timeout: default_timeout(),
            reconnect: default_reconnect(),
            batch: BatchCfg::default(),
            compression: CompressionCfg::default(),
            buffer_size: default_buffer_size(),
            tls: TlsCfg::default(),
        }
    }
}

impl Default for BatchCfg {
    fn default() -> Self {
        Self {
            size: default_batch_size(),
            timeout: default_batch_timeout(),
        }
    }
}

impl Default for CompressionCfg {
    fn default() -> Self {
        Self {
            enabled: false,
            level: default_compression_level(),
            threshold: default_compression_threshold(),
        }
    }
}

impl Default for TlsCfg {
    fn default() -> Self {
        Self {
            enabled: false,
            server_name: String::new(),
            cert_path: String::new(),
            key_path: String::new(),
            ca_path: String::new(),
            verify_cert: default_verify_cert(),
        }
    }
}

/// MQTT消息内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttMessage {
    /// 设备ID
    pub device_id: String,
    
    /// 时间戳（毫秒）
    pub timestamp: u64,
    
    /// 数据点
    pub points: Vec<DataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// 标签
    pub tag: String,
    
    /// 值
    pub value: serde_json::Value,
    
    /// 质量等级 (0=Bad, 1=Uncertain, 2=Good)
    pub quality: u8,
    
    /// 元数据
    #[serde(default)]
    pub meta: std::collections::HashMap<String, String>,
}