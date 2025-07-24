//! MQTT重连机制和错误处理测试

use mqtt5::config::{MqttCfg, BatchCfg, MqttMessage, DataPoint};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use frame_bus::Value;

/// 模拟的MQTT连接状态
#[derive(Debug, Clone, PartialEq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

/// 模拟的MQTT连接器用于测试重连逻辑
struct MockMqttClient {
    state: Arc<Mutex<ConnectionState>>,
    connection_attempts: Arc<Mutex<usize>>,
    last_attempt: Arc<Mutex<Option<Instant>>>,
    reconnect_delay: Duration,
    max_retries: usize,
    should_fail: Arc<Mutex<bool>>,
    published_messages: Arc<Mutex<Vec<MqttMessage>>>,
    publish_failures: Arc<Mutex<usize>>,
}

impl MockMqttClient {
    fn new(reconnect_delay: Duration, max_retries: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            connection_attempts: Arc::new(Mutex::new(0)),
            last_attempt: Arc::new(Mutex::new(None)),
            reconnect_delay,
            max_retries,
            should_fail: Arc::new(Mutex::new(false)),
            published_messages: Arc::new(Mutex::new(Vec::new())),
            publish_failures: Arc::new(Mutex::new(0)),
        }
    }

    fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().unwrap() = fail;
    }

    fn connect(&self) -> Result<(), String> {
        let mut attempts = self.connection_attempts.lock().unwrap();
        *attempts += 1;
        *self.last_attempt.lock().unwrap() = Some(Instant::now());

        if *self.should_fail.lock().unwrap() {
            *self.state.lock().unwrap() = ConnectionState::Failed;
            return Err("Connection failed".to_string());
        }

        if *attempts <= self.max_retries {
            *self.state.lock().unwrap() = ConnectionState::Connected;
            Ok(())
        } else {
            *self.state.lock().unwrap() = ConnectionState::Failed;
            Err("Max retries exceeded".to_string())
        }
    }

    fn disconnect(&self) {
        *self.state.lock().unwrap() = ConnectionState::Disconnected;
    }

    fn is_connected(&self) -> bool {
        matches!(*self.state.lock().unwrap(), ConnectionState::Connected)
    }

    fn get_state(&self) -> ConnectionState {
        self.state.lock().unwrap().clone()
    }

    fn get_connection_attempts(&self) -> usize {
        *self.connection_attempts.lock().unwrap()
    }

    fn publish(&self, message: MqttMessage) -> Result<(), String> {
        if !self.is_connected() {
            *self.publish_failures.lock().unwrap() += 1;
            return Err("Not connected".to_string());
        }

        if *self.should_fail.lock().unwrap() {
            *self.publish_failures.lock().unwrap() += 1;
            return Err("Publish failed".to_string());
        }

        self.published_messages.lock().unwrap().push(message);
        Ok(())
    }

    fn get_published_count(&self) -> usize {
        self.published_messages.lock().unwrap().len()
    }

    fn get_publish_failures(&self) -> usize {
        *self.publish_failures.lock().unwrap()
    }

    fn simulate_disconnect(&self) {
        *self.state.lock().unwrap() = ConnectionState::Disconnected;
    }
}

/// 重连管理器
struct ReconnectManager {
    client: MockMqttClient,
    retry_count: usize,
    max_retries: usize,
    base_delay: Duration,
    max_delay: Duration,
}

impl ReconnectManager {
    fn new(client: MockMqttClient, max_retries: usize, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            client,
            retry_count: 0,
            max_retries,
            base_delay,
            max_delay,
        }
    }

    fn attempt_reconnect(&mut self) -> Result<(), String> {
        if self.retry_count >= self.max_retries {
            return Err("Max retries exceeded".to_string());
        }

        // 指数退避延迟
        let delay = std::cmp::min(
            self.base_delay * (2u32.pow(self.retry_count as u32)),
            self.max_delay
        );

        // 在实际实现中，这里会有真正的延迟
        // std::thread::sleep(delay);

        match self.client.connect() {
            Ok(_) => {
                self.retry_count = 0; // 重置重试计数
                Ok(())
            }
            Err(e) => {
                self.retry_count += 1;
                Err(e)
            }
        }
    }

    fn get_retry_count(&self) -> usize {
        self.retry_count
    }

    fn get_current_delay(&self) -> Duration {
        std::cmp::min(
            self.base_delay * (2u32.pow(self.retry_count as u32)),
            self.max_delay
        )
    }
}

#[test]
fn test_initial_connection_success() {
    let client = MockMqttClient::new(Duration::from_secs(1), 3);
    client.set_should_fail(false);

    assert_eq!(client.get_state(), ConnectionState::Disconnected);
    assert!(!client.is_connected());

    let result = client.connect();
    assert!(result.is_ok());
    assert!(client.is_connected());
    assert_eq!(client.get_state(), ConnectionState::Connected);
    assert_eq!(client.get_connection_attempts(), 1);
}

#[test]
fn test_initial_connection_failure() {
    let client = MockMqttClient::new(Duration::from_secs(1), 3);
    client.set_should_fail(true);

    let result = client.connect();
    assert!(result.is_err());
    assert!(!client.is_connected());
    assert_eq!(client.get_state(), ConnectionState::Failed);
    assert_eq!(client.get_connection_attempts(), 1);
}

#[test]
fn test_reconnect_with_exponential_backoff() {
    let client = MockMqttClient::new(Duration::from_secs(1), 5);
    let mut manager = ReconnectManager::new(
        client,
        5,
        Duration::from_secs(1),
        Duration::from_secs(30),
    );

    // 第一次重连尝试
    let delay1 = manager.get_current_delay();
    assert_eq!(delay1, Duration::from_secs(1)); // 2^0 * 1s = 1s

    manager.client.set_should_fail(true);
    let result1 = manager.attempt_reconnect();
    assert!(result1.is_err());
    assert_eq!(manager.get_retry_count(), 1);

    // 第二次重连尝试
    let delay2 = manager.get_current_delay();
    assert_eq!(delay2, Duration::from_secs(2)); // 2^1 * 1s = 2s

    let result2 = manager.attempt_reconnect();
    assert!(result2.is_err());
    assert_eq!(manager.get_retry_count(), 2);

    // 第三次重连尝试
    let delay3 = manager.get_current_delay();
    assert_eq!(delay3, Duration::from_secs(4)); // 2^2 * 1s = 4s

    let result3 = manager.attempt_reconnect();
    assert!(result3.is_err());
    assert_eq!(manager.get_retry_count(), 3);

    // 第四次重连尝试
    let delay4 = manager.get_current_delay();
    assert_eq!(delay4, Duration::from_secs(8)); // 2^3 * 1s = 8s

    // 成功重连，重试计数应该重置
    manager.client.set_should_fail(false);
    let result4 = manager.attempt_reconnect();
    assert!(result4.is_ok());
    assert_eq!(manager.get_retry_count(), 0); // 重置
    assert!(manager.client.is_connected());
}

#[test]
fn test_max_retry_limit() {
    let client = MockMqttClient::new(Duration::from_secs(1), 2);
    let mut manager = ReconnectManager::new(
        client,
        3, // 最大3次重试
        Duration::from_secs(1),
        Duration::from_secs(30),
    );

    manager.client.set_should_fail(true);

    // 尝试重连直到达到最大次数
    for i in 1..=3 {
        let result = manager.attempt_reconnect();
        assert!(result.is_err());
        assert_eq!(manager.get_retry_count(), i);
    }

    // 超过最大重试次数应该立即失败
    let result = manager.attempt_reconnect();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Max retries exceeded"));
}

#[test]
fn test_max_delay_cap() {
    let client = MockMqttClient::new(Duration::from_secs(1), 10);
    let mut manager = ReconnectManager::new(
        client,
        10,
        Duration::from_secs(2),
        Duration::from_secs(10), // 最大延迟10秒
    );

    manager.client.set_should_fail(true);

    // 测试延迟上限
    for i in 1..=6 {
        let delay_before = manager.get_current_delay();
        let _result = manager.attempt_reconnect();
        
        match i {
            1 => assert_eq!(delay_before, Duration::from_secs(2)), // 2^0 * 2s = 2s
            2 => assert_eq!(delay_before, Duration::from_secs(4)), // 2^1 * 2s = 4s
            3 => assert_eq!(delay_before, Duration::from_secs(8)), // 2^2 * 2s = 8s
            4 => assert_eq!(delay_before, Duration::from_secs(10)), // min(2^3 * 2s, 10s) = 10s
            5 => assert_eq!(delay_before, Duration::from_secs(10)), // 保持在最大值
            6 => assert_eq!(delay_before, Duration::from_secs(10)), // 保持在最大值
            _ => {}
        }
    }
}

#[test]
fn test_publish_during_disconnection() {
    let client = MockMqttClient::new(Duration::from_secs(1), 3);
    
    // 初始未连接状态
    assert!(!client.is_connected());

    let message = create_test_message("test-device", 1);
    let result = client.publish(message);
    assert!(result.is_err());
    assert_eq!(client.get_publish_failures(), 1);
    assert_eq!(client.get_published_count(), 0);
}

#[test]
fn test_publish_after_reconnection() {
    let client = MockMqttClient::new(Duration::from_secs(1), 3);
    
    // 连接成功
    client.set_should_fail(false);
    let _result = client.connect();
    assert!(client.is_connected());

    // 发布消息成功
    let message1 = create_test_message("device1", 1);
    let result1 = client.publish(message1);
    assert!(result1.is_ok());
    assert_eq!(client.get_published_count(), 1);

    // 模拟断开连接
    client.simulate_disconnect();
    assert!(!client.is_connected());

    // 尝试发布应该失败
    let message2 = create_test_message("device2", 2);
    let result2 = client.publish(message2);
    assert!(result2.is_err());
    assert_eq!(client.get_publish_failures(), 1);

    // 重新连接
    let _result = client.connect();
    assert!(client.is_connected());

    // 发布应该再次成功
    let message3 = create_test_message("device3", 3);
    let result3 = client.publish(message3);
    assert!(result3.is_ok());
    assert_eq!(client.get_published_count(), 2);
}

#[test]
fn test_message_buffering_during_disconnection() {
    // 模拟消息缓冲器
    let mut message_buffer: Vec<MqttMessage> = Vec::new();
    let client = MockMqttClient::new(Duration::from_secs(1), 3);

    // 断开状态下积累消息
    assert!(!client.is_connected());
    
    for i in 1..=5 {
        let message = create_test_message(&format!("device{}", i), i);
        message_buffer.push(message);
    }

    assert_eq!(message_buffer.len(), 5);

    // 连接后发送缓冲的消息
    client.set_should_fail(false);
    let _result = client.connect();
    assert!(client.is_connected());

    // 发送缓冲的消息
    while let Some(message) = message_buffer.pop() {
        let result = client.publish(message);
        assert!(result.is_ok());
    }

    assert_eq!(client.get_published_count(), 5);
    assert_eq!(message_buffer.len(), 0);
}

#[test]
fn test_reconnect_state_transitions() {
    let client = MockMqttClient::new(Duration::from_secs(1), 3);
    
    // 初始状态
    assert_eq!(client.get_state(), ConnectionState::Disconnected);

    // 连接失败
    client.set_should_fail(true);
    let _result = client.connect();
    assert_eq!(client.get_state(), ConnectionState::Failed);

    // 断开连接后状态变化
    client.disconnect();
    assert_eq!(client.get_state(), ConnectionState::Disconnected);

    // 成功连接
    client.set_should_fail(false);
    let _result = client.connect();
    assert_eq!(client.get_state(), ConnectionState::Connected);

    // 模拟网络断开
    client.simulate_disconnect();
    assert_eq!(client.get_state(), ConnectionState::Disconnected);
}

#[test]
fn test_publish_retry_mechanism() {
    let client = MockMqttClient::new(Duration::from_secs(1), 3);
    
    // 连接成功但发布失败
    client.set_should_fail(false);
    let _result = client.connect();
    assert!(client.is_connected());

    // 设置发布失败
    client.set_should_fail(true);
    
    let message = create_test_message("retry-device", 1);
    let result = client.publish(message.clone());
    assert!(result.is_err());
    assert_eq!(client.get_publish_failures(), 1);

    // 模拟重试发布（恢复正常）
    client.set_should_fail(false);
    let retry_result = client.publish(message);
    assert!(retry_result.is_ok());
    assert_eq!(client.get_published_count(), 1);
}

#[test]
fn test_connection_health_monitoring() {
    let client = MockMqttClient::new(Duration::from_secs(1), 3);
    
    // 连接健康检查
    assert!(!client.is_connected());
    
    client.set_should_fail(false);
    let _result = client.connect();
    assert!(client.is_connected());

    // 模拟连接健康状态的监控逻辑
    let connection_checks = 5;
    let mut health_check_results = Vec::new();

    for _i in 0..connection_checks {
        health_check_results.push(client.is_connected());
        
        // 在第3次检查后模拟断开
        if health_check_results.len() == 3 {
            client.simulate_disconnect();
        }
    }

    // 验证健康检查结果
    assert_eq!(health_check_results, vec![true, true, true, false, false]);
}

#[test]
fn test_burst_reconnection_attempts() {
    let client = MockMqttClient::new(Duration::from_millis(100), 10);
    let mut manager = ReconnectManager::new(
        client,
        5,
        Duration::from_millis(100),
        Duration::from_secs(2),
    );

    manager.client.set_should_fail(true);

    // 快速连续的重连尝试
    let start_time = Instant::now();
    let mut attempts = 0;
    
    while attempts < 5 {
        let _result = manager.attempt_reconnect();
        attempts += 1;
    }

    let elapsed = start_time.elapsed();
    assert_eq!(attempts, 5);
    assert_eq!(manager.get_retry_count(), 5);
    
    // 验证时间消耗合理（不包含实际延迟，因为测试中没有真正sleep）
    assert!(elapsed < Duration::from_millis(100));
}

/// 创建测试用的MQTT消息
fn create_test_message(device_id: &str, point_count: usize) -> MqttMessage {
    let points = (0..point_count).map(|i| {
        DataPoint {
            tag: format!("sensor_{}", i),
            value: serde_json::Value::Number(serde_json::Number::from_f64(i as f64 * 0.5).unwrap()),
            quality: 2,
            meta: {
                let mut meta = HashMap::new();
                meta.insert("index".to_string(), i.to_string());
                meta
            },
        }
    }).collect();

    MqttMessage {
        device_id: device_id.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        points,
    }
}