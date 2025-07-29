//! driver-sdk lib.rs —— 第三方驱动开发SDK
//!
//! 提供Driver trait定义和辅助宏：
//! - Driver trait: 驱动接口规范
//! - declare_driver!: 驱动声明宏
//! - ABI stability: 接口稳定性保证
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

pub mod driver;
pub mod abi;
pub mod macros;
pub mod error;

pub use driver::Driver;
pub use abi::{DriverMeta, DriverApiVersion};
pub use error::{DriverError, DriverResult};

// Re-export common dependencies for driver developers
pub use async_trait;
pub use serde;
pub use serde_json;
pub use uuid;
pub use anyhow;