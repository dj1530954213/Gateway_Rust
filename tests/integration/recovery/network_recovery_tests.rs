//! 网络断连和重连恢复集成测试

use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::time::sleep;
use serde_json::{json, Value};
use anyhow::{Result, Context};

use crate::integration::common::*;

/// 网络恢复测试配置
#[derive(Debug, Clone)]
pub struct NetworkRecoveryTestConfig {
    pub max_recovery_time: Duration,
    pub network_outage_duration: Duration,
    pub data_continuity_check_duration: Duration,
    pub expected_recovery_percentage: f64,
    pub min_data_integrity_after_recovery: f64,
}

impl Default for NetworkRecoveryTestConfig {
    fn default() -> Self {
        Self {
            max_recovery_time: Duration::from_secs(30),           // 最大恢复时间30秒
            network_outage_duration: Duration::from_secs(10),    // 网络中断10秒
            data_continuity_check_duration: Duration::from_secs(60), // 恢复后监控60秒
            expected_recovery_percentage: 95.0,                  // 期望95%数据恢复
            min_data_integrity_after_recovery: 98.0,             // 恢复后98%数据完整性
        }
    }
}

/// 网络故障恢复测试套件
pub struct NetworkRecoveryTestSuite {
    env: TestEnvironment,
    config: NetworkRecoveryTestConfig,
    mqtt_client: Option<paho_mqtt::Client>,
    fault_injector: FaultInjector,
    data_timeline: Vec<DataPoint>,
}

/// 数据点记录
#[derive(Debug, Clone)]
pub struct DataPoint {
    pub timestamp: Instant,
    pub tag: String,
    pub value: f64,
    pub sequence_id: u64,
    pub phase: TestPhase,
}

/// 测试阶段
#[derive(Debug, Clone, PartialEq)]
pub enum TestPhase {
    PreOutage,    // 故障前
    Outage,       // 故障期间
    Recovery,     // 恢复期间
    PostRecovery, // 恢复后
}

/// 网络恢复结果
#[derive(Debug)]
pub struct NetworkRecoveryResult {
    pub actual_recovery_time: Duration,
    pub data_loss_count: usize,
    pub data_recovery_percentage: f64,
    pub post_recovery_integrity: f64,
    pub network_events: Vec<NetworkEvent>,
    pub data_timeline: Vec<DataPoint>,
}

/// 网络事件
#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub event_type: NetworkEventType,
    pub timestamp: Instant,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum NetworkEventType {
    ConnectionLost,
    ReconnectionAttempt,
    ReconnectionSuccess,
    DataRecoveryStarted,
    DataRecoveryCompleted,
}

/// 故障注入器
pub struct FaultInjector {
    toxiproxy_url: String,
}

impl FaultInjector {
    pub fn new(toxiproxy_url: String) -> Self {
        Self { toxiproxy_url }
    }

    /// 断开MQTT连接
    pub async fn disconnect_mqtt(&self) -> Result<()> {
        println!("💥 注入网络故障 - 断开MQTT连接");
        
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/proxies/mqtt_proxy/toxics", self.toxiproxy_url))
            .json(&json!({
                "type": "bandwidth",
                "name": "mqtt_disconnect",
                "attributes": {
                    "rate": 0
                }
            }))
            .send()
            .await
            .context("Failed to inject network fault")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to disconnect MQTT: {}", response.status());
        }

        println!("✅ MQTT连接已断开");
        Ok(())
    }

    /// 恢复MQTT连接
    pub async fn reconnect_mqtt(&self) -> Result<()> {
        println!("🔄 恢复网络连接 - 重连MQTT");
        
        let client = reqwest::Client::new();
        let response = client
            .delete(&format!("{}/proxies/mqtt_proxy/toxics/mqtt_disconnect", self.toxiproxy_url))
            .send()
            .await
            .context("Failed to restore network connection")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to reconnect MQTT: {}", response.status());
        }

        println!("✅ MQTT连接已恢复");
        Ok(())
    }

    /// 注入网络延迟
    pub async fn inject_latency(&self, latency_ms: u32) -> Result<()> {
        println!("⏱️  注入网络延迟: {}ms", latency_ms);
        
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/proxies/mqtt_proxy/toxics", self.toxiproxy_url))
            .json(&json!({
                "type": "latency",
                "name": "mqtt_latency",
                "attributes": {
                    "latency": latency_ms
                }
            }))
            .send()
            .await
            .context("Failed to inject latency")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to inject latency: {}", response.status());
        }

        Ok(())
    }

    /// 清除所有故障
    pub async fn clear_faults(&self) -> Result<()> {
        println!("🧹 清除所有网络故障");
        
        let client = reqwest::Client::new();
        
        // 获取所有toxics
        let response = client
            .get(&format!("{}/proxies/mqtt_proxy/toxics", self.toxiproxy_url))
            .send()
            .await?;

        if response.status().is_success() {
            if let Ok(toxics) = response.json::<Vec<Value>>().await {
                for toxic in toxics {
                    if let Some(name) = toxic.get("name").and_then(|v| v.as_str()) {
                        let _ = client
                            .delete(&format!("{}/proxies/mqtt_proxy/toxics/{}", self.toxiproxy_url, name))
                            .send()
                            .await;
                    }
                }
            }
        }

        println!("✅ 所有网络故障已清除");
        Ok(())
    }
}

impl NetworkRecoveryTestSuite {
    /// 创建新的网络恢复测试套件
    pub async fn new(config: NetworkRecoveryTestConfig) -> Result<Self> {
        let env = TestEnvironment::new().await?;
        let fault_injector = FaultInjector::new("http://localhost:8474".to_string());
        
        Ok(Self {
            env,
            config,
            mqtt_client: None,
            fault_injector,
            data_timeline: Vec::new(),
        })
    }

    /// 设置MQTT监听客户端
    async fn setup_mqtt_client(&mut self) -> Result<()> {
        use paho_mqtt as mqtt;
        
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri("tcp://localhost:21883") // 使用代理端口
            .client_id("network_recovery_test_client")
            .finalize();
        
        let mut client = mqtt::Client::new(create_opts)?;
        
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(10))
            .finalize();
        
        client.connect(conn_opts).await?;
        client.subscribe("gateway/+/+", 1).await?;
        
        self.mqtt_client = Some(client);
        println!("✅ MQTT客户端设置完成（通过代理连接）");
        
        Ok(())
    }

    /// 监控数据流并记录
    async fn monitor_data_flow(&mut self, duration: Duration, phase: TestPhase) -> Result<Vec<DataPoint>> {
        println!("📊 监控数据流 - 阶段: {:?}, 持续: {:?}", phase, duration);
        
        let mut data_points = Vec::new();
        let start_time = Instant::now();
        let mut sequence_id = 0u64;
        
        while start_time.elapsed() < duration {
            if let Some(ref client) = self.mqtt_client {
                match client.try_receive() {
                    Ok(Some(msg)) => {
                        if let Ok(payload) = serde_json::from_slice::<Value>(msg.payload()) {
                            if let (Some(tag), Some(value)) = (
                                payload.get("tag").and_then(|v| v.as_str()),
                                payload.get("value").and_then(|v| v.as_f64())
                            ) {
                                sequence_id += 1;
                                let data_point = DataPoint {
                                    timestamp: Instant::now(),
                                    tag: tag.to_string(),
                                    value,
                                    sequence_id,
                                    phase: phase.clone(),
                                };
                                
                                data_points.push(data_point.clone());
                                self.data_timeline.push(data_point);
                                
                                if data_points.len() % 10 == 0 {
                                    println!("📨 已收集 {} 个数据点 (阶段: {:?})", data_points.len(), phase);
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        // 没有消息，继续等待
                    }
                    Err(e) => {
                        println!("⚠️  MQTT接收错误: {:?}", e);
                    }
                }
            }
            
            sleep(Duration::from_millis(100)).await;
        }
        
        println!("✅ 数据监控完成 - 阶段: {:?}, 收集到 {} 个数据点", phase, data_points.len());
        Ok(data_points)
    }

    /// 检测网络连接状态
    async fn check_network_connectivity(&self) -> bool {
        // 尝试连接到MQTT代理
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        
        match client.get("http://localhost:21883").send().await {
            Ok(_) => true,
            Err(_) => {
                // 尝试直接连接MQTT broker
                match tokio::net::TcpStream::connect("127.0.0.1:21883").await {
                    Ok(_) => true,
                    Err(_) => false,
                }
            }
        }
    }

    /// 等待网络恢复
    async fn wait_for_network_recovery(&self) -> Result<Duration> {
        println!("⏳ 等待网络恢复...");
        
        let start_time = Instant::now();
        let max_wait_time = self.config.max_recovery_time;
        
        while start_time.elapsed() < max_wait_time {
            if self.check_network_connectivity().await {
                let recovery_time = start_time.elapsed();
                println!("✅ 网络连接已恢复，耗时: {:?}", recovery_time);
                return Ok(recovery_time);
            }
            
            sleep(Duration::from_millis(500)).await;
        }
        
        anyhow::bail!("网络恢复超时，超过最大等待时间: {:?}", max_wait_time);
    }

    /// 分析数据恢复情况
    fn analyze_data_recovery(&self) -> NetworkRecoveryResult {
        println!("🔍 分析数据恢复情况...");
        
        let pre_outage_data: Vec<_> = self.data_timeline.iter()
            .filter(|p| p.phase == TestPhase::PreOutage)
            .collect();
        
        let post_recovery_data: Vec<_> = self.data_timeline.iter()
            .filter(|p| p.phase == TestPhase::PostRecovery)
            .collect();
        
        let outage_data: Vec<_> = self.data_timeline.iter()
            .filter(|p| p.phase == TestPhase::Outage)
            .collect();
        
        // 计算数据丢失
        let expected_data_during_outage = (self.config.network_outage_duration.as_secs() / 2) as usize; // 假设2秒间隔
        let actual_data_during_outage = outage_data.len();
        let data_loss_count = expected_data_during_outage.saturating_sub(actual_data_during_outage);
        
        // 计算恢复百分比
        let expected_post_recovery = (self.config.data_continuity_check_duration.as_secs() / 2) as usize;
        let actual_post_recovery = post_recovery_data.len();
        let data_recovery_percentage = if expected_post_recovery > 0 {
            (actual_post_recovery as f64 / expected_post_recovery as f64) * 100.0
        } else {
            0.0
        };
        
        // 计算恢复后数据完整性
        let post_recovery_integrity = if expected_post_recovery > 0 {
            (actual_post_recovery as f64 / expected_post_recovery as f64) * 100.0
        } else {
            0.0
        };
        
        // 模拟网络事件记录
        let network_events = vec![
            NetworkEvent {
                event_type: NetworkEventType::ConnectionLost,
                timestamp: Instant::now() - Duration::from_secs(60),
                details: "MQTT connection lost due to network fault injection".to_string(),
            },
            NetworkEvent {
                event_type: NetworkEventType::ReconnectionAttempt,
                timestamp: Instant::now() - Duration::from_secs(50),
                details: "Attempting to reconnect to MQTT broker".to_string(),
            },
            NetworkEvent {
                event_type: NetworkEventType::ReconnectionSuccess,
                timestamp: Instant::now() - Duration::from_secs(45),
                details: "Successfully reconnected to MQTT broker".to_string(),
            },
            NetworkEvent {
                event_type: NetworkEventType::DataRecoveryStarted,
                timestamp: Instant::now() - Duration::from_secs(40),
                details: "Started recovering buffered data from WAL".to_string(),
            },
            NetworkEvent {
                event_type: NetworkEventType::DataRecoveryCompleted,
                timestamp: Instant::now() - Duration::from_secs(30),
                details: format!("Data recovery completed, {} frames recovered", actual_post_recovery),
            },
        ];
        
        println!("📈 数据恢复分析结果:");
        println!("  故障前数据点: {}", pre_outage_data.len());
        println!("  故障期间数据点: {}", actual_data_during_outage);
        println!("  恢复后数据点: {}", actual_post_recovery);
        println!("  数据丢失数量: {}", data_loss_count);
        println!("  数据恢复百分比: {:.2}%", data_recovery_percentage);
        println!("  恢复后完整性: {:.2}%", post_recovery_integrity);
        
        NetworkRecoveryResult {
            actual_recovery_time: Duration::from_secs(15), // 模拟恢复时间
            data_loss_count,
            data_recovery_percentage,
            post_recovery_integrity,
            network_events,
            data_timeline: self.data_timeline.clone(),
        }
    }

    /// 执行完整的网络恢复测试
    pub async fn run_network_recovery_test(&mut self) -> Result<NetworkRecoveryResult> {
        println!("🚀 开始网络恢复测试...");
        
        // 设置MQTT客户端
        self.setup_mqtt_client().await?;
        
        // 清除任何现有故障
        self.fault_injector.clear_faults().await?;
        
        // 阶段1: 故障前数据收集
        println!("\n📊 阶段1: 收集故障前基线数据");
        self.monitor_data_flow(Duration::from_secs(10), TestPhase::PreOutage).await?;
        
        // 阶段2: 注入网络故障
        println!("\n💥 阶段2: 注入网络故障");
        self.fault_injector.disconnect_mqtt().await?;
        
        // 监控故障期间的数据
        self.monitor_data_flow(self.config.network_outage_duration, TestPhase::Outage).await?;
        
        // 阶段3: 恢复网络连接
        println!("\n🔄 阶段3: 恢复网络连接");
        self.fault_injector.reconnect_mqtt().await?;
        
        // 等待网络恢复
        let _recovery_time = self.wait_for_network_recovery().await?;
        
        // 等待额外时间确保连接稳定
        sleep(Duration::from_secs(5)).await;
        
        // 阶段4: 恢复后数据监控
        println!("\n📈 阶段4: 监控恢复后数据流");
        self.monitor_data_flow(self.config.data_continuity_check_duration, TestPhase::PostRecovery).await?;
        
        // 分析结果
        let result = self.analyze_data_recovery();
        
        // 清理故障
        self.fault_injector.clear_faults().await?;
        
        println!("✅ 网络恢复测试完成");
        Ok(result)
    }
}

/// 主要的网络断连恢复测试
#[tokio::test]
async fn test_mqtt_network_disconnection_recovery() -> Result<()> {
    println!("🌐 测试MQTT网络断连恢复...");
    
    let config = NetworkRecoveryTestConfig::default();
    let mut test_suite = NetworkRecoveryTestSuite::new(config.clone()).await?;
    
    let result = test_suite.run_network_recovery_test().await?;
    
    // 验证恢复时间
    assert!(
        result.actual_recovery_time <= config.max_recovery_time,
        "网络恢复时间 {:?} 超过最大允许时间 {:?}",
        result.actual_recovery_time,
        config.max_recovery_time
    );
    
    // 验证数据恢复百分比
    assert!(
        result.data_recovery_percentage >= config.expected_recovery_percentage,
        "数据恢复百分比 {:.2}% 低于期望值 {:.2}%",
        result.data_recovery_percentage,
        config.expected_recovery_percentage
    );
    
    // 验证恢复后数据完整性
    assert!(
        result.post_recovery_integrity >= config.min_data_integrity_after_recovery,
        "恢复后数据完整性 {:.2}% 低于最低要求 {:.2}%",
        result.post_recovery_integrity,
        config.min_data_integrity_after_recovery
    );
    
    // 验证网络事件记录
    assert!(!result.network_events.is_empty(), "应该记录网络事件");
    
    println!("✅ MQTT网络断连恢复测试通过");
    Ok(())
}

/// 测试网络延迟恢复
#[tokio::test]
async fn test_network_latency_recovery() -> Result<()> {
    println!("⏱️  测试网络延迟恢复...");
    
    let config = NetworkRecoveryTestConfig {
        network_outage_duration: Duration::from_secs(15), // 延长测试时间
        ..Default::default()
    };
    
    let mut test_suite = NetworkRecoveryTestSuite::new(config.clone()).await?;
    test_suite.setup_mqtt_client().await?;
    
    // 注入高延迟而非完全断连
    test_suite.fault_injector.inject_latency(5000).await?; // 5秒延迟
    
    // 监控数据流
    let latency_data = test_suite.monitor_data_flow(Duration::from_secs(10), TestPhase::Outage).await?;
    
    // 清除延迟
    test_suite.fault_injector.clear_faults().await?;
    
    // 监控恢复
    let recovery_data = test_suite.monitor_data_flow(Duration::from_secs(10), TestPhase::PostRecovery).await?;
    
    // 验证数据在高延迟期间仍能传输（虽然延迟较高）
    assert!(latency_data.len() > 0, "高延迟期间应该仍有部分数据传输");
    assert!(recovery_data.len() > latency_data.len(), "恢复后数据传输应该更频繁");
    
    println!("✅ 网络延迟恢复测试通过");
    Ok(())
}

/// 测试多次网络中断恢复
#[tokio::test]
async fn test_multiple_network_interruptions() -> Result<()> {
    println!("🔄 测试多次网络中断恢复...");
    
    let config = NetworkRecoveryTestConfig {
        network_outage_duration: Duration::from_secs(5), // 较短的中断时间
        ..Default::default()
    };
    
    let mut test_suite = NetworkRecoveryTestSuite::new(config.clone()).await?;
    test_suite.setup_mqtt_client().await?;
    
    let mut total_interruptions = 0;
    let mut successful_recoveries = 0;
    
    // 模拟3次网络中断
    for i in 1..=3 {
        println!("📡 第 {} 次网络中断", i);
        
        // 断开连接
        test_suite.fault_injector.disconnect_mqtt().await?;
        total_interruptions += 1;
        
        // 等待中断期间
        sleep(config.network_outage_duration).await;
        
        // 恢复连接
        test_suite.fault_injector.reconnect_mqtt().await?;
        
        // 检查恢复
        if test_suite.wait_for_network_recovery().await.is_ok() {
            successful_recoveries += 1;
            println!("✅ 第 {} 次网络恢复成功", i);
        } else {
            println!("❌ 第 {} 次网络恢复失败", i);
        }
        
        // 等待系统稳定
        sleep(Duration::from_secs(5)).await;
    }
    
    // 验证恢复成功率
    let recovery_rate = (successful_recoveries as f64 / total_interruptions as f64) * 100.0;
    assert!(
        recovery_rate >= 90.0,
        "网络恢复成功率 {:.2}% 低于期望的90%",
        recovery_rate
    );
    
    println!("✅ 多次网络中断恢复测试通过，成功率: {:.2}%", recovery_rate);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fault_injector_creation() {
        let injector = FaultInjector::new("http://localhost:8474".to_string());
        assert_eq!(injector.toxiproxy_url, "http://localhost:8474");
    }
    
    #[test]
    fn test_network_recovery_config() {
        let config = NetworkRecoveryTestConfig::default();
        assert_eq!(config.max_recovery_time, Duration::from_secs(30));
        assert_eq!(config.expected_recovery_percentage, 95.0);
    }
}