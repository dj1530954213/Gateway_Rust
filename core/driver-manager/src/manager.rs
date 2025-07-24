//! 驱动管理器

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use anyhow::Result;

use crate::driver::{DriverState, DriverMeta};
use crate::registry::StaticDriverRegistry;
use crate::supervisor::DriverSupervisor;

/// 驱动实例包装器
pub struct DriverInstance {
    pub meta: DriverMeta,
    pub state: DriverState,
    pub supervisor: DriverSupervisor,
    pub task_handle: Option<JoinHandle<()>>,
}

/// 驱动管理器
pub struct DriverManager {
    drivers: Arc<RwLock<HashMap<String, DriverInstance>>>,
    static_registry: StaticDriverRegistry,
}

impl DriverManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            drivers: Arc::new(RwLock::new(HashMap::new())),
            static_registry: StaticDriverRegistry::new(),
        })
    }

    /// 加载静态驱动
    pub async fn load_static_driver(
        &self,
        driver_id: String,
        driver_name: &str,
        config: serde_json::Value,
    ) -> Result<()> {
        let factory = self.static_registry.get(driver_name)
            .ok_or_else(|| anyhow::anyhow!("Static driver '{}' not found", driver_name))?;

        let mut driver = factory();
        let meta = driver.meta();
        
        // 初始化驱动
        driver.init(&config).await?;
        
        let supervisor = DriverSupervisor::new(driver_id.clone(), driver);
        let instance = DriverInstance {
            meta,
            state: DriverState::Init,
            supervisor,
            task_handle: None,
        };

        let mut drivers = self.drivers.write().await;
        drivers.insert(driver_id, instance);

        Ok(())
    }

    /// 启动驱动
    pub async fn start_driver(&self, driver_id: &str) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        let instance = drivers.get_mut(driver_id)
            .ok_or_else(|| anyhow::anyhow!("Driver '{}' not found", driver_id))?;

        if instance.task_handle.is_some() {
            return Err(anyhow::anyhow!("Driver '{}' already started", driver_id));
        }

        let supervisor = instance.supervisor.clone();
        let handle = tokio::spawn(async move {
            supervisor.run().await;
        });

        instance.task_handle = Some(handle);
        instance.state = DriverState::Active;

        tracing::info!("Started driver: {}", driver_id);
        Ok(())
    }

    /// 停止驱动
    pub async fn stop_driver(&self, driver_id: &str) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        let instance = drivers.get_mut(driver_id)
            .ok_or_else(|| anyhow::anyhow!("Driver '{}' not found", driver_id))?;

        if let Some(handle) = instance.task_handle.take() {
            // 先通知supervisor关闭
            instance.supervisor.shutdown().await;
            
            // 等待任务完成，而不是强制中止
            let _ = tokio::time::timeout(
                std::time::Duration::from_millis(20),
                handle
            ).await;
            
            instance.state = DriverState::Shutdown;
            tracing::info!("Stopped driver: {}", driver_id);
        }

        Ok(())
    }

    /// 获取驱动状态
    pub async fn get_driver_state(&self, driver_id: &str) -> Option<DriverState> {
        let drivers = self.drivers.read().await;
        drivers.get(driver_id).map(|instance| instance.state.clone())
    }

    /// 列出所有驱动
    pub async fn list_drivers(&self) -> Vec<(String, DriverMeta, DriverState)> {
        let drivers = self.drivers.read().await;
        drivers
            .iter()
            .map(|(id, instance)| (id.clone(), instance.meta.clone(), instance.state.clone()))
            .collect()
    }

    /// 停止所有驱动
    pub async fn shutdown(&self) -> Result<()> {
        let driver_ids: Vec<String> = {
            let drivers = self.drivers.read().await;
            drivers.keys().cloned().collect()
        };

        for driver_id in driver_ids {
            if let Err(e) = self.stop_driver(&driver_id).await {
                tracing::error!("Failed to stop driver {}: {}", driver_id, e);
            }
        }

        Ok(())
    }
}