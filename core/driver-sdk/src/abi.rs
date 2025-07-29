//! abi.rs —— 驱动ABI接口规范
//!
//! 定义驱动二进制接口，确保驱动间的兼容性
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::driver::ProtocolKind;
use serde::{Deserialize, Serialize};

/// 驱动API版本
pub type DriverApiVersion = i16;

/// 当前驱动API版本
pub const DRIVER_API_VERSION: DriverApiVersion = 1;

/// 驱动元数据
/// 
/// 包含驱动的基本信息，用于驱动发现和加载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverMeta {
    /// 支持的协议类型
    pub protocol: ProtocolKind,
    
    /// 驱动版本
    pub version: String,
    
    /// 驱动名称
    pub name: String,
    
    /// 驱动描述
    pub description: String,
    
    /// API版本
    pub api_version: DriverApiVersion,
}

impl DriverMeta {
    /// 检查API版本兼容性
    pub fn is_compatible_with(&self, required_version: DriverApiVersion) -> bool {
        self.api_version >= required_version
    }
    
    /// 获取驱动的唯一标识符
    pub fn unique_id(&self) -> String {
        format!("{}:{}", self.protocol_name(), self.version)
    }
    
    /// 获取协议名称字符串
    pub fn protocol_name(&self) -> &str {
        match self.protocol {
            ProtocolKind::ModbusTcp => "modbus-tcp",
            ProtocolKind::OpcUa => "opc-ua",
            ProtocolKind::Mqtt => "mqtt",
        }
    }
}

/// 驱动加载状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DriverStatus {
    /// 未加载
    Unloaded,
    /// 加载中
    Loading,
    /// 已加载
    Loaded,
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 错误状态
    Error(String),
    /// 卸载中
    Unloading,
}

impl DriverStatus {
    /// 是否为活跃状态
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Initializing)
    }
    
    /// 是否为错误状态
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
    
    /// 获取状态描述
    pub fn description(&self) -> &str {
        match self {
            Self::Unloaded => "未加载",
            Self::Loading => "加载中",
            Self::Loaded => "已加载",
            Self::Initializing => "初始化中",
            Self::Running => "运行中",
            Self::Error(_) => "错误",
            Self::Unloading => "卸载中",
        }
    }
}

/// 驱动加载结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverLoadResult {
    /// 是否成功
    pub success: bool,
    
    /// 驱动元数据（成功时）
    pub meta: Option<DriverMeta>,
    
    /// 错误信息（失败时）
    pub error: Option<String>,
    
    /// 加载耗时（毫秒）
    pub load_time_ms: u64,
}

impl DriverLoadResult {
    /// 创建成功结果
    pub fn success(meta: DriverMeta, load_time_ms: u64) -> Self {
        Self {
            success: true,
            meta: Some(meta),
            error: None,
            load_time_ms,
        }
    }
    
    /// 创建失败结果
    pub fn failure<S: Into<String>>(error: S, load_time_ms: u64) -> Self {
        Self {
            success: false,
            meta: None,
            error: Some(error.into()),
            load_time_ms,
        }
    }
}

/// 驱动统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverStats {
    /// 关联的设备数量
    pub attached_devices: u32,
    
    /// 读取操作次数
    pub read_count: u64,
    
    /// 写入操作次数
    pub write_count: u64,
    
    /// 错误次数
    pub error_count: u64,
    
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    
    /// 上次操作时间
    pub last_operation_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for DriverStats {
    fn default() -> Self {
        Self {
            attached_devices: 0,
            read_count: 0,
            write_count: 0,
            error_count: 0,
            avg_response_time_ms: 0.0,
            last_operation_time: None,
        }
    }
}

impl DriverStats {
    /// 计算成功率
    pub fn success_rate(&self) -> f64 {
        let total = self.read_count + self.write_count;
        if total == 0 {
            1.0
        } else {
            1.0 - (self.error_count as f64 / total as f64)
        }
    }
    
    /// 记录操作
    pub fn record_operation(&mut self, is_error: bool, response_time_ms: f64) {
        if is_error {
            self.error_count += 1;
        }
        
        // 更新平均响应时间（简化的移动平均）
        let total_ops = self.read_count + self.write_count + 1;
        self.avg_response_time_ms = 
            (self.avg_response_time_ms * (total_ops - 1) as f64 + response_time_ms) / total_ops as f64;
        
        self.last_operation_time = Some(chrono::Utc::now());
    }
}