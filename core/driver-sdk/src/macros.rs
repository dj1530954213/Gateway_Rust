//! macros.rs —— 驱动声明宏
//!
//! 提供声明驱动的便利宏，自动生成必要的导出函数
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

/// 声明驱动宏
/// 
/// 自动生成驱动注册所需的导出函数和元数据
/// 
/// # 示例
/// 
/// ```rust
/// use driver_sdk::{declare_driver, Driver, ProtocolKind, DriverResult};
/// use async_trait::async_trait;
/// 
/// struct MyModbusDriver;
/// 
/// #[async_trait]
/// impl Driver for MyModbusDriver {
///     fn protocol(&self) -> ProtocolKind {
///         ProtocolKind::ModbusTcp
///     }
///     
///     fn version(&self) -> &'static str {
///         "1.0.0"
///     }
///     
///     fn name(&self) -> &'static str {
///         "My Modbus Driver"
///     }
///     
///     async fn initialize(&self, _config: serde_json::Value) -> DriverResult<()> {
///         Ok(())
///     }
///     
///     // ... 其他方法实现
/// }
/// 
/// declare_driver!(MyModbusDriver);
/// ```
#[macro_export]
macro_rules! declare_driver {
    ($driver_type:ty) => {
        use $crate::abi::{DriverMeta, DriverApiVersion, DRIVER_API_VERSION};
        
        /// 获取驱动元数据
        /// 
        /// 此函数会被驱动管理器调用以获取驱动信息
        #[no_mangle]
        pub extern "C" fn get_driver_meta() -> DriverMeta {
            let driver = <$driver_type>::default();
            DriverMeta {
                protocol: driver.protocol(),
                version: driver.version().to_string(),
                name: driver.name().to_string(),
                description: driver.description().to_string(),
                api_version: DRIVER_API_VERSION,
            }
        }
        
        /// 获取驱动API版本
        /// 
        /// 用于兼容性检查
        #[no_mangle]
        pub extern "C" fn get_api_version() -> DriverApiVersion {
            DRIVER_API_VERSION
        }
        
        /// 创建驱动实例
        /// 
        /// 返回驱动实例的指针，调用方负责生命周期管理
        #[no_mangle]
        pub extern "C" fn create_driver() -> *mut dyn $crate::Driver {
            let driver = Box::new(<$driver_type>::default());
            Box::into_raw(driver)
        }
        
        /// 销毁驱动实例
        /// 
        /// 释放由create_driver创建的实例
        #[no_mangle]
        pub unsafe extern "C" fn destroy_driver(driver: *mut dyn $crate::Driver) {
            if !driver.is_null() {
                let _ = Box::from_raw(driver);
            }
        }
        
        /// 驱动入口点检查
        /// 
        /// 用于验证驱动是否正确编译
        #[no_mangle]
        pub extern "C" fn driver_entry_point() -> bool {
            true
        }
    };
}

/// 简化版驱动声明宏，带默认实现
/// 
/// 为简单驱动提供默认的Default实现要求
#[macro_export]
macro_rules! declare_simple_driver {
    ($driver_type:ty, $protocol:expr, $version:expr, $name:expr) => {
        impl Default for $driver_type {
            fn default() -> Self {
                Self::new()
            }
        }
        
        impl $driver_type {
            pub fn new() -> Self {
                Self
            }
        }
        
        $crate::declare_driver!($driver_type);
    };
}