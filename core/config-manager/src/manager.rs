//! 配置管理器实现

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::fs;
use notify::{Watcher, RecommendedWatcher, Event, RecursiveMode};
use anyhow::Result;

use crate::schemas::{EndpointsConfig, DriversConfig, VariablesConfig};

/// 配置变更事件
#[derive(Debug, Clone)]
pub enum ConfigEvent {
    EndpointsChanged(EndpointsConfig),
    DriversChanged(DriversConfig),
    VariablesChanged(VariablesConfig),
}

/// 配置管理器
pub struct ConfigManager {
    config_dir: PathBuf,
    endpoints: Arc<RwLock<EndpointsConfig>>,
    drivers: Arc<RwLock<DriversConfig>>,
    variables: Arc<RwLock<VariablesConfig>>,
    event_tx: mpsc::UnboundedSender<ConfigEvent>,
    _watcher: Box<dyn notify::Watcher + Send + Sync>,
}

impl Clone for ConfigManager {
    fn clone(&self) -> Self {
        // 创建一个虚拟的watcher，因为实际的watcher不能被克隆
        // 这个克隆的manager将共享相同的配置数据，但不会有文件监听能力
        let dummy_watcher = notify::NullWatcher::new(
            |_: notify::Result<notify::Event>| {}, 
            notify::Config::default()
        ).unwrap();
        
        Self {
            config_dir: self.config_dir.clone(),
            endpoints: self.endpoints.clone(),
            drivers: self.drivers.clone(),
            variables: self.variables.clone(),
            event_tx: self.event_tx.clone(),
            _watcher: Box::new(dummy_watcher),
        }
    }
}

impl ConfigManager {
    /// 创建配置管理器
    pub async fn new<P: AsRef<Path>>(config_dir: P) -> Result<(Self, mpsc::UnboundedReceiver<ConfigEvent>)> {
        let config_dir = config_dir.as_ref().to_path_buf();
        
        // 确保配置目录存在
        fs::create_dir_all(&config_dir).await?;
        
        // 创建事件通道
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        // 初始加载配置
        let endpoints = Arc::new(RwLock::new(Self::load_endpoints(&config_dir).await?));
        let drivers = Arc::new(RwLock::new(Self::load_drivers(&config_dir).await?));
        let variables = Arc::new(RwLock::new(Self::load_variables(&config_dir).await?));
        
        // 设置文件监听
        let watcher = Self::setup_watcher(&config_dir, event_tx.clone(), 
            endpoints.clone(), drivers.clone(), variables.clone())?;
        
        Ok((Self {
            config_dir,
            endpoints,
            drivers,
            variables,
            event_tx,
            _watcher: Box::new(watcher),
        }, event_rx))
    }
    
    /// 获取端点配置
    pub async fn get_endpoints(&self) -> EndpointsConfig {
        self.endpoints.read().await.clone()
    }
    
    /// 获取驱动配置
    pub async fn get_drivers(&self) -> DriversConfig {
        self.drivers.read().await.clone()
    }
    
    /// 获取变量配置
    pub async fn get_variables(&self) -> VariablesConfig {
        self.variables.read().await.clone()
    }
    
    /// 重新加载所有配置
    pub async fn reload_all(&self) -> Result<()> {
        let endpoints = Self::load_endpoints(&self.config_dir).await?;
        let drivers = Self::load_drivers(&self.config_dir).await?;
        let variables = Self::load_variables(&self.config_dir).await?;
        
        *self.endpoints.write().await = endpoints.clone();
        *self.drivers.write().await = drivers.clone();
        *self.variables.write().await = variables.clone();
        
        // 发送变更事件
        let _ = self.event_tx.send(ConfigEvent::EndpointsChanged(endpoints));
        let _ = self.event_tx.send(ConfigEvent::DriversChanged(drivers));
        let _ = self.event_tx.send(ConfigEvent::VariablesChanged(variables));
        
        tracing::info!("All configurations reloaded");
        Ok(())
    }
    
    /// 加载端点配置
    async fn load_endpoints(config_dir: &Path) -> Result<EndpointsConfig> {
        let path = config_dir.join("endpoints.yml");
        
        if !path.exists() {
            // 创建默认配置文件
            let default_config = Self::default_endpoints_config();
            let content = serde_yaml::to_string(&default_config)?;
            fs::write(&path, content).await?;
            tracing::info!("Created default endpoints.yml");
            return Ok(default_config);
        }
        
        let content = fs::read_to_string(&path).await?;
        let config: EndpointsConfig = serde_yaml::from_str(&content)?;
        tracing::info!("Loaded endpoints.yml with {} endpoints", config.endpoints.len());
        Ok(config)
    }
    
    /// 加载驱动配置
    async fn load_drivers(config_dir: &Path) -> Result<DriversConfig> {
        let path = config_dir.join("drivers.yml");
        
        if !path.exists() {
            let default_config = Self::default_drivers_config();
            let content = serde_yaml::to_string(&default_config)?;
            fs::write(&path, content).await?;
            tracing::info!("Created default drivers.yml");
            return Ok(default_config);
        }
        
        let content = fs::read_to_string(&path).await?;
        let config: DriversConfig = serde_yaml::from_str(&content)?;
        tracing::info!("Loaded drivers.yml with {} drivers", config.drivers.len());
        Ok(config)
    }
    
    /// 加载变量配置
    async fn load_variables(config_dir: &Path) -> Result<VariablesConfig> {
        let path = config_dir.join("variables.yml");
        
        if !path.exists() {
            let default_config = Self::default_variables_config();
            let content = serde_yaml::to_string(&default_config)?;
            fs::write(&path, content).await?;
            tracing::info!("Created default variables.yml");
            return Ok(default_config);
        }
        
        let content = fs::read_to_string(&path).await?;
        let config: VariablesConfig = serde_yaml::from_str(&content)?;
        tracing::info!("Loaded variables.yml with {} variables", config.variables.len());
        Ok(config)
    }
    
    /// 设置文件监听器
    fn setup_watcher(
        config_dir: &Path,
        event_tx: mpsc::UnboundedSender<ConfigEvent>,
        endpoints: Arc<RwLock<EndpointsConfig>>,
        drivers: Arc<RwLock<DriversConfig>>,
        variables: Arc<RwLock<VariablesConfig>>,
    ) -> Result<RecommendedWatcher> {
        let config_dir_for_watcher = config_dir.to_path_buf();
        let config_dir_for_watch = config_dir.to_path_buf();
        
        // 创建文件变更事件通道
        let (file_tx, mut file_rx) = mpsc::unbounded_channel::<String>();
        
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if let Some(path) = event.paths.get(0) {
                    if let Some(filename) = path.file_name() {
                        let filename = filename.to_string_lossy().to_string();
                        let _ = file_tx.send(filename);
                    }
                }
            }
        })?;
        
        // 在单独的任务中处理文件变更
        tokio::spawn({
            let config_dir = config_dir_for_watcher.clone();
            let event_tx = event_tx.clone();
            let endpoints = endpoints.clone();
            let drivers = drivers.clone();
            let variables = variables.clone();
            
            async move {
                while let Some(filename) = file_rx.recv().await {
                    match filename.as_str() {
                        "endpoints.yml" => {
                            if let Ok(config) = Self::load_endpoints(&config_dir).await {
                                *endpoints.write().await = config.clone();
                                let _ = event_tx.send(ConfigEvent::EndpointsChanged(config));
                                tracing::info!("endpoints.yml reloaded");
                            }
                        }
                        "drivers.yml" => {
                            if let Ok(config) = Self::load_drivers(&config_dir).await {
                                *drivers.write().await = config.clone();
                                let _ = event_tx.send(ConfigEvent::DriversChanged(config));
                                tracing::info!("drivers.yml reloaded");
                            }
                        }
                        "variables.yml" => {
                            if let Ok(config) = Self::load_variables(&config_dir).await {
                                *variables.write().await = config.clone();
                                let _ = event_tx.send(ConfigEvent::VariablesChanged(config));
                                tracing::info!("variables.yml reloaded");
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
        
        watcher.watch(&config_dir_for_watch, RecursiveMode::NonRecursive)?;
        Ok(watcher)
    }
    
    /// 默认端点配置
    fn default_endpoints_config() -> EndpointsConfig {
        use std::collections::HashMap;
        use crate::schemas::{EndpointCfg, PoolCfg};
        
        let mut endpoints = HashMap::new();
        endpoints.insert("plc1".to_string(), EndpointCfg {
            url: "tcp://[REAL_PLC_IP]:502".to_string(),
            description: "PLC设备1".to_string(),
            timeout: std::time::Duration::from_secs(10),
            pool: PoolCfg::default(),
            tls: None,
            serial: None,
        });
        
        EndpointsConfig { endpoints }
    }
    
    /// 默认驱动配置
    fn default_drivers_config() -> DriversConfig {
        use std::collections::HashMap;
        use crate::schemas::DriverCfg;
        
        let mut drivers = HashMap::new();
        drivers.insert("modbus1".to_string(), DriverCfg {
            driver_type: "modbus-tcp".to_string(),
            endpoint: "plc1".to_string(),
            enabled: true,
            polling: std::time::Duration::from_secs(1),
            retry: 3,
            config: serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        });
        
        DriversConfig { drivers }
    }
    
    /// 默认变量配置
    fn default_variables_config() -> VariablesConfig {
        use std::collections::HashMap;
        use crate::schemas::{VariableCfg, DataType, Access};
        
        let mut variables = HashMap::new();
        variables.insert("plant.flow_rate".to_string(), VariableCfg {
            description: "流量计".to_string(),
            driver: "modbus1".to_string(),
            data_type: DataType::Uint16,
            address: "40001".to_string(),
            access: Access::Read,
            scale: Some("value / 10.0".to_string()),
            unit: "m³/h".to_string(),
            alarms: Vec::new(),
        });
        
        VariablesConfig { variables }
    }
}