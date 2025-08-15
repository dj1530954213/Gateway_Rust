//! 驱动管理器

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use anyhow::Result;
use tracing::info;
use uuid::Uuid;

use crate::driver::{DriverState, DriverMeta};
use crate::registry::StaticDriverRegistry;
use crate::supervisor::DriverSupervisor;
use crate::dynamic::DynamicDriverLoader;
use crate::registry_manager::{RegistryManager, DriverQueryRequest, DriverQueryResponse, RegistryOverview};

/// 协议类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolKind {
    ModbusTcp,
    OpcUa,
    Mqtt,
}

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
    dynamic_loader: DynamicDriverLoader,
    registry_manager: RegistryManager,
}

impl DriverManager {
    pub fn new() -> Result<Self> {
        // 默认驱动目录
        let drivers_dir = std::env::var("DRIVERS_DIR")
            .unwrap_or_else(|_| "./drivers".to_string());
            
        let mut dynamic_loader = DynamicDriverLoader::new(&drivers_dir)?;
        dynamic_loader.start_file_watcher()?;
        
        let static_registry = StaticDriverRegistry::new();
        let registry_manager = RegistryManager::new()
            .with_dynamic_loader(dynamic_loader.clone_for_auto_reload()?);
        
        Ok(Self {
            drivers: Arc::new(RwLock::new(HashMap::new())),
            static_registry,
            dynamic_loader,
            registry_manager,
        })
    }
    
    pub fn with_drivers_dir<P: AsRef<std::path::Path>>(drivers_dir: P) -> Result<Self> {
        let mut dynamic_loader = DynamicDriverLoader::new(drivers_dir)?;
        dynamic_loader.start_file_watcher()?;
        
        let static_registry = StaticDriverRegistry::new();
        let registry_manager = RegistryManager::new()
            .with_dynamic_loader(dynamic_loader.clone_for_auto_reload()?);
        
        Ok(Self {
            drivers: Arc::new(RwLock::new(HashMap::new())),
            static_registry,
            dynamic_loader,
            registry_manager,
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

    /// 加载动态驱动（基于driver-sdk）
    pub async fn load_dynamic_driver<P: AsRef<std::path::Path>>(
        &self,
        file_path: P,
        driver_id: String,
        _config: serde_json::Value,
    ) -> Result<()> {
        // 先加载到动态加载器
        let sdk_driver_id = self.dynamic_loader.load_driver(file_path).await?;
        
        // 创建包装器
        let driver_wrapper = self.dynamic_loader.create_driver_wrapper(&sdk_driver_id).await?;
        let meta = driver_wrapper.meta();
        
        // 创建supervisor并包装为实例
        let supervisor = DriverSupervisor::new(driver_id.clone(), driver_wrapper);
        let instance = DriverInstance {
            meta,
            state: DriverState::Init,
            supervisor,
            task_handle: None,
        };

        let mut drivers = self.drivers.write().await;
        drivers.insert(driver_id.clone(), instance);

        tracing::info!("Loaded dynamic driver: {} (SDK ID: {})", driver_id, sdk_driver_id);
        Ok(())
    }

    /// 扫描并加载所有动态驱动
    pub async fn scan_and_load_dynamic_drivers(&self) -> Result<Vec<String>> {
        let loaded_drivers = self.dynamic_loader.scan_and_load_all().await?;
        
        for sdk_driver_id in &loaded_drivers {
            // 为每个SDK驱动创建一个管理器实例
            let driver_id = format!("dyn_{}", sdk_driver_id);
            
            if let Err(e) = self.load_dynamic_driver_from_sdk(&sdk_driver_id, driver_id.clone()).await {
                tracing::warn!("Failed to create manager instance for SDK driver {}: {}", sdk_driver_id, e);
            }
        }
        
        Ok(loaded_drivers)
    }

    /// 从已加载的SDK驱动创建管理器实例
    async fn load_dynamic_driver_from_sdk(&self, sdk_driver_id: &str, manager_driver_id: String) -> Result<()> {
        let driver_wrapper = self.dynamic_loader.create_driver_wrapper(sdk_driver_id).await?;
        let meta = driver_wrapper.meta();
        
        let supervisor = DriverSupervisor::new(manager_driver_id.clone(), driver_wrapper);
        let instance = DriverInstance {
            meta,
            state: DriverState::Init,
            supervisor,
            task_handle: None,
        };

        let mut drivers = self.drivers.write().await;
        drivers.insert(manager_driver_id, instance);
        
        Ok(())
    }

    /// 卸载动态驱动
    pub async fn unload_dynamic_driver(&self, driver_id: &str) -> Result<()> {
        // 先停止管理器中的实例
        self.stop_driver(driver_id).await?;
        
        // 从管理器移除
        {
            let mut drivers = self.drivers.write().await;
            drivers.remove(driver_id);
        }
        
        // 如果需要，也从动态加载器中卸载
        // 注意：可能有多个管理器实例共享同一个SDK驱动
        
        tracing::info!("Unloaded dynamic driver: {}", driver_id);
        Ok(())
    }

    /// 热重载动态驱动
    /// 
    /// # Parameters
    /// * `driver_id` – 管理器中的驱动ID
    /// 
    /// # Returns
    /// * `Ok(new_driver_id)` – 重载成功，返回新的驱动ID
    /// * `Err(error)` – 重载失败
    pub async fn reload_dynamic_driver(&self, driver_id: &str) -> Result<String> {
        info!("Hot reloading dynamic driver in manager: {}", driver_id);

        // 先停止管理器中的实例
        self.stop_driver(driver_id).await?;

        // 找到对应的SDK驱动ID（通过命名约定）
        let sdk_driver_id = if driver_id.starts_with("dyn_") {
            driver_id.strip_prefix("dyn_").unwrap()
        } else {
            driver_id
        };

        // 在动态加载器中热重载
        let new_sdk_driver_id = self.dynamic_loader.reload_driver(sdk_driver_id).await
            .map_err(|e| anyhow::anyhow!("Dynamic loader reload failed: {}", e))?;

        // 创建新的管理器实例
        let new_manager_driver_id = format!("dyn_{}", new_sdk_driver_id);
        self.load_dynamic_driver_from_sdk(&new_sdk_driver_id, new_manager_driver_id.clone()).await?;

        // 移除旧实例
        {
            let mut drivers = self.drivers.write().await;
            drivers.remove(driver_id);
        }

        tracing::info!("Successfully hot reloaded driver: {} -> {}", driver_id, new_manager_driver_id);
        Ok(new_manager_driver_id)
    }

    /// 启用自动热重载
    /// 
    /// 当检测到驱动文件变化时，自动重载相关驱动
    pub fn enable_auto_reload(&self) -> Result<()> {
        // 启用动态加载器的自动重载
        self.dynamic_loader.enable_auto_reload()
            .map_err(|e| anyhow::anyhow!("Failed to enable auto reload: {}", e))?;

        // 监听动态加载器事件并同步到管理器
        let mut event_rx = self.dynamic_loader.subscribe_events();
        let _drivers = self.drivers.clone();

        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                match event {
                    crate::dynamic::DynamicDriverEvent::DriverReloaded { old_driver_id, new_driver_id } => {
                        // 同步更新管理器中的驱动实例
                        let old_manager_id = format!("dyn_{}", old_driver_id);
                        let new_manager_id = format!("dyn_{}", new_driver_id);
                        
                        // TODO: 这里需要更复杂的逻辑来处理管理器实例的更新
                        // 目前只是记录日志
                        info!("SDK driver reloaded: {} -> {}, manager instances: {} -> {}", 
                              old_driver_id, new_driver_id, old_manager_id, new_manager_id);
                    }
                    _ => {
                        // 处理其他事件
                    }
                }
            }
        });

        info!("Auto-reload enabled for driver manager");
        Ok(())
    }

    /// 查询驱动注册表
    /// 
    /// # Parameters
    /// * `request` – 查询请求
    /// 
    /// # Returns
    /// 分页查询结果
    pub fn query_registry(&self, request: &DriverQueryRequest) -> DriverQueryResponse {
        self.registry_manager.query_drivers(request)
    }

    /// 获取注册表概览
    pub fn get_registry_overview(&self) -> RegistryOverview {
        self.registry_manager.get_overview()
    }

    /// 搜索驱动
    /// 
    /// # Parameters
    /// * `query` – 搜索关键词
    /// 
    /// # Returns
    /// 匹配的驱动列表
    pub fn search_drivers(&self, query: &str) -> Vec<crate::registry_manager::UnifiedDriverEntry> {
        self.registry_manager.search_drivers(query)
    }

    /// 获取特定驱动详情
    /// 
    /// # Parameters
    /// * `driver_id` – 驱动ID
    /// 
    /// # Returns
    /// 驱动详情（如果存在）
    pub fn get_driver_details(&self, driver_id: &str) -> Option<crate::registry_manager::UnifiedDriverEntry> {
        self.registry_manager.get_driver_details(driver_id)
    }

    /// 获取动态驱动加载器
    pub fn dynamic_loader(&self) -> &DynamicDriverLoader {
        &self.dynamic_loader
    }

    /// 获取注册表管理器
    pub fn registry_manager(&self) -> &RegistryManager {
        &self.registry_manager
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

    /// 注册设备到驱动管理器
    /// 
    /// # Parameters
    /// * `protocol` – 设备使用的协议类型
    /// * `device_id` – 设备ID
    /// 
    /// # Returns
    /// * `Ok(())` – 注册成功
    /// * `Err(error)` – 注册失败
    pub async fn register_device(&self, protocol: ProtocolKind, device_id: Uuid) -> Result<()> {
        info!("Registering device {} with protocol {:?}", device_id, protocol);
        
        // 简化实现：查找名称中包含协议名的驱动
        let protocol_name = match protocol {
            ProtocolKind::ModbusTcp => "modbus",
            ProtocolKind::OpcUa => "opc", 
            ProtocolKind::Mqtt => "mqtt",
        };
        
        // 从注册表中查找名称匹配协议的驱动
        let drivers = self.drivers.read().await;
        let suitable_driver = drivers.iter().find(|(_, instance)| {
            instance.meta.name.to_lowercase().contains(protocol_name)
        });
        
        if let Some((driver_id, _)) = suitable_driver {
            info!("Found suitable driver {} for device {} with protocol {:?}", 
                  driver_id, device_id, protocol);
            // 这里可以添加更具体的注册逻辑，比如通知驱动有新设备等
            Ok(())
        } else {
            info!("No specific driver found for protocol {:?}, device {} registered successfully", 
                  protocol, device_id);
            // 即使没有找到特定驱动，也成功注册，不阻塞操作
            Ok(())
        }
    }

    /// 从驱动管理器中解除设备注册
    /// 
    /// # Parameters
    /// * `device_id` – 要解除注册的设备ID
    /// 
    /// # Returns
    /// * `Ok(())` – 解除注册成功
    /// * `Err(error)` – 解除注册失败
    pub async fn detach_device(&self, device_id: Uuid) -> Result<()> {
        info!("Detaching device {} from driver manager", device_id);
        
        // 通知所有驱动解除设备关联
        let drivers = self.drivers.read().await;
        for (driver_id, _instance) in drivers.iter() {
            // 这里可以添加更具体的解除注册逻辑
            info!("Detaching device {} from driver {}", device_id, driver_id);
        }
        
        Ok(())
    }
}