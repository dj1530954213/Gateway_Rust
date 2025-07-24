//! 端到端数据流集成测试
//! 测试完整的数据链路：Mock PLC → Modbus Driver → FrameBus → MQTT Connector

use std::time::Duration;
use std::collections::HashMap;
use tokio::time::{sleep, timeout};
use serde_json::{json, Value};
use anyhow::{Result, Context};

use crate::integration::common::*;

/// 端到端数据流测试配置
#[derive(Debug, Clone)]
pub struct E2ETestConfig {
    pub test_duration: Duration,
    pub expected_frame_count: usize,
    pub data_accuracy_threshold: f64,
    pub max_latency_ms: u64,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            test_duration: Duration::from_secs(30),
            expected_frame_count: 30, // 假设每秒1帧
            data_accuracy_threshold: 99.0, // 99%准确率
            max_latency_ms: 1000, // 最大延迟1秒
        }
    }
}

/// 端到端测试套件
pub struct E2ETestSuite {
    env: TestEnvironment,
    config: E2ETestConfig,
    mqtt_client: Option<paho_mqtt::Client>,
    received_messages: Vec<MqttMessage>,
}

#[derive(Debug, Clone)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: Value,
    pub timestamp: std::time::Instant,
}

impl E2ETestSuite {
    /// 创建新的测试套件
    pub async fn new(config: E2ETestConfig) -> Result<Self> {
        let env = TestEnvironment::new().await?;
        
        Ok(Self {
            env,
            config,
            mqtt_client: None,
            received_messages: Vec::new(),
        })
    }

    /// 设置MQTT客户端
    async fn setup_mqtt_client(&mut self) -> Result<()> {
        use paho_mqtt as mqtt;
        
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(self.env.config().mqtt_url())
            .client_id("e2e_test_client")
            .finalize();
        
        let mut client = mqtt::Client::new(create_opts)?;
        
        // 设置消息接收回调
        let received_messages = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let received_clone = received_messages.clone();
        
        client.set_message_callback(move |_cli, msg| {
            if let Some(msg) = msg {
                let mqtt_msg = MqttMessage {
                    topic: msg.topic().to_string(),
                    payload: serde_json::from_slice(msg.payload())
                        .unwrap_or_else(|_| json!({"raw": std::str::from_utf8(msg.payload()).unwrap_or("invalid")})),
                    timestamp: std::time::Instant::now(),
                };
                
                if let Ok(mut messages) = received_clone.lock() {
                    messages.push(mqtt_msg);
                }
            }
        });
        
        // 连接MQTT Broker
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .finalize();
        
        client.connect(conn_opts).await
            .context("Failed to connect to MQTT broker")?;
        
        // 订阅所有相关主题
        client.subscribe("gateway/+/+", 1).await
            .context("Failed to subscribe to MQTT topics")?;
        
        self.mqtt_client = Some(client);
        println!("✅ MQTT客户端设置完成");
        
        Ok(())
    }

    /// 创建网关配置文件
    async fn create_gateway_config(&self) -> Result<()> {
        // 创建endpoints.yml
        let endpoints_config = json!({
            "mock_plc": {
                "scheme": "tcp",
                "host": "127.0.0.1",
                "port": 1502,
                "opts": {
                    "timeout": "5s"
                }
            }
        });
        
        tokio::fs::write(
            "tests/integration/test_config/endpoints.yml",
            serde_yaml::to_string(&endpoints_config)?
        ).await?;
        
        // 创建drivers.yml
        let drivers_config = json!({
            "modbus_test": {
                "kind": "static",
                "endpoint_id": "mock_plc", 
                "proto": "modbus",
                "cfg": {
                    "unit_id": 1,
                    "polling": "1s",
                    "max_regs_per_req": 100,
                    "retry": 3,
                    "endian": "big"
                }
            }
        });
        
        tokio::fs::write(
            "tests/integration/test_config/drivers.yml",
            serde_yaml::to_string(&drivers_config)?
        ).await?;
        
        // 创建variables.yml
        let variables_config = json!([
            {
                "tag": "test.temperature",
                "driver_id": "modbus_test",
                "access": "R",
                "address": {
                    "type": "modbus",
                    "kind": "holding",
                    "addr": 40001,
                    "len": 1
                },
                "datatype": "uint16",
                "scale": "value / 10",
                "unit": "°C"
            },
            {
                "tag": "test.pressure",
                "driver_id": "modbus_test", 
                "access": "R",
                "address": {
                    "type": "modbus",
                    "kind": "holding",
                    "addr": 40011,
                    "len": 1
                },
                "datatype": "uint16",
                "scale": "value / 100",
                "unit": "bar"
            },
            {
                "tag": "test.flow",
                "driver_id": "modbus_test",
                "access": "R", 
                "address": {
                    "type": "modbus",
                    "kind": "holding",
                    "addr": 40021,
                    "len": 1
                },
                "datatype": "uint16",
                "unit": "L/min"
            }
        ]);
        
        tokio::fs::write(
            "tests/integration/test_config/variables.yml",
            serde_yaml::to_string(&variables_config)?
        ).await?;
        
        println!("✅ 网关配置文件创建完成");
        Ok(())
    }

    /// 启动网关实例
    async fn start_gateway(&self) -> Result<tokio::process::Child> {
        use tokio::process::Command;
        
        // 确保配置目录存在
        tokio::fs::create_dir_all("tests/integration/test_config").await?;
        
        // 创建配置文件
        self.create_gateway_config().await?;
        
        // 启动网关进程
        let child = Command::new("cargo")
            .args(&["run", "--bin", "edge-gateway"])
            .env("GATEWAY_CONFIG_DIR", "tests/integration/test_config")
            .env("RUST_LOG", "debug")
            .env("GATEWAY_METRICS_PORT", "9100")
            .kill_on_drop(true)
            .spawn()
            .context("Failed to start gateway process")?;
        
        // 等待网关启动
        sleep(Duration::from_secs(5)).await;
        
        println!("✅ 网关实例启动完成");
        Ok(child)
    }

    /// 等待并收集MQTT消息
    async fn collect_mqtt_messages(&mut self, duration: Duration) -> Result<Vec<MqttMessage>> {
        println!("📡 开始收集MQTT消息，持续 {:?}...", duration);
        
        let start = std::time::Instant::now();
        let mut collected_messages = Vec::new();
        
        while start.elapsed() < duration {
            if let Some(ref client) = self.mqtt_client {
                // 尝试接收消息（非阻塞）
                if let Ok(Some(msg)) = client.try_receive() {
                    let mqtt_msg = MqttMessage {
                        topic: msg.topic().to_string(),
                        payload: serde_json::from_slice(msg.payload())
                            .unwrap_or_else(|_| json!({"raw": std::str::from_utf8(msg.payload()).unwrap_or("invalid")})),
                        timestamp: std::time::Instant::now(),
                    };
                    
                    println!("📨 收到消息: {} - {:?}", mqtt_msg.topic, mqtt_msg.payload);
                    collected_messages.push(mqtt_msg);
                }
            }
            
            sleep(Duration::from_millis(100)).await;
        }
        
        println!("✅ 消息收集完成，共收到 {} 条消息", collected_messages.len());
        Ok(collected_messages)
    }

    /// 验证数据准确性
    fn validate_data_accuracy(&self, messages: &[MqttMessage]) -> Result<FrameValidation> {
        println!("🔍 验证数据准确性...");
        
        let mut tag_counts = HashMap::new();
        let mut valid_messages = 0;
        
        for msg in messages {
            // 检查消息格式
            if let Some(tag) = msg.payload.get("tag").and_then(|v| v.as_str()) {
                *tag_counts.entry(tag.to_string()).or_insert(0) += 1;
                
                // 验证必需字段
                if msg.payload.get("value").is_some() &&
                   msg.payload.get("timestamp").is_some() &&
                   msg.payload.get("unit").is_some() {
                    valid_messages += 1;
                }
            }
        }
        
        let total_messages = messages.len();
        let accuracy = if total_messages > 0 {
            (valid_messages as f64 / total_messages as f64) * 100.0
        } else {
            0.0
        };
        
        println!("📊 数据验证结果:");
        println!("  总消息数: {}", total_messages);
        println!("  有效消息数: {}", valid_messages);
        println!("  准确率: {:.2}%", accuracy);
        println!("  标签统计: {:?}", tag_counts);
        
        Ok(FrameValidation {
            total_frames: total_messages,
            valid_frames: valid_messages,
            invalid_frames: total_messages - valid_messages,
            accuracy_percentage: accuracy,
        })
    }

    /// 测试延迟
    async fn test_latency(&self) -> Result<Duration> {
        println!("⏱️  测试端到端延迟...");
        
        // 向Mock PLC发送控制命令，触发数据变化
        let response = reqwest::Client::new()
            .post("http://127.0.0.1:8080/")
            .json(&json!({
                "command": "inject_fault",
                "error_rate": 0.0,
                "response_delay": 0.0
            }))
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to trigger PLC data change");
        }
        
        let start = std::time::Instant::now();
        
        // 等待MQTT消息出现（简化的延迟测试）
        let mut attempts = 0;
        while attempts < 100 {  // 最多等待10秒
            if let Some(ref client) = self.mqtt_client {
                if client.try_receive().is_ok() {
                    break;
                }
            }
            sleep(Duration::from_millis(100)).await;
            attempts += 1;
        }
        
        let latency = start.elapsed();
        println!("✅ 端到端延迟: {:?}", latency);
        
        Ok(latency)
    }
}

/// 主要的端到端测试函数
#[tokio::test]
async fn test_end_to_end_data_flow() -> Result<()> {
    println!("🚀 开始端到端数据流测试...");
    
    let config = E2ETestConfig::default();
    let mut test_suite = E2ETestSuite::new(config.clone()).await?;
    
    // 设置MQTT客户端
    test_suite.setup_mqtt_client().await?;
    
    // 启动网关
    let _gateway_process = test_suite.start_gateway().await?;
    
    // 等待系统稳定
    sleep(Duration::from_secs(10)).await;
    
    // 收集MQTT消息
    let messages = test_suite.collect_mqtt_messages(config.test_duration).await?;
    
    // 验证消息数量
    assert!(
        messages.len() >= config.expected_frame_count / 2,
        "期望至少收到 {} 条消息，实际收到 {} 条",
        config.expected_frame_count / 2,
        messages.len()
    );
    
    // 验证数据准确性
    let validation = test_suite.validate_data_accuracy(&messages)?;
    assert!(
        validation.meets_accuracy_requirement(config.data_accuracy_threshold),
        "数据准确率 {:.2}% 低于要求的 {:.2}%",
        validation.accuracy(),
        config.data_accuracy_threshold
    );
    
    // 测试延迟
    let latency = test_suite.test_latency().await?;
    assert!(
        latency.as_millis() <= config.max_latency_ms as u128,
        "端到端延迟 {:?} 超过最大允许值 {}ms",
        latency,
        config.max_latency_ms
    );
    
    println!("✅ 端到端数据流测试完成");
    Ok(())
}

/// 测试数据完整性 - 验证所有配置的标签都有数据
#[tokio::test]
async fn test_data_completeness() -> Result<()> {
    println!("🔍 测试数据完整性...");
    
    let config = E2ETestConfig {
        test_duration: Duration::from_secs(20),
        ..Default::default()
    };
    
    let mut test_suite = E2ETestSuite::new(config.clone()).await?;
    test_suite.setup_mqtt_client().await?;
    let _gateway_process = test_suite.start_gateway().await?;
    
    sleep(Duration::from_secs(5)).await;
    
    let messages = test_suite.collect_mqtt_messages(config.test_duration).await?;
    
    // 期望的标签
    let expected_tags = vec!["test.temperature", "test.pressure", "test.flow"];
    let mut found_tags = std::collections::HashSet::new();
    
    for msg in &messages {
        if let Some(tag) = msg.payload.get("tag").and_then(|v| v.as_str()) {
            found_tags.insert(tag.to_string());
        }
    }
    
    for expected_tag in &expected_tags {
        assert!(
            found_tags.contains(*expected_tag),
            "未收到标签 {} 的数据",
            expected_tag
        );
    }
    
    println!("✅ 数据完整性测试通过，所有标签都有数据");
    Ok(())
}

/// 测试数据格式标准化
#[tokio::test]
async fn test_data_format_standardization() -> Result<()> {
    println!("📋 测试数据格式标准化...");
    
    let config = E2ETestConfig {
        test_duration: Duration::from_secs(15),
        ..Default::default()
    };
    
    let mut test_suite = E2ETestSuite::new(config.clone()).await?;
    test_suite.setup_mqtt_client().await?;
    let _gateway_process = test_suite.start_gateway().await?;
    
    sleep(Duration::from_secs(3)).await;
    
    let messages = test_suite.collect_mqtt_messages(config.test_duration).await?;
    
    // 验证每条消息的格式
    for msg in &messages {
        // 检查必需字段
        assert!(msg.payload.get("tag").is_some(), "消息缺少 'tag' 字段: {:?}", msg.payload);
        assert!(msg.payload.get("value").is_some(), "消息缺少 'value' 字段: {:?}", msg.payload);
        assert!(msg.payload.get("timestamp").is_some(), "消息缺少 'timestamp' 字段: {:?}", msg.payload);
        
        // 检查数据类型
        if let Some(value) = msg.payload.get("value") {
            assert!(value.is_number(), "value字段应该是数字: {:?}", value);
        }
        
        if let Some(timestamp) = msg.payload.get("timestamp") {
            assert!(timestamp.is_number(), "timestamp字段应该是数字: {:?}", timestamp);
        }
    }
    
    println!("✅ 数据格式标准化测试通过");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_e2e_config_creation() {
        let config = E2ETestConfig::default();
        assert_eq!(config.test_duration, Duration::from_secs(30));
        assert_eq!(config.expected_frame_count, 30);
        assert_eq!(config.data_accuracy_threshold, 99.0);
    }
}