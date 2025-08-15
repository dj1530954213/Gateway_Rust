//! Modbus-TCP静态驱动
//! 
//! 基于tokio-modbus实现的高性能Modbus-TCP驱动

pub mod driver;
pub mod config;
pub mod codec;
pub mod metrics;

pub use driver::ModbusDriver;
pub use config::ModbusCfg;

use driver_manager::{DriverMeta, DriverKind, Driver, register_static_driver};

/// 创建Modbus驱动实例的工厂函数
fn create_modbus_driver() -> Box<dyn Driver> {
    Box::new(ModbusDriver::new())
}

// 注册Modbus TCP驱动到静态驱动注册表
register_static_driver!("modbus-tcp", create_modbus_driver);

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