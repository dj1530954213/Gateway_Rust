//! Modbus驱动配置

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Modbus驱动配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModbusCfg {
    /// Modbus单元ID (1-247)
    pub unit_id: u8,
    /// 轮询间隔
    #[serde(with = "humantime_serde")]
    pub polling: Duration,
    /// 每次请求最大寄存器数量
    #[serde(default = "ModbusCfg::default_max_regs")]
    pub max_regs_per_req: u16,
    /// 重试次数
    #[serde(default = "ModbusCfg::default_retry")]
    pub retry: u8,
    /// 字节序
    #[serde(default)]
    pub endian: Endian,
    /// 是否启用写入
    #[serde(default)]
    pub enable_write: bool,
}

/// 字节序枚举
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Endian {
    #[default]
    Big,
    Little,
}

impl ModbusCfg {
    fn default_max_regs() -> u16 {
        120 // 低于Modbus 125限制，留安全余量
    }

    fn default_retry() -> u8 {
        3
    }
}

impl Default for ModbusCfg {
    fn default() -> Self {
        Self {
            unit_id: 1,
            polling: Duration::from_secs(1),
            max_regs_per_req: Self::default_max_regs(),
            retry: Self::default_retry(),
            endian: Endian::Big,
            enable_write: false,
        }
    }
}

/// 点位描述
#[derive(Debug, Clone)]
pub struct RegPoint {
    pub tag: String,
    pub func: tokio_modbus::FunctionCode,
    pub addr: u16,
    pub len: u16,
    pub datatype: DataType,
    pub scale: Option<String>, // Lua表达式
    pub access: Access,
}

/// 数据类型
#[derive(Debug, Clone)]
pub enum DataType {
    Bool,
    Uint16,
    Int16,
    Uint32,
    Int32,
    Float32,
    Float64,
}

/// 访问权限
#[derive(Debug, Clone)]
pub enum Access {
    R,  // 只读
    W,  // 只写
    RW, // 读写
}

/// 批量请求
#[derive(Debug, Clone)]
pub struct PollBatch {
    pub func: tokio_modbus::FunctionCode,
    pub start: u16,
    pub qty: u16,
    pub points: Vec<RegPoint>,
}