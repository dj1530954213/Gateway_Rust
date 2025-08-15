//! services/mod.rs —— 业务服务层模块
//!
//! - frame_bus_bridge: FrameBus与WebSocket桥接服务
//! - history: InfluxDB历史数据查询服务
//! - protocol_mapper: 协议名称映射服务
//! - driver_config_monitor: 驱动配置监听和自动启动服务
//! - 其他业务服务将在后续添加
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

pub mod frame_bus_bridge;
pub mod history;
pub mod protocol_mapper;
pub mod driver_config_monitor;

pub use frame_bus_bridge::{FrameBusBridge, TelemetryPublisher, AlertPublisher};
pub use history::HistoryService;
pub use protocol_mapper::{ProtocolMapper, get_protocol_mapper};
pub use driver_config_monitor::{DriverConfigMonitor, init_driver_config_monitor, get_driver_config_monitor};