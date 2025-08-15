//! web-gw-api lib.rs —— API服务库导出
//!
//! 导出公共模块和函数供外部使用

pub mod bootstrap;
pub mod config;
pub mod database_pool;
pub mod dto;
pub mod error;
pub mod routes;
pub mod services;
pub mod middleware;
pub mod response;

// 重新导出常用类型和函数
pub use bootstrap::{init_state, AppState};
pub use config::{ApiConfig, DatabasePoolConfig, ConnectionRetryConfig};
pub use bootstrap::load_config;
pub use database_pool::{DatabasePoolManager, PoolStats};
pub use error::{ApiError, ApiResult};