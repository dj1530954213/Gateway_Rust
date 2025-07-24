//! 配置管理器
//! 
//! 实现三表YAML配置(endpoints.yml, drivers.yml, variables.yml)和热加载

pub mod schemas;
pub mod manager;

pub use schemas::*;
pub use manager::{ConfigManager, ConfigEvent};

/// 创建配置管理器
pub async fn create_manager<P: AsRef<std::path::Path>>(
    config_dir: P
) -> anyhow::Result<(ConfigManager, tokio::sync::mpsc::UnboundedReceiver<ConfigEvent>)> {
    ConfigManager::new(config_dir).await
}