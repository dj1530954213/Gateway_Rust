//! 静态驱动注册表

use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::driver::DriverFactory;

/// 全局驱动注册表
static GLOBAL_REGISTRY: Lazy<Mutex<HashMap<String, DriverFactory>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// 静态驱动注册表
pub struct StaticDriverRegistry {
    drivers: HashMap<String, DriverFactory>,
}

impl StaticDriverRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            drivers: HashMap::new(),
        };
        
        // 从全局注册表复制驱动
        if let Ok(global_drivers) = GLOBAL_REGISTRY.lock() {
            registry.drivers = global_drivers.clone();
        }
        
        registry
    }

    pub fn get(&self, name: &str) -> Option<DriverFactory> {
        self.drivers.get(name).copied()
    }

    pub fn list(&self) -> Vec<String> {
        self.drivers.keys().cloned().collect()
    }
}

/// 手动注册驱动的函数
pub fn register_driver(name: &str, factory: DriverFactory) {
    if let Ok(mut global_drivers) = GLOBAL_REGISTRY.lock() {
        global_drivers.insert(name.to_string(), factory);
    }
}

/// 便捷宏：注册静态驱动
#[macro_export]
macro_rules! register_static_driver {
    ($name:expr, $factory:expr) => {
        paste::paste! {
            #[ctor::ctor]
            fn [<register_ $name:snake>]() {
                $crate::registry::register_driver($name, $factory);
            }
        }
    };
}

pub use register_static_driver;