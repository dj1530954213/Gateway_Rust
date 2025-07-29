//! dynamic.rs —— 动态驱动加载器（基于driver-sdk）
//!
//! 为新的driver-sdk提供动态驱动加载支持，与现有frame-bus系统集成
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::driver::{Driver as LegacyDriver, DriverMeta as LegacyDriverMeta, DriverKind, DriverState};
use async_trait::async_trait;
use dashmap::DashMap;
use driver_sdk::{
    abi::{DriverMeta, DriverStatus, DriverStats, DriverLoadResult},
    driver::ProtocolKind,
    Driver,
};
use libloading::{Library, Symbol};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// SDK驱动包装器，将新SDK驱动适配到现有系统
pub struct SdkDriverWrapper {
    sdk_driver: Box<dyn Driver>,
    meta: LegacyDriverMeta,
    loaded_library: Arc<Library>,
}

impl SdkDriverWrapper {
    /// 从动态库创建SDK驱动包装器
    pub unsafe fn from_library(library: Arc<Library>) -> anyhow::Result<Self> {
        // 获取元数据
        let get_meta: Symbol<unsafe extern "C" fn() -> DriverMeta> = library
            .get(b"get_driver_meta")
            .map_err(|e| anyhow::anyhow!("Failed to get get_driver_meta symbol: {}", e))?;
        
        let sdk_meta = get_meta();
        
        // 创建驱动实例
        let create_fn: Symbol<unsafe extern "C" fn() -> *mut dyn Driver> = library
            .get(b"create_driver")
            .map_err(|e| anyhow::anyhow!("Failed to get create_driver symbol: {}", e))?;
        
        let raw_driver = create_fn();
        if raw_driver.is_null() {
            return Err(anyhow::anyhow!("create_driver returned null"));
        }
        
        let sdk_driver = Box::from_raw(raw_driver);
        
        // 转换元数据格式
        let legacy_meta = LegacyDriverMeta {
            name: sdk_meta.name.clone(),
            kind: DriverKind::Dyn,
            version: sdk_meta.version.clone(),
            api_version: sdk_meta.api_version as u16,
            description: sdk_meta.description.clone(),
            features: vec![sdk_meta.protocol_name().to_string()],
        };
        
        Ok(Self {
            sdk_driver,
            meta: legacy_meta,
            loaded_library: library,
        })
    }
}

#[async_trait]
impl LegacyDriver for SdkDriverWrapper {
    fn meta(&self) -> LegacyDriverMeta {
        self.meta.clone()
    }

    async fn init(&mut self, cfg: &serde_json::Value) -> anyhow::Result<()> {
        self.sdk_driver.initialize(cfg.clone()).await
            .map_err(|e| anyhow::anyhow!("SDK driver initialization failed: {}", e))
    }

    async fn connect(&mut self, _pool: Arc<endpoint_kit::EndpointHandle>) -> anyhow::Result<()> {
        // SDK驱动使用不同的连接模式，这里启动它
        self.sdk_driver.start().await
            .map_err(|e| anyhow::anyhow!("SDK driver start failed: {}", e))
    }

    async fn read_loop(&mut self, _tx: frame_bus::FrameSender) -> anyhow::Result<()> {
        // SDK驱动的数据读取由其内部处理
        // 这里可以实现桥接逻辑，将SDK驱动的数据转发到frame-bus
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            // TODO: 实现数据桥接
        }
    }

    async fn write(&mut self, _cmd: frame_bus::CmdFrame) -> anyhow::Result<()> {
        // TODO: 将frame-bus命令转换为SDK驱动调用
        Err(anyhow::anyhow!("Write not implemented for SDK wrapper"))
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.sdk_driver.stop().await
            .map_err(|e| anyhow::anyhow!("SDK driver stop failed: {}", e))?;
        
        self.sdk_driver.cleanup().await
            .map_err(|e| anyhow::anyhow!("SDK driver cleanup failed: {}", e))
    }
}

/// 动态驱动加载器
pub struct DynamicDriverLoader {
    /// 已加载的驱动库（path -> library）
    loaded_libraries: Arc<DashMap<PathBuf, Arc<Library>>>,
    /// 驱动注册表（driver_id -> driver_info）
    driver_registry: Arc<DashMap<String, DynamicDriverInfo>>,
    /// 文件监控器
    watcher: Option<RecommendedWatcher>,
    /// 监控的驱动目录
    drivers_dir: PathBuf,
    /// 事件发送器
    event_tx: broadcast::Sender<DynamicDriverEvent>,
}

/// 动态驱动信息
#[derive(Debug, Clone)]
pub struct DynamicDriverInfo {
    pub driver_id: String,
    pub meta: DriverMeta,
    pub file_path: PathBuf,
    pub status: DriverStatus,
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    pub stats: DriverStats,
}

/// 动态驱动事件
#[derive(Debug, Clone)]
pub enum DynamicDriverEvent {
    /// 驱动加载成功
    DriverLoaded { driver_id: String },
    /// 驱动卸载
    DriverUnloaded { driver_id: String },
    /// 驱动热重载成功
    DriverReloaded { old_driver_id: String, new_driver_id: String },
    /// 文件变化检测到
    FileChanged { path: PathBuf },
    /// 加载失败
    LoadError { path: PathBuf, error: String },
}

impl DynamicDriverLoader {
    /// 创建新的动态驱动加载器
    pub fn new<P: AsRef<Path>>(drivers_dir: P) -> anyhow::Result<Self> {
        let drivers_dir = drivers_dir.as_ref().to_path_buf();
        
        if !drivers_dir.exists() {
            std::fs::create_dir_all(&drivers_dir)?;
        }
        
        let (event_tx, _) = broadcast::channel(1000);
        
        Ok(Self {
            loaded_libraries: Arc::new(DashMap::new()),
            driver_registry: Arc::new(DashMap::new()),
            watcher: None,
            drivers_dir,
            event_tx,
        })
    }

    /// 启动文件监控
    pub fn start_file_watcher(&mut self) -> anyhow::Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        let event_tx = self.event_tx.clone();
        let drivers_dir = self.drivers_dir.clone();
        
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Err(e) = tx.send(res) {
                    error!("Failed to send file event: {}", e);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;

        watcher.watch(&self.drivers_dir, RecursiveMode::Recursive)?;

        // 启动事件处理任务
        tokio::spawn(async move {
            while let Ok(event_result) = rx.recv() {
                match event_result {
                    Ok(event) => {
                        for path in event.paths {
                            if Self::is_driver_file(&path) {
                                match event.kind {
                                    EventKind::Create(_) | EventKind::Modify(_) => {
                                        let _ = event_tx.send(DynamicDriverEvent::FileChanged { path });
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("File watcher error: {}", e);
                    }
                }
            }
        });

        self.watcher = Some(watcher);
        info!("Started file watcher for: {}", self.drivers_dir.display());
        Ok(())
    }

    /// 从文件加载驱动
    pub async fn load_driver<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<String> {
        let path = path.as_ref().to_path_buf();
        
        info!("Loading dynamic driver from: {}", path.display());

        // 检查文件是否已加载
        if self.loaded_libraries.contains_key(&path) {
            return Err(anyhow::anyhow!("Driver already loaded: {}", path.display()));
        }

        // 加载动态库
        let library = unsafe {
            Library::new(&path)
                .map_err(|e| anyhow::anyhow!("Failed to load library: {}", e))?
        };

        // 验证驱动
        let sdk_meta = unsafe {
            let get_meta: Symbol<unsafe extern "C" fn() -> DriverMeta> = library
                .get(b"get_driver_meta")
                .map_err(|e| anyhow::anyhow!("Missing get_driver_meta symbol: {}", e))?;
            get_meta()
        };

        let driver_id = sdk_meta.unique_id();
        
        // 检查驱动ID是否已存在
        if self.driver_registry.contains_key(&driver_id) {
            return Err(anyhow::anyhow!("Driver ID already exists: {}", driver_id));
        }

        let library = Arc::new(library);
        
        // 注册驱动信息
        let driver_info = DynamicDriverInfo {
            driver_id: driver_id.clone(),
            meta: sdk_meta,
            file_path: path.clone(),
            status: DriverStatus::Loaded,
            loaded_at: chrono::Utc::now(),
            stats: DriverStats::default(),
        };

        self.loaded_libraries.insert(path, library.clone());
        self.driver_registry.insert(driver_id.clone(), driver_info);

        // 发送事件
        let _ = self.event_tx.send(DynamicDriverEvent::DriverLoaded { driver_id: driver_id.clone() });

        info!("Successfully loaded dynamic driver: {}", driver_id);
        Ok(driver_id)
    }

    /// 卸载驱动
    pub async fn unload_driver(&self, driver_id: &str) -> anyhow::Result<()> {
        info!("Unloading dynamic driver: {}", driver_id);

        let driver_info = self.driver_registry.remove(driver_id)
            .ok_or_else(|| anyhow::anyhow!("Driver not found: {}", driver_id))?;

        let (_, driver_info) = driver_info;
        self.loaded_libraries.remove(&driver_info.file_path);

        // 发送事件
        let _ = self.event_tx.send(DynamicDriverEvent::DriverUnloaded { driver_id: driver_id.to_string() });

        info!("Successfully unloaded dynamic driver: {}", driver_id);
        Ok(())
    }

    /// 热重载驱动
    /// 
    /// # Parameters
    /// * `driver_id` – 要重载的驱动ID
    /// 
    /// # Returns
    /// * `Ok(new_driver_id)` – 重载成功，返回新的驱动ID
    /// * `Err(error)` – 重载失败
    pub async fn reload_driver(&self, driver_id: &str) -> anyhow::Result<String> {
        info!("Hot reloading dynamic driver: {}", driver_id);

        // 获取原驱动信息
        let original_info = self.driver_registry.get(driver_id)
            .ok_or_else(|| anyhow::anyhow!("Driver not found: {}", driver_id))?
            .clone();

        let file_path = original_info.file_path.clone();

        // 先卸载旧驱动
        self.unload_driver(driver_id).await?;

        // 短暂等待确保旧库完全卸载
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // 重新加载驱动
        let new_driver_id = self.load_driver(&file_path).await?;

        // 发送热重载事件
        let _ = self.event_tx.send(DynamicDriverEvent::DriverReloaded { 
            old_driver_id: driver_id.to_string(),
            new_driver_id: new_driver_id.clone(),
        });

        info!("Successfully hot reloaded driver: {} -> {}", driver_id, new_driver_id);
        Ok(new_driver_id)
    }

    /// 热重载指定路径的驱动文件
    /// 
    /// # Parameters
    /// * `file_path` – 驱动文件路径
    /// 
    /// # Returns
    /// * `Ok(new_driver_id)` – 重载成功，返回新的驱动ID
    /// * `Err(error)` – 重载失败
    pub async fn reload_driver_by_path<P: AsRef<std::path::Path>>(&self, file_path: P) -> anyhow::Result<Option<String>> {
        let file_path = file_path.as_ref();

        // 查找使用该文件的驱动
        let driver_to_reload = self.driver_registry.iter()
            .find(|entry| entry.value().file_path == file_path)
            .map(|entry| entry.key().clone());

        if let Some(driver_id) = driver_to_reload {
            let new_driver_id = self.reload_driver(&driver_id).await?;
            Ok(Some(new_driver_id))
        } else {
            // 如果没有找到现有驱动，尝试加载新文件
            if file_path.exists() && Self::is_driver_file(file_path) {
                let new_driver_id = self.load_driver(file_path).await?;
                Ok(Some(new_driver_id))
            } else {
                Ok(None)
            }
        }
    }

    /// 设置热重载处理器
    /// 
    /// 这个方法会启动一个后台任务来处理文件变化事件并自动触发热重载
    pub fn enable_auto_reload(&self) -> anyhow::Result<()> {
        let mut event_rx = self.event_tx.subscribe();
        let self_clone = Self::clone_for_auto_reload(self)?;

        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                match event {
                    DynamicDriverEvent::FileChanged { path } => {
                        info!("Detected file change: {}", path.display());
                        
                        // 等待一小段时间确保文件写入完成
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                        // 尝试热重载
                        match self_clone.reload_driver_by_path(&path).await {
                            Ok(Some(new_driver_id)) => {
                                info!("Auto-reloaded driver: {}", new_driver_id);
                            }
                            Ok(None) => {
                                debug!("No driver to reload for path: {}", path.display());
                            }
                            Err(e) => {
                                error!("Auto-reload failed for {}: {}", path.display(), e);
                                let _ = self_clone.event_tx.send(DynamicDriverEvent::LoadError {
                                    path,
                                    error: e.to_string(),
                                });
                            }
                        }
                    }
                    _ => {
                        // 忽略其他事件
                    }
                }
            }
        });

        info!("Auto-reload enabled for dynamic drivers");
        Ok(())
    }

    /// 为自动重载创建克隆（仅包含必要字段）
    pub fn clone_for_auto_reload(&self) -> anyhow::Result<Self> {
        Ok(Self {
            loaded_libraries: self.loaded_libraries.clone(),
            driver_registry: self.driver_registry.clone(),
            watcher: None, // 不克隆watcher
            drivers_dir: self.drivers_dir.clone(),
            event_tx: self.event_tx.clone(),
        })
    }

    /// 创建SDK驱动包装器
    pub async fn create_driver_wrapper(&self, driver_id: &str) -> anyhow::Result<Box<dyn LegacyDriver>> {
        let driver_info = self.driver_registry.get(driver_id)
            .ok_or_else(|| anyhow::anyhow!("Driver not found: {}", driver_id))?;

        let library = self.loaded_libraries.get(&driver_info.file_path)
            .ok_or_else(|| anyhow::anyhow!("Library not loaded for driver: {}", driver_id))?;

        let wrapper = unsafe {
            SdkDriverWrapper::from_library(library.clone())?
        };

        Ok(Box::new(wrapper))
    }

    /// 获取驱动信息
    pub fn get_driver_info(&self, driver_id: &str) -> Option<DynamicDriverInfo> {
        self.driver_registry.get(driver_id).map(|info| info.clone())
    }

    /// 列出所有已加载的驱动
    pub fn list_drivers(&self) -> Vec<DynamicDriverInfo> {
        self.driver_registry.iter().map(|entry| entry.value().clone()).collect()
    }

    /// 扫描目录并加载所有驱动
    pub async fn scan_and_load_all(&self) -> anyhow::Result<Vec<String>> {
        let mut loaded_drivers = Vec::new();
        
        if !self.drivers_dir.exists() {
            return Ok(loaded_drivers);
        }

        let entries = std::fs::read_dir(&self.drivers_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && Self::is_driver_file(&path) {
                match self.load_driver(&path).await {
                    Ok(driver_id) => {
                        loaded_drivers.push(driver_id);
                    }
                    Err(e) => {
                        warn!("Failed to load driver from {}: {}", path.display(), e);
                        let _ = self.event_tx.send(DynamicDriverEvent::LoadError {
                            path,
                            error: e.to_string(),
                        });
                    }
                }
            }
        }

        info!("Scanned and loaded {} dynamic drivers", loaded_drivers.len());
        Ok(loaded_drivers)
    }

    /// 订阅事件
    pub fn subscribe_events(&self) -> broadcast::Receiver<DynamicDriverEvent> {
        self.event_tx.subscribe()
    }

    /// 判断是否为驱动文件
    fn is_driver_file(path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            matches!(extension.to_str(), Some("so") | Some("dll") | Some("dylib"))
        } else {
            false
        }
    }
}