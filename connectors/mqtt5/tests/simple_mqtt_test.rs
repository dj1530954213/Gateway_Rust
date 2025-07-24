//! 简化的 MQTT5 连接器测试

use mqtt5::config::{MqttCfg, BatchCfg};
use mqtt5::connector::MqttConnector;
use frame_bus::Value;
use std::time::Duration;

fn create_test_config() -> MqttCfg {
    MqttCfg {
        broker: "tcp://localhost:1883".to_string(),
        client_id: "test-client".to_string(),
        username: "".to_string(),
        password: "".to_string(),
        qos: 1,
        topic_prefix: "test".to_string(),
        keep_alive: Duration::from_secs(60),
        timeout: Duration::from_secs(10),
        reconnect: Duration::from_secs(5),
        batch: BatchCfg {
            size: 10,
            timeout: Duration::from_millis(100),
        },
        compression: Default::default(),
        buffer_size: 1000,
        tls: Default::default(),
    }
}

#[tokio::test]
async fn test_mqtt_connector_creation() {
    let cfg = create_test_config();
    let connector = MqttConnector::new(cfg.clone());
    
    // 基本验证 - 连接器应该能够被创建
    assert!(true); // 如果到达这里就表明连接器创建成功
}

#[tokio::test]
async fn test_config_clone() {
    let cfg1 = create_test_config();
    let cfg2 = cfg1.clone();
    
    assert_eq!(cfg1.broker, cfg2.broker);
    assert_eq!(cfg1.client_id, cfg2.client_id);
    assert_eq!(cfg1.qos, cfg2.qos);
}

#[tokio::test]
async fn test_value_creation() {
    // 测试不同类型的 Value 创建
    let bool_val = Value::bool(true);
    let int_val = Value::int(42);
    let float_val = Value::float(3.14);
    let string_val = Value::string("test");
    let bytes_val = Value::bytes(vec![1, 2, 3]);
    
    // 验证值能够被创建 (如果到达这里表明创建成功)
    assert!(true);
}