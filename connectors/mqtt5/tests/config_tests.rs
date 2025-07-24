//! MQTT配置测试

use mqtt5::config::{MqttCfg, BatchCfg, CompressionCfg, TlsCfg, MqttMessage, DataPoint};
use std::time::Duration;
use std::collections::HashMap;
use serde_json::Value;

#[test]
fn test_mqtt_config_defaults() {
    let cfg = MqttCfg::default();
    
    assert_eq!(cfg.broker, "tcp://localhost:1883");
    assert_eq!(cfg.client_id, "");
    assert_eq!(cfg.username, "");
    assert_eq!(cfg.password, "");
    assert_eq!(cfg.qos, 2);
    assert_eq!(cfg.topic_prefix, "gateway");
    assert_eq!(cfg.keep_alive, Duration::from_secs(60));
    assert_eq!(cfg.timeout, Duration::from_secs(10));
    assert_eq!(cfg.reconnect, Duration::from_secs(5));
    assert_eq!(cfg.buffer_size, 10000);
    assert!(!cfg.compression.enabled);
    assert!(!cfg.tls.enabled);
}

#[test]
fn test_batch_config_defaults() {
    let batch_cfg = BatchCfg::default();
    
    assert_eq!(batch_cfg.size, 100);
    assert_eq!(batch_cfg.timeout, Duration::from_millis(500));
}

#[test]
fn test_compression_config_defaults() {
    let comp_cfg = CompressionCfg::default();
    
    assert!(!comp_cfg.enabled);
    assert_eq!(comp_cfg.level, 3);
    assert_eq!(comp_cfg.threshold, 1024);
}

#[test]
fn test_tls_config_defaults() {
    let tls_cfg = TlsCfg::default();
    
    assert!(!tls_cfg.enabled);
    assert_eq!(tls_cfg.server_name, "");
    assert_eq!(tls_cfg.cert_path, "");
    assert_eq!(tls_cfg.key_path, "");
    assert_eq!(tls_cfg.ca_path, "");
    assert!(tls_cfg.verify_cert);
}

#[test]
fn test_mqtt_config_serialization() {
    let cfg = MqttCfg {
        broker: "tcp://example.com:1883".to_string(),
        client_id: "test-client".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        qos: 1,
        topic_prefix: "test".to_string(),
        keep_alive: Duration::from_secs(30),
        timeout: Duration::from_secs(5),
        reconnect: Duration::from_secs(2),
        batch: BatchCfg {
            size: 50,
            timeout: Duration::from_millis(200),
        },
        compression: CompressionCfg {
            enabled: true,
            level: 5,
            threshold: 512,
        },
        buffer_size: 5000,
        tls: TlsCfg {
            enabled: true,
            server_name: "example.com".to_string(),
            cert_path: "/path/to/cert".to_string(),
            key_path: "/path/to/key".to_string(),
            ca_path: "/path/to/ca".to_string(),
            verify_cert: false,
        },
    };

    // 序列化为JSON
    let json = serde_json::to_string(&cfg).expect("Failed to serialize config");
    assert!(json.contains("tcp://example.com:1883"));
    assert!(json.contains("test-client"));

    // 反序列化
    let deserialized: MqttCfg = serde_json::from_str(&json).expect("Failed to deserialize config");
    assert_eq!(deserialized.broker, cfg.broker);
    assert_eq!(deserialized.client_id, cfg.client_id);
    assert_eq!(deserialized.qos, cfg.qos);
    assert_eq!(deserialized.compression.enabled, cfg.compression.enabled);
    assert_eq!(deserialized.tls.enabled, cfg.tls.enabled);
}

#[test]
fn test_mqtt_config_yaml_serialization() {
    let cfg = MqttCfg {
        broker: "tcp://test-broker:1883".to_string(),
        client_id: "yaml-test".to_string(),
        ..Default::default()
    };

    // 序列化为YAML
    let yaml = serde_yaml::to_string(&cfg).expect("Failed to serialize to YAML");
    assert!(yaml.contains("tcp://test-broker:1883"));
    assert!(yaml.contains("yaml-test"));

    // 反序列化
    let deserialized: MqttCfg = serde_yaml::from_str(&yaml).expect("Failed to deserialize from YAML");
    assert_eq!(deserialized.broker, cfg.broker);
    assert_eq!(deserialized.client_id, cfg.client_id);
}

#[test]
fn test_mqtt_message_creation() {
    let mut meta = HashMap::new();
    meta.insert("source".to_string(), "test_driver".to_string());
    meta.insert("location".to_string(), "room_1".to_string());

    let point = DataPoint {
        tag: "temperature".to_string(),
        value: Value::Number(serde_json::Number::from_f64(23.5).unwrap()),
        quality: 2,
        meta: meta.clone(),
    };

    let message = MqttMessage {
        device_id: "device-001".to_string(),
        timestamp: 1640995200000, // 2022-01-01 00:00:00 UTC
        points: vec![point.clone()],
    };

    assert_eq!(message.device_id, "device-001");
    assert_eq!(message.timestamp, 1640995200000);
    assert_eq!(message.points.len(), 1);
    assert_eq!(message.points[0].tag, "temperature");
    assert_eq!(message.points[0].quality, 2);
    assert_eq!(message.points[0].meta, meta);
}

#[test]
fn test_mqtt_message_serialization() {
    let mut meta = HashMap::new();
    meta.insert("driver".to_string(), "modbus".to_string());

    let points = vec![
        DataPoint {
            tag: "sensor.temp".to_string(),
            value: Value::Number(serde_json::Number::from_f64(25.3).unwrap()),
            quality: 2,
            meta: meta.clone(),
        },
        DataPoint {
            tag: "sensor.humidity".to_string(),
            value: Value::Number(serde_json::Number::from_f64(65.2).unwrap()),
            quality: 2,
            meta: meta.clone(),
        },
    ];

    let message = MqttMessage {
        device_id: "gateway-001".to_string(),
        timestamp: 1640995200000,
        points,
    };

    // 序列化为JSON
    let json = serde_json::to_string(&message).expect("Failed to serialize message");
    assert!(json.contains("gateway-001"));
    assert!(json.contains("sensor.temp"));
    assert!(json.contains("sensor.humidity"));
    assert!(json.contains("25.3"));
    assert!(json.contains("65.2"));

    // 反序列化
    let deserialized: MqttMessage = serde_json::from_str(&json).expect("Failed to deserialize message");
    assert_eq!(deserialized.device_id, message.device_id);
    assert_eq!(deserialized.timestamp, message.timestamp);
    assert_eq!(deserialized.points.len(), message.points.len());
    assert_eq!(deserialized.points[0].tag, message.points[0].tag);
    assert_eq!(deserialized.points[1].tag, message.points[1].tag);
}

#[test]
fn test_data_point_value_types() {
    let test_cases = vec![
        ("bool_value", Value::Bool(true)),
        ("int_value", Value::Number(42.into())),
        ("float_value", Value::Number(serde_json::Number::from_f64(3.14159).unwrap())),
        ("string_value", Value::String("test".to_string())),
        ("null_value", Value::Null),
    ];

    for (tag, value) in test_cases {
        let point = DataPoint {
            tag: tag.to_string(),
            value: value.clone(),
            quality: 2,
            meta: HashMap::new(),
        };

        // 验证值类型正确
        match (&value, &point.value) {
            (Value::Bool(a), Value::Bool(b)) => assert_eq!(a, b),
            (Value::Number(a), Value::Number(b)) => assert_eq!(a, b),
            (Value::String(a), Value::String(b)) => assert_eq!(a, b),
            (Value::Null, Value::Null) => {},
            _ => panic!("Value type mismatch for {}", tag),
        }
    }
}

#[test]
fn test_config_with_partial_values() {
    // 测试部分配置值的反序列化
    let partial_json = r#"{
        "broker": "tcp://custom-broker:1883",
        "qos": 1,
        "batch": {
            "size": 50
        },
        "compression": {
            "enabled": true
        }
    }"#;

    let cfg: MqttCfg = serde_json::from_str(partial_json).expect("Failed to parse partial config");
    
    assert_eq!(cfg.broker, "tcp://custom-broker:1883");
    assert_eq!(cfg.qos, 1);
    assert_eq!(cfg.client_id, ""); // 默认值
    assert_eq!(cfg.batch.size, 50);
    assert_eq!(cfg.batch.timeout, Duration::from_millis(500)); // 默认值
    assert!(cfg.compression.enabled);
    assert_eq!(cfg.compression.level, 3); // 默认值
}

#[test]
fn test_humantime_serde_format() {
    // 测试人类可读时间格式的序列化/反序列化
    let yaml_config = r#"
broker: "tcp://localhost:1883"
keep_alive: "30s"
timeout: "5s"
reconnect: "1s"
batch:
  timeout: "200ms"
"#;

    let cfg: MqttCfg = serde_yaml::from_str(yaml_config).expect("Failed to parse humantime config");
    
    assert_eq!(cfg.keep_alive, Duration::from_secs(30));
    assert_eq!(cfg.timeout, Duration::from_secs(5));
    assert_eq!(cfg.reconnect, Duration::from_secs(1));
    assert_eq!(cfg.batch.timeout, Duration::from_millis(200));
}

#[test]
fn test_invalid_config_values() {
    // 测试无效配置的处理
    let invalid_qos_json = r#"{"broker": "tcp://localhost:1883", "qos": 5}"#;
    let cfg: MqttCfg = serde_json::from_str(invalid_qos_json).expect("Should parse even with invalid QoS");
    assert_eq!(cfg.qos, 5); // 配置解析不验证值的有效性，由使用者验证

    // 测试空字符串处理
    let empty_broker_json = r#"{"broker": ""}"#;
    let cfg: MqttCfg = serde_json::from_str(empty_broker_json).expect("Should parse empty broker");
    assert_eq!(cfg.broker, "");
}

#[test]
fn test_config_clone() {
    let original = MqttCfg {
        broker: "tcp://test:1883".to_string(),
        client_id: "test-clone".to_string(),
        ..Default::default()
    };

    let cloned = original.clone();
    
    assert_eq!(original.broker, cloned.broker);
    assert_eq!(original.client_id, cloned.client_id);
    assert_eq!(original.qos, cloned.qos);
}

#[test]
fn test_config_debug_format() {
    let cfg = MqttCfg::default();
    let debug_string = format!("{:?}", cfg);
    
    // 验证Debug输出包含关键字段
    assert!(debug_string.contains("MqttCfg"));
    assert!(debug_string.contains("broker"));
    assert!(debug_string.contains("tcp://localhost:1883"));
}