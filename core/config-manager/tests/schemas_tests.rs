//! 配置结构体测试

use config_manager::schemas::*;
use std::collections::HashMap;
use std::time::Duration;
use serde_yaml;

#[test]
fn test_endpoint_config_serialization() {
    let cfg = EndpointCfg {
        url: "tcp://192.168.1.100:502".to_string(),
        description: "Test PLC".to_string(),
        timeout: Duration::from_secs(5),
        pool: PoolCfg {
            min_connections: 1,
            max_connections: 5,
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
        },
        tls: Some(TlsCfg {
            server_name: "plc.example.com".to_string(),
            cert_path: "/path/to/cert.pem".to_string(),
            key_path: "/path/to/key.pem".to_string(),
            ca_path: "/path/to/ca.pem".to_string(),
            verify_cert: true,
        }),
        serial: None,
    };

    // 序列化为YAML
    let yaml = serde_yaml::to_string(&cfg).expect("Failed to serialize endpoint config");
    assert!(yaml.contains("tcp://192.168.1.100:502"));
    assert!(yaml.contains("Test PLC"));

    // 反序列化
    let deserialized: EndpointCfg = serde_yaml::from_str(&yaml).expect("Failed to deserialize endpoint config");
    assert_eq!(deserialized.url, cfg.url);
    assert_eq!(deserialized.description, cfg.description);
    assert_eq!(deserialized.timeout, cfg.timeout);
    assert_eq!(deserialized.pool.min_connections, cfg.pool.min_connections);
    assert!(deserialized.tls.is_some());
}

#[test]
fn test_endpoint_config_defaults() {
    let minimal_yaml = r#"
url: "tcp://localhost:502"
"#;

    let cfg: EndpointCfg = serde_yaml::from_str(minimal_yaml).expect("Failed to parse minimal config");
    
    assert_eq!(cfg.url, "tcp://localhost:502");
    assert_eq!(cfg.description, ""); // 默认空字符串
    assert_eq!(cfg.timeout, Duration::from_secs(10)); // 默认超时
    assert_eq!(cfg.pool.min_connections, 1); // 默认连接池配置
    assert_eq!(cfg.pool.max_connections, 10);
    assert!(cfg.tls.is_none());
    assert!(cfg.serial.is_none());
}

#[test]
fn test_serial_config() {
    let serial_cfg = SerialCfg {
        baud_rate: 19200,
        data_bits: 8,
        stop_bits: 1,
        parity: Parity::Even,
    };

    let yaml = serde_yaml::to_string(&serial_cfg).expect("Failed to serialize serial config");
    assert!(yaml.contains("19200"));
    assert!(yaml.contains("even"));

    let deserialized: SerialCfg = serde_yaml::from_str(&yaml).expect("Failed to deserialize serial config");
    assert_eq!(deserialized.baud_rate, 19200);
    assert_eq!(deserialized.data_bits, 8);
    assert_eq!(deserialized.stop_bits, 1);
    assert!(matches!(deserialized.parity, Parity::Even));
}

#[test]
fn test_parity_serialization() {
    let test_cases = vec![
        (Parity::None, "none"),
        (Parity::Odd, "odd"),
        (Parity::Even, "even"),
    ];

    for (parity, expected_str) in test_cases {
        let yaml = serde_yaml::to_string(&parity).expect("Failed to serialize parity");
        assert_eq!(yaml.trim(), expected_str);

        let deserialized: Parity = serde_yaml::from_str(&yaml).expect("Failed to deserialize parity");
        assert!(matches!((parity, deserialized), 
            (Parity::None, Parity::None) | 
            (Parity::Odd, Parity::Odd) | 
            (Parity::Even, Parity::Even)
        ));
    }
}

#[test]
fn test_driver_config_serialization() {
    let mut config_map = serde_yaml::Mapping::new();
    config_map.insert(
        serde_yaml::Value::String("slave_id".to_string()),
        serde_yaml::Value::Number(serde_yaml::Number::from(1))
    );
    config_map.insert(
        serde_yaml::Value::String("function_code".to_string()),
        serde_yaml::Value::Number(serde_yaml::Number::from(3))
    );

    let driver_cfg = DriverCfg {
        driver_type: "modbus-tcp".to_string(),
        endpoint: "plc1".to_string(),
        enabled: true,
        polling: Duration::from_millis(500),
        retry: 5,
        config: serde_yaml::Value::Mapping(config_map),
    };

    let yaml = serde_yaml::to_string(&driver_cfg).expect("Failed to serialize driver config");
    assert!(yaml.contains("modbus-tcp"));
    assert!(yaml.contains("plc1"));
    assert!(yaml.contains("slave_id"));

    let deserialized: DriverCfg = serde_yaml::from_str(&yaml).expect("Failed to deserialize driver config");
    assert_eq!(deserialized.driver_type, "modbus-tcp");
    assert_eq!(deserialized.endpoint, "plc1");
    assert!(deserialized.enabled);
    assert_eq!(deserialized.polling, Duration::from_millis(500));
    assert_eq!(deserialized.retry, 5);
}

#[test]
fn test_driver_config_defaults() {
    let minimal_yaml = r#"
driver_type: "test-driver"
endpoint: "test-endpoint"
"#;

    let cfg: DriverCfg = serde_yaml::from_str(minimal_yaml).expect("Failed to parse minimal driver config");
    
    assert_eq!(cfg.driver_type, "test-driver");
    assert_eq!(cfg.endpoint, "test-endpoint");
    assert!(cfg.enabled); // 默认启用
    assert_eq!(cfg.polling, Duration::from_secs(1)); // 默认轮询间隔
    assert_eq!(cfg.retry, 3); // 默认重试次数
}

#[test]
fn test_variable_config_serialization() {
    let var_cfg = VariableCfg {
        description: "Temperature sensor".to_string(),
        driver: "modbus1".to_string(),
        data_type: DataType::Float32,
        address: "40001".to_string(),
        access: Access::ReadWrite,
        scale: Some("value / 10.0".to_string()),
        unit: "°C".to_string(),
        alarms: vec![
            AlarmCfg {
                alarm_type: AlarmType::High,
                value: serde_yaml::Value::Number(serde_yaml::Number::from(50)),
                level: AlarmLevel::Warning,
                message: "Temperature too high".to_string(),
            },
            AlarmCfg {
                alarm_type: AlarmType::HighHigh,
                value: serde_yaml::Value::Number(serde_yaml::Number::from(80)),
                level: AlarmLevel::Critical,
                message: "Temperature critical".to_string(),
            },
        ],
    };

    let yaml = serde_yaml::to_string(&var_cfg).expect("Failed to serialize variable config");
    assert!(yaml.contains("Temperature sensor"));
    assert!(yaml.contains("float32"));
    assert!(yaml.contains("rw"));
    assert!(yaml.contains("high"));
    assert!(yaml.contains("critical"));

    let deserialized: VariableCfg = serde_yaml::from_str(&yaml).expect("Failed to deserialize variable config");
    assert_eq!(deserialized.description, "Temperature sensor");
    assert!(matches!(deserialized.data_type, DataType::Float32));
    assert!(matches!(deserialized.access, Access::ReadWrite));
    assert_eq!(deserialized.alarms.len(), 2);
    assert!(matches!(deserialized.alarms[0].alarm_type, AlarmType::High));
    assert!(matches!(deserialized.alarms[1].level, AlarmLevel::Critical));
}

#[test]
fn test_data_type_serialization() {
    let test_cases = vec![
        (DataType::Bool, "bool"),
        (DataType::Int16, "int16"),
        (DataType::Uint32, "uint32"),
        (DataType::Float64, "float64"),
        (DataType::String, "string"),
        (DataType::Binary, "binary"),
    ];

    for (data_type, expected_str) in test_cases {
        let yaml = serde_yaml::to_string(&data_type).expect("Failed to serialize data type");
        assert_eq!(yaml.trim(), expected_str);

        let deserialized: DataType = serde_yaml::from_str(&yaml).expect("Failed to deserialize data type");
        assert!(matches!(
            (data_type, deserialized),
            (DataType::Bool, DataType::Bool) |
            (DataType::Int16, DataType::Int16) |
            (DataType::Uint32, DataType::Uint32) |
            (DataType::Float64, DataType::Float64) |
            (DataType::String, DataType::String) |
            (DataType::Binary, DataType::Binary)
        ));
    }
}

#[test]
fn test_access_type_serialization() {
    let test_cases = vec![
        (Access::Read, "r"),
        (Access::Write, "w"),
        (Access::ReadWrite, "rw"),
    ];

    for (access, expected_str) in test_cases {
        let yaml = serde_yaml::to_string(&access).expect("Failed to serialize access type");
        assert_eq!(yaml.trim(), expected_str);

        let deserialized: Access = serde_yaml::from_str(&yaml).expect("Failed to deserialize access type");
        assert!(matches!(
            (access, deserialized),
            (Access::Read, Access::Read) |
            (Access::Write, Access::Write) |
            (Access::ReadWrite, Access::ReadWrite)
        ));
    }
}

#[test]
fn test_alarm_config_serialization() {
    let alarm_cfg = AlarmCfg {
        alarm_type: AlarmType::Low,
        value: serde_yaml::Value::Number(serde_yaml::Number::from(10)),
        level: AlarmLevel::Warning,
        message: "Value too low".to_string(),
    };

    let yaml = serde_yaml::to_string(&alarm_cfg).expect("Failed to serialize alarm config");
    assert!(yaml.contains("low"));
    assert!(yaml.contains("warning"));
    assert!(yaml.contains("Value too low"));

    let deserialized: AlarmCfg = serde_yaml::from_str(&yaml).expect("Failed to deserialize alarm config");
    assert!(matches!(deserialized.alarm_type, AlarmType::Low));
    assert!(matches!(deserialized.level, AlarmLevel::Warning));
    assert_eq!(deserialized.message, "Value too low");
}

#[test]
fn test_alarm_type_and_level_serialization() {
    let alarm_types = vec![
        (AlarmType::High, "high"),
        (AlarmType::Low, "low"),
        (AlarmType::HighHigh, "highhigh"),
        (AlarmType::LowLow, "lowlow"),
        (AlarmType::Equal, "equal"),
        (AlarmType::NotEqual, "notequal"),
    ];

    for (alarm_type, expected_str) in alarm_types {
        let yaml = serde_yaml::to_string(&alarm_type).expect("Failed to serialize alarm type");
        assert_eq!(yaml.trim(), expected_str);
    }

    let alarm_levels = vec![
        (AlarmLevel::Info, "info"),
        (AlarmLevel::Warning, "warning"),
        (AlarmLevel::Critical, "critical"),
    ];

    for (alarm_level, expected_str) in alarm_levels {
        let yaml = serde_yaml::to_string(&alarm_level).expect("Failed to serialize alarm level");
        assert_eq!(yaml.trim(), expected_str);
    }
}

#[test]
fn test_complete_config_structure() {
    // 创建完整的端点配置
    let mut endpoints = HashMap::new();
    endpoints.insert("plc1".to_string(), EndpointCfg {
        url: "tcp://192.168.1.100:502".to_string(),
        description: "Main PLC".to_string(),
        timeout: Duration::from_secs(10),
        pool: PoolCfg::default(),
        tls: None,
        serial: None,
    });

    let endpoints_config = EndpointsConfig { endpoints };

    // 创建完整的驱动配置
    let mut drivers = HashMap::new();
    drivers.insert("driver1".to_string(), DriverCfg {
        driver_type: "modbus-tcp".to_string(),
        endpoint: "plc1".to_string(),
        enabled: true,
        polling: Duration::from_secs(1),
        retry: 3,
        config: serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
    });

    let drivers_config = DriversConfig { drivers };

    // 创建完整的变量配置
    let mut variables = HashMap::new();
    variables.insert("temp1".to_string(), VariableCfg {
        description: "Temperature".to_string(),
        driver: "driver1".to_string(),
        data_type: DataType::Float32,
        address: "40001".to_string(),
        access: Access::Read,
        scale: Some("value / 10.0".to_string()),
        unit: "°C".to_string(),
        alarms: vec![],
    });

    let variables_config = VariablesConfig { variables };

    // 测试序列化和反序列化
    let endpoints_yaml = serde_yaml::to_string(&endpoints_config).unwrap();
    let drivers_yaml = serde_yaml::to_string(&drivers_config).unwrap();
    let variables_yaml = serde_yaml::to_string(&variables_config).unwrap();

    let _endpoints_restored: EndpointsConfig = serde_yaml::from_str(&endpoints_yaml).unwrap();
    let _drivers_restored: DriversConfig = serde_yaml::from_str(&drivers_yaml).unwrap();
    let _variables_restored: VariablesConfig = serde_yaml::from_str(&variables_yaml).unwrap();

    // 如果到这里没有panic，说明序列化/反序列化成功
    assert!(true);
}

#[test]
fn test_humantime_serde_integration() {
    let endpoint_yaml = r#"
url: "tcp://localhost:502"
timeout: "30s"
pool:
  idle_timeout: "5m"
  max_lifetime: "1h"
"#;

    let cfg: EndpointCfg = serde_yaml::from_str(endpoint_yaml).expect("Failed to parse humantime config");
    
    assert_eq!(cfg.timeout, Duration::from_secs(30));
    assert_eq!(cfg.pool.idle_timeout, Duration::from_secs(300)); // 5 minutes
    assert_eq!(cfg.pool.max_lifetime, Duration::from_secs(3600)); // 1 hour
}

#[test]
fn test_config_validation_scenarios() {
    // 测试无效的数据类型
    let invalid_data_type_yaml = r#"
description: "Test"
driver: "test"
data_type: "invalid_type"
address: "40001"
"#;

    let result: Result<VariableCfg, _> = serde_yaml::from_str(invalid_data_type_yaml);
    assert!(result.is_err());

    // 测试无效的访问类型
    let invalid_access_yaml = r#"
description: "Test"
driver: "test"
data_type: "int16"
address: "40001"
access: "invalid_access"
"#;

    let result: Result<VariableCfg, _> = serde_yaml::from_str(invalid_access_yaml);
    assert!(result.is_err());
}

#[test]
fn test_config_clone_and_debug() {
    let original = EndpointCfg {
        url: "tcp://test:502".to_string(),
        description: "Test endpoint".to_string(),
        timeout: Duration::from_secs(5),
        pool: PoolCfg::default(),
        tls: None,
        serial: None,
    };

    let cloned = original.clone();
    assert_eq!(original.url, cloned.url);
    assert_eq!(original.description, cloned.description);

    // 测试Debug输出
    let debug_output = format!("{:?}", original);
    assert!(debug_output.contains("EndpointCfg"));
    assert!(debug_output.contains("tcp://test:502"));
}

#[test]
fn test_default_implementations() {
    let pool_cfg = PoolCfg::default();
    assert_eq!(pool_cfg.min_connections, 1);
    assert_eq!(pool_cfg.max_connections, 10);

    let parity = Parity::default();
    assert!(matches!(parity, Parity::None));

    let access = Access::default();
    assert!(matches!(access, Access::Read));

    let alarm_level = AlarmLevel::default();
    assert!(matches!(alarm_level, AlarmLevel::Info));
}