//! 动态驱动加载器

use std::path::Path;
use anyhow::Result;

use crate::driver::Driver;

/// 动态驱动加载器（MVP-0暂不实现）
pub struct DynDriverLoader;

impl DynDriverLoader {
    pub fn new() -> Self {
        Self
    }

    /// 加载动态库驱动
    pub fn load_driver<P: AsRef<Path>>(&self, _path: P) -> Result<Box<dyn Driver>> {
        // MVP-0 阶段暂不实现动态加载
        Err(anyhow::anyhow!("Dynamic driver loading not implemented in MVP-0"))
    }

    /// 卸载动态库驱动
    pub fn unload_driver(&self, _driver_id: &str) -> Result<()> {
        Err(anyhow::anyhow!("Dynamic driver unloading not implemented in MVP-0"))
    }

    /// 验证驱动ABI版本
    pub fn verify_abi_version<P: AsRef<Path>>(&self, _path: P) -> Result<u16> {
        Err(anyhow::anyhow!("ABI verification not implemented in MVP-0"))
    }
}

/// WASM驱动加载器（MVP-0暂不实现）
pub struct WasmDriverLoader;

impl WasmDriverLoader {
    pub fn new() -> Self {
        Self
    }

    /// 加载WASM驱动
    pub fn load_driver(&self, _wasm_bytes: &[u8]) -> Result<Box<dyn Driver>> {
        Err(anyhow::anyhow!("WASM driver loading not implemented in MVP-0"))
    }

    /// 验证WASM模块
    pub fn verify_wasm(&self, _wasm_bytes: &[u8]) -> Result<()> {
        Err(anyhow::anyhow!("WASM verification not implemented in MVP-0"))
    }
}