/*!
# Plugin System

插件系统实现，支持动态加载和ABI v1接口
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{BridgeError, Result};
use crate::abi::ABIv1;

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件ID
    pub id: String,
    /// 插件名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 作者
    pub author: String,
    /// 描述
    pub description: String,
    /// ABI版本
    pub abi_version: String,
    /// 支持的协议
    pub protocols: Vec<String>,
    /// 依赖项
    pub dependencies: Vec<String>,
    /// 加载时间
    pub load_time: SystemTime,
    /// 文件路径
    pub file_path: PathBuf,
}

/// 插件状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    /// 未加载
    Unloaded,
    /// 正在加载
    Loading,
    /// 已加载
    Loaded,
    /// 正在初始化
    Initializing,
    /// 已初始化（可使用）
    Initialized,
    /// 正在运行
    Running,
    /// 已暂停
    Paused,
    /// 正在停止
    Stopping,
    /// 已停止
    Stopped,
    /// 正在卸载
    Unloading,
    /// 错误状态
    Error(String),
}

/// 插件实例
pub struct Plugin {
    /// 插件元数据
    pub metadata: PluginMetadata,
    /// 插件状态
    pub state: PluginState,
    /// 动态库句柄
    library: Arc<Library>,
    /// ABI接口
    abi: Arc<dyn ABIv1>,
    /// 插件配置
    config: HashMap<String, serde_json::Value>,
}

impl Plugin {
    /// 加载插件
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        tracing::info!("Loading plugin from: {:?}", path);

        // 加载动态库
        let library = unsafe {
            Library::new(path).map_err(|e| {
                BridgeError::PluginLoadFailed(format!("Failed to load library: {}", e))
            })?
        };

        // 获取插件入口点
        let bridge_entry: Symbol<extern "C" fn() -> *mut dyn ABIv1> = unsafe {
            library.get(b"bridge_entry").map_err(|e| {
                BridgeError::PluginLoadFailed(format!("bridge_entry symbol not found: {}", e))
            })?
        };

        // 调用入口点获取ABI接口
        let abi_ptr = bridge_entry();
        if abi_ptr.is_null() {
            return Err(BridgeError::PluginLoadFailed(
                "bridge_entry returned null pointer".to_string(),
            ));
        }

        let abi: Arc<dyn ABIv1> = unsafe { Arc::from_raw(abi_ptr) };

        // 获取插件元数据
        let metadata = Self::load_metadata(&abi, path)?;

        // 验证ABI版本
        if metadata.abi_version != "v1" {
            return Err(BridgeError::ABIMismatch {
                expected: "v1".to_string(),
                actual: metadata.abi_version.clone(),
            });
        }

        Ok(Self {
            metadata,
            state: PluginState::Loaded,
            library: Arc::new(library),
            abi,
            config: HashMap::new(),
        })
    }

    /// 加载插件元数据
    fn load_metadata(abi: &Arc<dyn ABIv1>, path: &Path) -> Result<PluginMetadata> {
        let info = abi.get_plugin_info();
        
        Ok(PluginMetadata {
            id: Uuid::new_v4().to_string(),
            name: info.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            version: info.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string(),
            author: info.get("author")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            description: info.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            abi_version: info.get("abi_version")
                .and_then(|v| v.as_str())
                .unwrap_or("v1")
                .to_string(),
            protocols: info.get("protocols")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default(),
            dependencies: info.get("dependencies")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default(),
            load_time: SystemTime::now(),
            file_path: path.to_path_buf(),
        })
    }

    /// 初始化插件
    pub fn initialize(&mut self, config: HashMap<String, serde_json::Value>) -> Result<()> {
        if self.state != PluginState::Loaded {
            return Err(BridgeError::BridgeState(
                format!("Plugin must be in Loaded state to initialize, current: {:?}", self.state)
            ));
        }

        self.state = PluginState::Initializing;
        self.config = config.clone();

        match self.abi.initialize(config) {
            Ok(_) => {
                self.state = PluginState::Initialized;
                tracing::info!("Plugin {} initialized successfully", self.metadata.name);
                Ok(())
            }
            Err(e) => {
                self.state = PluginState::Error(e.clone());
                Err(BridgeError::plugin(format!("Plugin initialization failed: {}", e)))
            }
        }
    }

    /// 启动插件
    pub fn start(&mut self) -> Result<()> {
        if self.state != PluginState::Initialized {
            return Err(BridgeError::BridgeState(
                format!("Plugin must be initialized to start, current: {:?}", self.state)
            ));
        }

        match self.abi.start() {
            Ok(_) => {
                self.state = PluginState::Running;
                tracing::info!("Plugin {} started successfully", self.metadata.name);
                Ok(())
            }
            Err(e) => {
                self.state = PluginState::Error(e.clone());
                Err(BridgeError::plugin(format!("Plugin start failed: {}", e)))
            }
        }
    }

    /// 停止插件
    pub fn stop(&mut self) -> Result<()> {
        if !matches!(self.state, PluginState::Running | PluginState::Paused) {
            return Ok(()); // 已经停止
        }

        self.state = PluginState::Stopping;

        match self.abi.stop() {
            Ok(_) => {
                self.state = PluginState::Stopped;
                tracing::info!("Plugin {} stopped successfully", self.metadata.name);
                Ok(())
            }
            Err(e) => {
                self.state = PluginState::Error(e.clone());
                Err(BridgeError::plugin(format!("Plugin stop failed: {}", e)))
            }
        }
    }

    /// 暂停插件
    pub fn pause(&mut self) -> Result<()> {
        if self.state != PluginState::Running {
            return Err(BridgeError::BridgeState(
                format!("Plugin must be running to pause, current: {:?}", self.state)
            ));
        }

        match self.abi.pause() {
            Ok(_) => {
                self.state = PluginState::Paused;
                tracing::info!("Plugin {} paused successfully", self.metadata.name);
                Ok(())
            }
            Err(e) => {
                self.state = PluginState::Error(e.clone());
                Err(BridgeError::plugin(format!("Plugin pause failed: {}", e)))
            }
        }
    }

    /// 恢复插件
    pub fn resume(&mut self) -> Result<()> {
        if self.state != PluginState::Paused {
            return Err(BridgeError::BridgeState(
                format!("Plugin must be paused to resume, current: {:?}", self.state)
            ));
        }

        match self.abi.resume() {
            Ok(_) => {
                self.state = PluginState::Running;
                tracing::info!("Plugin {} resumed successfully", self.metadata.name);
                Ok(())
            }
            Err(e) => {
                self.state = PluginState::Error(e.clone());
                Err(BridgeError::plugin(format!("Plugin resume failed: {}", e)))
            }
        }
    }

    /// 配置插件
    pub fn configure(&mut self, config: HashMap<String, serde_json::Value>) -> Result<()> {
        self.config.extend(config.clone());

        match self.abi.configure(config) {
            Ok(_) => {
                tracing::info!("Plugin {} configured successfully", self.metadata.name);
                Ok(())
            }
            Err(e) => {
                Err(BridgeError::plugin(format!("Plugin configuration failed: {}", e)))
            }
        }
    }

    /// 获取插件状态
    pub fn get_state(&self) -> PluginState {
        self.state.clone()
    }

    /// 获取插件统计信息
    pub fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        self.abi.get_stats()
    }

    /// 健康检查
    pub fn health_check(&self) -> bool {
        self.abi.health_check()
    }

    /// 获取ABI接口
    pub fn abi(&self) -> &Arc<dyn ABIv1> {
        &self.abi
    }
}

/// 插件管理器
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Arc<Mutex<Plugin>>>>>,
    plugin_directories: Vec<PathBuf>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Result<Self> {
        Ok(Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_directories: vec![
                PathBuf::from("./plugins"),
                PathBuf::from("/usr/local/lib/gateway/plugins"),
                PathBuf::from("C:\\Program Files\\Gateway\\plugins"),
            ],
        })
    }

    /// 添加插件目录
    pub fn add_plugin_directory<P: AsRef<Path>>(&mut self, path: P) {
        self.plugin_directories.push(path.as_ref().to_path_buf());
    }

    /// 加载插件
    pub async fn load_plugin<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let plugin = Plugin::load(path)?;
        let plugin_id = plugin.metadata.id.clone();
        
        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(plugin_id.clone(), Arc::new(Mutex::new(plugin)));
        
        tracing::info!("Plugin loaded with ID: {}", plugin_id);
        Ok(plugin_id)
    }

    /// 卸载插件
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        let plugin_arc = {
            let mut plugins = self.plugins.write().unwrap();
            plugins.remove(plugin_id)
                .ok_or_else(|| BridgeError::PluginNotFound(plugin_id.to_string()))?
        };

        let mut plugin = plugin_arc.lock().await;
        
        // 确保插件已停止
        if matches!(plugin.state, PluginState::Running | PluginState::Paused) {
            plugin.stop()?;
        }

        plugin.state = PluginState::Unloading;
        
        // 通知插件即将卸载
        let _ = plugin.abi.cleanup();
        
        plugin.state = PluginState::Unloaded;
        tracing::info!("Plugin {} unloaded", plugin_id);
        
        Ok(())
    }

    /// 获取插件
    pub async fn get_plugin(&self, plugin_id: &str) -> Option<Arc<Mutex<Plugin>>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(plugin_id).cloned()
    }

    /// 列出所有插件
    pub async fn list_plugins(&self) -> Vec<PluginMetadata> {
        let plugins = self.plugins.read().unwrap();
        let mut result = Vec::new();
        
        for plugin_arc in plugins.values() {
            if let Ok(plugin) = plugin_arc.try_lock() {
                result.push(plugin.metadata.clone());
            }
        }
        
        result
    }

    /// 发现并加载目录中的插件
    pub async fn discover_plugins(&self) -> Result<Vec<String>> {
        let mut loaded_plugins = Vec::new();
        
        for dir in &self.plugin_directories {
            if !dir.exists() {
                continue;
            }

            let entries = std::fs::read_dir(dir)
                .map_err(|e| BridgeError::Io(e))?;

            for entry in entries {
                let entry = entry.map_err(|e| BridgeError::Io(e))?;
                let path = entry.path();
                
                // 检查文件扩展名
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy();
                    if ext_str == "so" || ext_str == "dll" || ext_str == "dylib" {
                        match self.load_plugin(&path).await {
                            Ok(plugin_id) => {
                                loaded_plugins.push(plugin_id);
                                tracing::info!("Discovered and loaded plugin: {:?}", path);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to load plugin {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(loaded_plugins)
    }

    /// 启动所有插件
    pub async fn start_all_plugins(&self) -> Result<()> {
        let plugins = self.plugins.read().unwrap();
        
        for plugin_arc in plugins.values() {
            let mut plugin = plugin_arc.lock().await;
            if plugin.state == PluginState::Initialized {
                if let Err(e) = plugin.start() {
                    tracing::error!("Failed to start plugin {}: {}", plugin.metadata.name, e);
                }
            }
        }
        
        Ok(())
    }

    /// 停止所有插件
    pub async fn stop_all_plugins(&self) -> Result<()> {
        let plugins = self.plugins.read().unwrap();
        
        for plugin_arc in plugins.values() {
            let mut plugin = plugin_arc.lock().await;
            if matches!(plugin.state, PluginState::Running | PluginState::Paused) {
                if let Err(e) = plugin.stop() {
                    tracing::error!("Failed to stop plugin {}: {}", plugin.metadata.name, e);
                }
            }
        }
        
        Ok(())
    }

    /// 获取插件统计信息
    pub async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let plugins = self.plugins.read().unwrap();
        let total_plugins = plugins.len();
        let mut running_plugins = 0;
        let mut error_plugins = 0;
        
        for plugin_arc in plugins.values() {
            if let Ok(plugin) = plugin_arc.try_lock() {
                match plugin.state {
                    PluginState::Running => running_plugins += 1,
                    PluginState::Error(_) => error_plugins += 1,
                    _ => {}
                }
            }
        }
        
        let mut stats = HashMap::new();
        stats.insert("total_plugins".to_string(), serde_json::Value::Number(total_plugins.into()));
        stats.insert("running_plugins".to_string(), serde_json::Value::Number(running_plugins.into()));
        stats.insert("error_plugins".to_string(), serde_json::Value::Number(error_plugins.into()));
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_state() {
        let state = PluginState::Loaded;
        assert_eq!(state, PluginState::Loaded);
        
        let error_state = PluginState::Error("test error".to_string());
        assert!(matches!(error_state, PluginState::Error(_)));
    }

    #[tokio::test]
    async fn test_plugin_manager() {
        let manager = PluginManager::new().unwrap();
        let plugins = manager.list_plugins().await;
        assert_eq!(plugins.len(), 0);
    }

    #[test]
    fn test_plugin_metadata() {
        let metadata = PluginMetadata {
            id: "test-id".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: "Test Description".to_string(),
            abi_version: "v1".to_string(),
            protocols: vec!["test".to_string()],
            dependencies: vec![],
            load_time: SystemTime::now(),
            file_path: PathBuf::from("test.so"),
        };
        
        assert_eq!(metadata.name, "Test Plugin");
        assert_eq!(metadata.abi_version, "v1");
    }
}