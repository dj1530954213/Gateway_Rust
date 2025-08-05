//! pg-repo lib.rs —— PostgreSQL仓储层入口
//!
//! 提供统一的数据访问接口：
//! - DeviceRepo: 设备管理
//! - TagRepo: 点位管理
//! - AlertRepo: 报警规则与历史
//! - DriverRepo: 驱动注册表
//! - DriverConfigRepo: 驱动配置管理
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

pub mod models;
pub mod device_repo;
pub mod tag_repo;
pub mod alert_repo;
pub mod driver_repo;
pub mod driver_config_repo;
pub mod error;

pub use device_repo::{DeviceRepo, DeviceRepoImpl};
pub use tag_repo::{TagRepo, TagRepoImpl};
pub use alert_repo::{AlertRepo, AlertRepoImpl};
pub use driver_repo::{DriverRepo, DriverRepoImpl};
pub use driver_config_repo::{DriverConfigRepo, DriverConfigRepoImpl};
pub use error::{RepoError, RepoResult};
pub use models::*;