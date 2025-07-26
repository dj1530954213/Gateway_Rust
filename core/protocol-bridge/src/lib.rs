/*!
# Protocol Bridge Plugin System

协议桥接插件系统，支持OPC-UA Server和Modbus Slave桥接。
实现ABI v1接口，支持动态插件加载。

## MVP-3功能要求

- OPC-UA Server桥接
- Modbus Slave桥接  
- ABI v1插件接口
- 动态加载和热插拔
- 共享内存通信
- 插件沙箱隔离

## 架构设计

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Gateway Core  │    │ Protocol Bridge │    │    Plugins      │
│                 │◄──►│                 │◄──►│  ┌──────────┐   │
│  FrameBus       │    │  Runtime Engine │    │  │ OPC-UA   │   │
│  WAL            │    │  Plugin Manager │    │  │ Server   │   │
│  Drivers        │    │  ABI v1         │    │  └──────────┘   │
│                 │    │                 │    │  ┌──────────┐   │
│                 │    │                 │    │  │ Modbus   │   │
│                 │    │                 │    │  │ Slave    │   │
│                 │    │                 │    │  └──────────┘   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```
*/

pub mod bridge;
pub mod plugin;
pub mod runtime;
pub mod opcua;
pub mod modbus;
pub mod abi;
pub mod error;

pub use bridge::{ProtocolBridge, BridgeConfig, BridgeType};
pub use plugin::{Plugin, PluginManager, PluginMetadata, PluginState};
pub use runtime::{PluginRuntime, RuntimeConfig};
pub use opcua::{OpcUaBridge, OpcUaConfig};
pub use modbus::{ModbusBridge, ModbusConfig};
pub use abi::{ABIv1, PluginContext, DataExchange};
pub use error::BridgeError;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 协议桥接结果类型
pub type Result<T> = std::result::Result<T, BridgeError>;

/// 桥接管理器
pub struct BridgeManager {
    bridges: Arc<RwLock<HashMap<String, Arc<dyn ProtocolBridge + Send + Sync>>>>,
    plugin_manager: Arc<PluginManager>,
    runtime: Arc<PluginRuntime>,
}

impl BridgeManager {
    /// 创建新的桥接管理器
    pub fn new() -> Result<Self> {
        let plugin_manager = Arc::new(PluginManager::new()?);
        let runtime = Arc::new(PluginRuntime::new(RuntimeConfig::default())?);
        
        Ok(Self {
            bridges: Arc::new(RwLock::new(HashMap::new())),
            plugin_manager,
            runtime,
        })
    }

    /// 注册协议桥接
    pub async fn register_bridge<B>(&self, name: String, bridge: B) -> Result<()>
    where
        B: ProtocolBridge + Send + Sync + 'static,
    {
        let mut bridges = self.bridges.write().await;
        bridges.insert(name, Arc::new(bridge));
        Ok(())
    }

    /// 获取协议桥接
    pub async fn get_bridge(&self, name: &str) -> Option<Arc<dyn ProtocolBridge + Send + Sync>> {
        let bridges = self.bridges.read().await;
        bridges.get(name).cloned()
    }

    /// 启动所有桥接
    pub async fn start_all(&self) -> Result<()> {
        let bridges = self.bridges.read().await;
        for (name, bridge) in bridges.iter() {
            tracing::info!("Starting protocol bridge: {}", name);
            bridge.start().await?;
        }
        Ok(())
    }

    /// 停止所有桥接
    pub async fn stop_all(&self) -> Result<()> {
        let bridges = self.bridges.read().await;
        for (name, bridge) in bridges.iter() {
            tracing::info!("Stopping protocol bridge: {}", name);
            bridge.stop().await?;
        }
        Ok(())
    }

    /// 加载插件
    pub async fn load_plugin<P: AsRef<std::path::Path>>(&self, path: P) -> Result<String> {
        self.plugin_manager.load_plugin(path).await
    }

    /// 卸载插件
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        self.plugin_manager.unload_plugin(plugin_id).await
    }

    /// 获取插件管理器
    pub fn plugin_manager(&self) -> &Arc<PluginManager> {
        &self.plugin_manager
    }

    /// 获取运行时
    pub fn runtime(&self) -> &Arc<PluginRuntime> {
        &self.runtime
    }
}

impl Default for BridgeManager {
    fn default() -> Self {
        Self::new().expect("Failed to create BridgeManager")
    }
}

/// 初始化协议桥接系统
pub async fn init() -> Result<Arc<BridgeManager>> {
    let manager = Arc::new(BridgeManager::new()?);
    
    // 注册内置桥接
    manager.register_bridge(
        "opcua-server".to_string(),
        OpcUaBridge::new(OpcUaConfig::default())?,
    ).await?;
    
    manager.register_bridge(
        "modbus-slave".to_string(),
        ModbusBridge::new(ModbusConfig::default())?,
    ).await?;
    
    tracing::info!("Protocol bridge system initialized");
    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bridge_manager() {
        let manager = BridgeManager::new().unwrap();
        
        // 测试OPC-UA桥接注册
        let opcua_bridge = OpcUaBridge::new(OpcUaConfig::default()).unwrap();
        manager.register_bridge("test-opcua".to_string(), opcua_bridge).await.unwrap();
        
        // 测试桥接获取
        let bridge = manager.get_bridge("test-opcua").await;
        assert!(bridge.is_some());
    }

    #[tokio::test]
    async fn test_init() {
        let manager = init().await.unwrap();
        
        // 验证内置桥接已注册
        assert!(manager.get_bridge("opcua-server").await.is_some());
        assert!(manager.get_bridge("modbus-slave").await.is_some());
    }
}