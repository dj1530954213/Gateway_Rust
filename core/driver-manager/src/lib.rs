//! DriverManager - L1 驱动管理器
//! 
//! 统一管理静态、动态和WASM驱动的生命周期

pub mod driver;
pub mod manager;
pub mod loader;
pub mod supervisor;
pub mod registry;

pub use driver::{Driver, DriverMeta, DriverKind, DriverState, StaticDriverEntry};
pub use manager::DriverManager;
pub use registry::StaticDriverRegistry;
pub use loader::{DynDriverLoader, WasmDriverLoader};

use anyhow::Result;

/// 初始化DriverManager
pub fn init() -> Result<DriverManager> {
    DriverManager::new()
}