//! driver.rs —— 驱动接口定义
//!
//! 定义所有驱动必须实现的核心trait
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::error::DriverResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 协议类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProtocolKind {
    ModbusTcp,
    OpcUa,
    Mqtt,
}

/// 驱动接口trait
#[async_trait]
pub trait Driver: Send + Sync {
    /// 获取驱动支持的协议类型
    fn protocol(&self) -> ProtocolKind;
    
    /// 获取驱动版本
    fn version(&self) -> &'static str;
    
    /// 获取驱动名称
    fn name(&self) -> &'static str;
    
    /// 获取驱动描述
    fn description(&self) -> &'static str {
        "Driver description not provided"
    }
    
    /// 初始化驱动
    async fn initialize(&self, config: serde_json::Value) -> DriverResult<()>;
    
    /// 关联设备到驱动
    async fn attach_device(&self, device_id: Uuid, config: serde_json::Value) -> DriverResult<()>;
    
    /// 解除设备关联
    async fn detach_device(&self, device_id: Uuid) -> DriverResult<()>;
    
    /// 读取设备数据点
    async fn read_tag(&self, device_id: Uuid, address: &str) -> DriverResult<serde_json::Value>;
    
    /// 写入设备数据点
    async fn write_tag(&self, device_id: Uuid, address: &str, value: serde_json::Value) -> DriverResult<()>;
    
    /// 启动驱动（开始数据采集）
    async fn start(&self) -> DriverResult<()>;
    
    /// 停止驱动
    async fn stop(&self) -> DriverResult<()>;
    
    /// 清理资源
    async fn cleanup(&self) -> DriverResult<()>;
}