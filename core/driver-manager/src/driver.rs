//! Driver trait 定义

use async_trait::async_trait;
use serde_json::Value;
use std::fmt;

/// 驱动类型
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DriverKind {
    Static,  // 静态编译驱动
    Dyn,     // 动态库驱动
    Wasm,    // WASM驱动
}

impl fmt::Display for DriverKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriverKind::Static => write!(f, "static"),
            DriverKind::Dyn => write!(f, "dyn"),
            DriverKind::Wasm => write!(f, "wasm"),
        }
    }
}

/// 驱动元信息
#[derive(Debug, Clone)]
pub struct DriverMeta {
    pub name: String,
    pub kind: DriverKind,
    pub version: String,
    pub api_version: u16,
    pub description: String,
    pub features: Vec<String>,
}

/// 驱动状态
#[derive(Debug, Clone, PartialEq)]
pub enum DriverState {
    Loading,
    Init,
    Connected,
    Active,
    Error(String),
    Fault,
    Shutdown,
}

/// 核心驱动trait
#[async_trait]
pub trait Driver: Send + Sync {
    /// 获取驱动元信息
    fn meta(&self) -> DriverMeta;

    /// 初始化驱动（解析配置，不做I/O）
    async fn init(&mut self, cfg: &Value) -> anyhow::Result<()>;

    /// 建立连接（注入EndpointHandle）
    async fn connect(&mut self, pool: std::sync::Arc<endpoint_kit::EndpointHandle>) -> anyhow::Result<()>;

    /// 循环读取数据（主要工作循环）
    async fn read_loop(&mut self, tx: frame_bus::FrameSender) -> anyhow::Result<()>;

    /// 写入命令（可选实现）
    async fn write(&mut self, _cmd: frame_bus::CmdFrame) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Write not supported"))
    }

    /// 优雅关闭
    async fn shutdown(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// 驱动工厂函数类型
pub type DriverFactory = fn() -> Box<dyn Driver>;

/// 静态驱动注册条目
pub struct StaticDriverEntry {
    pub name: &'static str,
    pub factory: DriverFactory,
}

/// 动态驱动ABI版本
pub const DRIVER_API_VERSION: u16 = 1;