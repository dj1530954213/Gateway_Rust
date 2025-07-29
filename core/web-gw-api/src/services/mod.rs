//! services/mod.rs —— 业务服务层模块
//!
//! - frame_bus_bridge: FrameBus与WebSocket桥接服务
//! - history: InfluxDB历史数据查询服务
//! - 其他业务服务将在后续添加
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

pub mod frame_bus_bridge;
pub mod history;

pub use frame_bus_bridge::{FrameBusBridge, TelemetryPublisher, AlertPublisher};
pub use history::HistoryService;