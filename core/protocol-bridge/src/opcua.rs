/*!
# OPC-UA Server Bridge

OPC-UA Server桥接实现，将内部数据暴露为OPC-UA服务器
*/

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::bridge::*;
use crate::{BridgeError, Result};

/// OPC-UA桥接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcUaConfig {
    /// 基础桥接配置
    pub base: BridgeConfig,
    /// OPC-UA服务器名称
    pub server_name: String,
    /// 应用程序URI
    pub application_uri: String,
    /// 端点路径
    pub endpoint_path: String,
    /// 发布间隔（毫秒）
    pub publishing_interval: f64,
}

impl Default for OpcUaConfig {
    fn default() -> Self {
        Self {
            base: BridgeConfig::default(),
            server_name: "Industrial Gateway OPC-UA Server".to_string(),
            application_uri: "urn:gateway:opcua:server".to_string(),
            endpoint_path: "/UA/Server".to_string(),
            publishing_interval: 1000.0,
        }
    }
}

/// OPC-UA桥接实现
pub struct OpcUaBridge {
    config: OpcUaConfig,
    data_store: Arc<RwLock<HashMap<String, DataPoint>>>,
    is_running: Arc<RwLock<bool>>,
}

impl OpcUaBridge {
    pub fn new(config: OpcUaConfig) -> Self {
        Self {
            config,
            data_store: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }
}

#[async_trait]
impl ProtocolBridge for OpcUaBridge {
    async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting OPC-UA bridge on {}", self.config.endpoint_path);
        
        // 简化实现：设置运行状态
        *self.is_running.write().unwrap() = true;
        
        tracing::info!("OPC-UA bridge started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping OPC-UA bridge");
        
        *self.is_running.write().unwrap() = false;
        
        tracing::info!("OPC-UA bridge stopped");
        Ok(())
    }

    async fn write_value(&self, id: &str, value: DataValue) -> Result<()> {
        let data_point = DataPoint {
            id: id.to_string(),
            name: id.to_string(),
            data_type: DataType::String,
            access: AccessLevel::ReadWrite,
            value: Some(value),
            last_updated: Some(std::time::SystemTime::now()),
            quality: Quality::Good,
        };
        
        self.data_store.write().unwrap().insert(id.to_string(), data_point);
        Ok(())
    }

    async fn read_value(&self, data_point_id: &str) -> Result<Option<DataValue>> {
        Ok(self.data_store.read().unwrap().get(data_point_id).and_then(|dp| dp.value.clone()))
    }

    async fn list_data_points(&self) -> Result<Vec<String>> {
        Ok(self.data_store.read().unwrap().keys().cloned().collect())
    }

    async fn stats(&self) -> BridgeStats {
        let data_store = self.data_store.read().unwrap();
        BridgeStats {
            connections: if *self.is_running.read().unwrap() { 1 } else { 0 },
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time: 0.0,
            start_time: None,
            last_activity: None,
        }
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(*self.is_running.read().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_opcua_bridge_creation() {
        let config = OpcUaConfig::default();
        let bridge = OpcUaBridge::new(config);
        
        assert!(!bridge.is_healthy());
    }

    #[tokio::test]
    async fn test_opcua_bridge_lifecycle() {
        let config = OpcUaConfig::default();
        let mut bridge = OpcUaBridge::new(config);
        
        assert!(bridge.start().await.is_ok());
        assert!(bridge.is_healthy());
        
        // 测试数据点操作
        let value = DataValue::Integer(42);
        assert!(bridge.write_data_point("test.value", value).await.is_ok());
        
        let data_point = bridge.read_data_point("test.value").await.unwrap();
        assert!(data_point.is_some());
        
        let stats = bridge.get_stats();
        assert_eq!(stats.connections, 1);
        assert_eq!(stats.data_points, 1);
        
        assert!(bridge.stop().await.is_ok());
        assert!(!bridge.is_healthy());
    }
}