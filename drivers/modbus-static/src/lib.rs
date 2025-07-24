//! Modbus-TCP静态驱动
//! 
//! 基于tokio-modbus实现的高性能Modbus-TCP驱动

pub mod driver;
pub mod config;
pub mod codec;
pub mod metrics;

pub use driver::ModbusDriver;
pub use config::ModbusCfg;

use driver_manager::{DriverMeta, DriverKind};

// TODO: 重新实现驱动注册机制（MVP-0阶段暂时禁用）
// inventory::submit! {
//     StaticDriverEntry {
//         name: "modbus-tcp",
//         factory: || Box::new(ModbusDriver::new()),
//     }
// }

/// 获取驱动元信息  
pub fn meta() -> DriverMeta {
    DriverMeta {
        name: "modbus-tcp".to_string(),
        kind: DriverKind::Static,
        version: "0.1.0".to_string(),
        api_version: 1,
        description: "Static Modbus-TCP driver based on tokio-modbus".to_string(),
        features: vec!["read".to_string()], // MVP-0 only read
    }
}