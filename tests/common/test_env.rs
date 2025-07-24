//! 测试环境管理器，提供完整的测试基础设施

use std::path::PathBuf;
use tempfile::TempDir;
use anyhow::Result;
use tokio::time::Duration;

use crate::common::mock_plc::MockPLCServer;
use config_manager::{EndpointsConfig, DriversConfig, VariablesConfig};

/// 测试环境管理器
pub struct TestEnvironment {
    temp_dir: TempDir,
    mock_plc: Option<MockPLCServer>,
}

impl TestEnvironment {
    /// 创建新的测试环境
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        
        Ok(Self {
            temp_dir,
            mock_plc: None,
        })
    }
    
    /// 获取临时目录路径
    pub fn temp_dir(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
    
    /// 启动Mock PLC服务器
    pub async fn start_mock_plc(&mut self) -> Result<&MockPLCServer> {
        let plc = MockPLCServer::new().await?;
        
        // 设置一些默认测试数据
        plc.set_holding_register(40001, 1000).await; // 流量计 100.0 m³/h
        plc.set_holding_register(40002, 500).await;  // 压力计 5.0 bar
        plc.set_holding_register(40003, 250).await;  // 温度计 25.0 °C
        plc.set_coil(1, true).await;                 // 水泵运行
        
        self.mock_plc = Some(plc);
        Ok(self.mock_plc.as_ref().unwrap())
    }
    
    /// 获取Mock PLC引用
    pub fn mock_plc(&self) -> Option<&MockPLCServer> {
        self.mock_plc.as_ref()
    }
    
    /// 创建测试配置目录和文件
    pub async fn setup_test_configs(&self) -> Result<PathBuf> {
        let config_dir = self.temp_dir.path().join("config");
        tokio::fs::create_dir_all(&config_dir).await?;
        
        // 创建endpoints.yml
        let endpoints_config = if let Some(plc) = &self.mock_plc {
            self.create_endpoints_config_with_plc(plc)
        } else {
            self.create_default_endpoints_config()
        };
        
        let endpoints_yaml = serde_yaml::to_string(&endpoints_config)?;
        tokio::fs::write(config_dir.join("endpoints.yml"), endpoints_yaml).await?;
        
        // 创建drivers.yml
        let drivers_config = self.create_drivers_config();
        let drivers_yaml = serde_yaml::to_string(&drivers_config)?;
        tokio::fs::write(config_dir.join("drivers.yml"), drivers_yaml).await?;
        
        // 创建variables.yml
        let variables_config = self.create_variables_config();
        let variables_yaml = serde_yaml::to_string(&variables_config)?;
        tokio::fs::write(config_dir.join("variables.yml"), variables_yaml).await?;
        
        Ok(config_dir)
    }
    
    /// 创建包含Mock PLC的端点配置
    fn create_endpoints_config_with_plc(&self, plc: &MockPLCServer) -> EndpointsConfig {
        use std::collections::HashMap;
        use config_manager::{EndpointCfg, PoolCfg};
        
        let mut endpoints = HashMap::new();
        endpoints.insert("test_plc".to_string(), EndpointCfg {
            url: plc.endpoint_url(),
            description: "测试PLC模拟器".to_string(),
            timeout: Duration::from_secs(5),
            pool: PoolCfg {
                min_connections: 1,
                max_connections: 2,
                idle_timeout: Duration::from_secs(30),
                max_lifetime: Duration::from_secs(300),
            },
            tls: None,
            serial: None,
        });
        
        EndpointsConfig { endpoints }
    }
    
    /// 创建默认端点配置
    fn create_default_endpoints_config(&self) -> EndpointsConfig {
        use std::collections::HashMap;
        use config_manager::{EndpointCfg, PoolCfg};
        
        let mut endpoints = HashMap::new();
        endpoints.insert("test_plc".to_string(), EndpointCfg {
            url: "tcp://127.0.0.1:5020".to_string(),
            description: "测试端点".to_string(),
            timeout: Duration::from_secs(5),
            pool: PoolCfg::default(),
            tls: None,
            serial: None,
        });
        
        EndpointsConfig { endpoints }
    }
    
    /// 创建驱动配置
    fn create_drivers_config(&self) -> DriversConfig {
        use std::collections::HashMap;
        use config_manager::DriverCfg;
        
        let mut drivers = HashMap::new();
        drivers.insert("test_modbus".to_string(), DriverCfg {
            driver_type: "modbus-tcp".to_string(),
            endpoint: "test_plc".to_string(),
            enabled: true,
            polling: Duration::from_millis(100), // 快速轮询用于测试
            retry: 1,
            config: serde_yaml::Value::Mapping({
                let mut map = serde_yaml::Mapping::new();
                map.insert(
                    serde_yaml::Value::String("unit_id".to_string()),
                    serde_yaml::Value::Number(serde_yaml::Number::from(1))
                );
                map.insert(
                    serde_yaml::Value::String("max_regs_per_req".to_string()),
                    serde_yaml::Value::Number(serde_yaml::Number::from(10))
                );
                map.insert(
                    serde_yaml::Value::String("endian".to_string()),
                    serde_yaml::Value::String("big".to_string())
                );
                map
            }),
        });
        
        DriversConfig { drivers }
    }
    
    /// 创建变量配置
    fn create_variables_config(&self) -> VariablesConfig {
        use std::collections::HashMap;
        use config_manager::{VariableCfg, DataType, Access};
        
        let mut variables = HashMap::new();
        
        // 流量计
        variables.insert("test.flow_m3h".to_string(), VariableCfg {
            description: "测试流量计".to_string(),
            driver: "test_modbus".to_string(),
            data_type: DataType::Uint16,
            address: "40001".to_string(),
            access: Access::Read,
            scale: Some("value / 10.0".to_string()),
            unit: "m³/h".to_string(),
            alarms: Vec::new(),
        });
        
        // 压力计
        variables.insert("test.pressure_bar".to_string(), VariableCfg {
            description: "测试压力计".to_string(),
            driver: "test_modbus".to_string(),
            data_type: DataType::Uint16,
            address: "40002".to_string(),
            access: Access::Read,
            scale: Some("value / 100.0".to_string()),
            unit: "bar".to_string(),
            alarms: Vec::new(),
        });
        
        // 温度计
        variables.insert("test.temperature_c".to_string(), VariableCfg {
            description: "测试温度计".to_string(),
            driver: "test_modbus".to_string(),
            data_type: DataType::Int16,
            address: "40003".to_string(),
            access: Access::Read,
            scale: Some("value / 10.0".to_string()),
            unit: "°C".to_string(),
            alarms: Vec::new(),
        });
        
        // 水泵状态
        variables.insert("test.pump_running".to_string(), VariableCfg {
            description: "测试水泵状态".to_string(),
            driver: "test_modbus".to_string(),
            data_type: DataType::Bool,
            address: "1".to_string(),
            access: Access::ReadWrite,
            scale: None,
            unit: "".to_string(),
            alarms: Vec::new(),
        });
        
        VariablesConfig { variables }
    }
    
    /// 等待文件系统同步
    pub async fn wait_for_fs_sync(&self) {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// 测试数据生成器
pub struct TestDataGenerator {
    counter: u64,
}

impl TestDataGenerator {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
    
    /// 生成测试DataFrame
    pub fn next_data_frame(&mut self, tag: &str) -> frame_bus::DataFrame {
        self.counter += 1;
        frame_bus::DataFrame::new(
            &format!("{}_{:06}", tag, self.counter),
            frame_bus::Value::int(self.counter as i64)
        )
    }
    
    /// 生成批量测试数据
    pub fn generate_batch(&mut self, prefix: &str, count: usize) -> Vec<frame_bus::DataFrame> {
        (0..count)
            .map(|i| self.next_data_frame(&format!("{}.{:04}", prefix, i)))
            .collect()
    }
    
    /// 生成性能测试数据
    pub fn generate_performance_data(&mut self, point_count: usize) -> Vec<frame_bus::DataFrame> {
        (0..point_count)
            .map(|i| {
                self.counter += 1;
                frame_bus::DataFrame::new(
                    &format!("perf.point.{:06}", i),
                    frame_bus::Value::float(self.counter as f64 / 100.0)
                )
            })
            .collect()
    }
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_environment_setup() {
        let mut env = TestEnvironment::new().unwrap();
        
        // 启动Mock PLC
        let plc = env.start_mock_plc().await.unwrap();
        assert!(plc.get_holding_register(40001).await.is_some());
        
        // 创建配置文件
        let config_dir = env.setup_test_configs().await.unwrap();
        assert!(config_dir.join("endpoints.yml").exists());
        assert!(config_dir.join("drivers.yml").exists());
        assert!(config_dir.join("variables.yml").exists());
    }
    
    #[test]
    fn test_data_generator() {
        let mut gen = TestDataGenerator::new();
        
        // 生成单个数据
        let frame1 = gen.next_data_frame("test");
        assert!(frame1.tag.starts_with("test_"));
        
        // 生成批量数据
        let batch = gen.generate_batch("batch", 5);
        assert_eq!(batch.len(), 5);
        assert!(batch[0].tag.starts_with("batch."));
    }
}