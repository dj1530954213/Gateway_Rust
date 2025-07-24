//! 集成测试模块
//! 包含端到端、性能、故障恢复等集成测试

pub mod common;
pub mod e2e;
pub mod performance;
pub mod recovery;

// 重新导出公共测试工具
pub use crate::common::{mock_plc, test_env};