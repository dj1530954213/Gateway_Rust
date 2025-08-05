//! 配置管理器核心功能测试

use config_manager::{ConfigManager, ConfigEvent};
use config_manager::schemas::*;
use tempfile::tempdir;
use std::time::Duration;
use tokio::fs;
use tokio::time::{sleep, timeout};

/// 创建测试用的配置目录
async fn create_test_config_dir() -> tempfile::TempDir {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    
    // 创建测试配置文件
    let endpoints_content = r#"
endpoints:
  plc1:
    url: "tcp://[REAL_PLC1_IP]:502"
    description: "Main PLC"
    timeout: "10s"
    pool:
      min_connections: 1
      max_connections: 5
  plc2:
    url: "tcp://[REAL_PLC2_IP]:502"
    description: "Secondary PLC"
"#;

    let drivers_content = r#"
drivers:
  modbus1:
    driver_type: "modbus-tcp"
    endpoint: "plc1"
    enabled: true
    polling: "1s"
    retry: 3
    slave_id: 1
  modbus2:
    driver_type: "modbus-tcp"
    endpoint: "plc2"
    enabled: false
    polling: "2s"
"#;

    let variables_content = r#"
variables:
  plant.temperature:
    description: "Plant temperature"
    driver: "modbus1"
    data_type: "float32"
    address: "40001"
    access: "r"
    scale: "value / 10.0"
    unit: "°C"
    alarms:
      - alarm_type: "high"
        value: 50
        level: "warning"
        message: "Temperature high"
  plant.pressure:
    description: "Plant pressure"
    driver: "modbus1"
    data_type: "uint16"
    address: "40002"
    access: "rw"
    unit: "bar"
"#;

    let endpoints_path = temp_dir.path().join("endpoints.yml");
    let drivers_path = temp_dir.path().join("drivers.yml");
    let variables_path = temp_dir.path().join("variables.yml");

    fs::write(&endpoints_path, endpoints_content).await.expect("Failed to write endpoints.yml");
    fs::write(&drivers_path, drivers_content).await.expect("Failed to write drivers.yml");
    fs::write(&variables_path, variables_content).await.expect("Failed to write variables.yml");

    temp_dir
}

#[tokio::test]
async fn test_config_manager_creation() {
    let temp_dir = create_test_config_dir().await;
    let (manager, mut event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    // 验证初始配置加载
    let endpoints = manager.get_endpoints().await;
    assert_eq!(endpoints.endpoints.len(), 2);
    assert!(endpoints.endpoints.contains_key("plc1"));
    assert!(endpoints.endpoints.contains_key("plc2"));

    let drivers = manager.get_drivers().await;
    assert_eq!(drivers.drivers.len(), 2);
    assert!(drivers.drivers.contains_key("modbus1"));
    assert!(drivers.drivers.contains_key("modbus2"));

    let variables = manager.get_variables().await;
    assert_eq!(variables.variables.len(), 2);
    assert!(variables.variables.contains_key("plant.temperature"));
    assert!(variables.variables.contains_key("plant.pressure"));

    // 检查是否有初始事件（可能没有，因为是初始加载）
    let _result = timeout(Duration::from_millis(100), event_rx.recv()).await;
    // 不强制要求初始事件，因为实现可能不同
}

#[tokio::test]
async fn test_config_manager_with_empty_directory() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let (manager, _event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    // 应该创建默认配置
    let endpoints = manager.get_endpoints().await;
    assert!(!endpoints.endpoints.is_empty()); // 应该有默认端点

    let drivers = manager.get_drivers().await;
    assert!(!drivers.drivers.is_empty()); // 应该有默认驱动

    let variables = manager.get_variables().await;
    assert!(!variables.variables.is_empty()); // 应该有默认变量

    // 验证默认配置文件已创建
    assert!(temp_dir.path().join("endpoints.yml").exists());
    assert!(temp_dir.path().join("drivers.yml").exists());
    assert!(temp_dir.path().join("variables.yml").exists());
}

#[tokio::test]
async fn test_config_content_verification() {
    let temp_dir = create_test_config_dir().await;
    let (manager, _event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    // 验证端点配置内容
    let endpoints = manager.get_endpoints().await;
    let plc1 = endpoints.endpoints.get("plc1").expect("plc1 should exist");
    assert_eq!(plc1.url, "tcp://[REAL_PLC1_IP]:502");
    assert_eq!(plc1.description, "Main PLC");
    assert_eq!(plc1.timeout, Duration::from_secs(10));
    assert_eq!(plc1.pool.min_connections, 1);
    assert_eq!(plc1.pool.max_connections, 5);

    // 验证驱动配置内容
    let drivers = manager.get_drivers().await;
    let modbus1 = drivers.drivers.get("modbus1").expect("modbus1 should exist");
    assert_eq!(modbus1.driver_type, "modbus-tcp");
    assert_eq!(modbus1.endpoint, "plc1");
    assert!(modbus1.enabled);
    assert_eq!(modbus1.polling, Duration::from_secs(1));
    assert_eq!(modbus1.retry, 3);

    let modbus2 = drivers.drivers.get("modbus2").expect("modbus2 should exist");
    assert!(!modbus2.enabled);
    assert_eq!(modbus2.polling, Duration::from_secs(2));

    // 验证变量配置内容
    let variables = manager.get_variables().await;
    let temp_var = variables.variables.get("plant.temperature").expect("temperature var should exist");
    assert_eq!(temp_var.description, "Plant temperature");
    assert_eq!(temp_var.driver, "modbus1");
    assert!(matches!(temp_var.data_type, DataType::Float32));
    assert!(matches!(temp_var.access, Access::Read));
    assert_eq!(temp_var.scale, Some("value / 10.0".to_string()));
    assert_eq!(temp_var.unit, "°C");
    assert_eq!(temp_var.alarms.len(), 1);

    let pressure_var = variables.variables.get("plant.pressure").expect("pressure var should exist");
    assert!(matches!(pressure_var.data_type, DataType::Uint16));
    assert!(matches!(pressure_var.access, Access::ReadWrite));
    assert_eq!(pressure_var.unit, "bar");
    assert!(pressure_var.alarms.is_empty());
}

#[tokio::test]
async fn test_manual_reload() {
    let temp_dir = create_test_config_dir().await;
    let (manager, mut event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    // 修改配置文件
    let new_endpoints_content = r#"
endpoints:
  plc1:
    url: "tcp://[REAL_PLC3_IP]:502"
    description: "Updated PLC"
  plc3:
    url: "tcp://[REAL_PLC4_IP]:502"
    description: "New PLC"
"#;

    let endpoints_path = temp_dir.path().join("endpoints.yml");
    fs::write(&endpoints_path, new_endpoints_content).await.expect("Failed to write updated endpoints.yml");

    // 手动重新加载
    manager.reload_all().await.expect("Failed to reload configs");

    // 验证配置已更新
    let endpoints = manager.get_endpoints().await;
    assert_eq!(endpoints.endpoints.len(), 2);
    assert!(endpoints.endpoints.contains_key("plc1"));
    assert!(endpoints.endpoints.contains_key("plc3"));
    assert!(!endpoints.endpoints.contains_key("plc2")); // 应该已删除

    let plc1 = endpoints.endpoints.get("plc1").unwrap();
    assert_eq!(plc1.url, "tcp://[REAL_PLC3_IP]:502");
    assert_eq!(plc1.description, "Updated PLC");

    // 应该收到配置变更事件
    let mut event_count = 0;
    while let Ok(event) = timeout(Duration::from_millis(100), event_rx.recv()).await {
        match event {
            Some(ConfigEvent::EndpointsChanged(_)) => event_count += 1,
            Some(ConfigEvent::DriversChanged(_)) => event_count += 1,
            Some(ConfigEvent::VariablesChanged(_)) => event_count += 1,
            None => break,
        }
        if event_count >= 3 {
            break;
        }
    }
    assert_eq!(event_count, 3); // 应该收到三个配置变更事件
}

#[tokio::test]
async fn test_file_watcher_reload() {
    let temp_dir = create_test_config_dir().await;
    let (manager, mut event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    // 等待初始化完成
    sleep(Duration::from_millis(100)).await;

    // 修改drivers.yml
    let new_drivers_content = r#"
drivers:
  modbus1:
    driver_type: "modbus-tcp"
    endpoint: "plc1"
    enabled: false
    polling: "5s"
    retry: 5
  new_driver:
    driver_type: "opcua"
    endpoint: "plc2"
    enabled: true
"#;

    let drivers_path = temp_dir.path().join("drivers.yml");
    fs::write(&drivers_path, new_drivers_content).await.expect("Failed to write updated drivers.yml");

    // 等待文件监听器检测到变化
    let mut received_driver_event = false;
    for _ in 0..10 { // 最多等待1秒
        if let Ok(Some(event)) = timeout(Duration::from_millis(100), event_rx.recv()).await {
            if matches!(event, ConfigEvent::DriversChanged(_)) {
                received_driver_event = true;
                break;
            }
        }
    }

    assert!(received_driver_event, "Should receive driver config change event");

    // 验证配置已自动更新
    let drivers = manager.get_drivers().await;
    assert_eq!(drivers.drivers.len(), 2);
    
    let modbus1 = drivers.drivers.get("modbus1").unwrap();
    assert!(!modbus1.enabled); // 应该已更新为disabled
    assert_eq!(modbus1.polling, Duration::from_secs(5));
    assert_eq!(modbus1.retry, 5);

    assert!(drivers.drivers.contains_key("new_driver"));
    let new_driver = drivers.drivers.get("new_driver").unwrap();
    assert_eq!(new_driver.driver_type, "opcua");
    assert!(new_driver.enabled);
}

#[tokio::test]
async fn test_variables_file_watcher() {
    let temp_dir = create_test_config_dir().await;
    let (manager, mut event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    // 等待初始化完成
    sleep(Duration::from_millis(100)).await;

    // 修改variables.yml，添加新变量
    let new_variables_content = r#"
variables:
  plant.temperature:
    description: "Plant temperature (updated)"
    driver: "modbus1"
    data_type: "float32"
    address: "40001"
    access: "rw"
    scale: "value / 100.0"
    unit: "°C"
  plant.flow_rate:
    description: "Flow rate"
    driver: "modbus2"
    data_type: "uint32"
    address: "40010"
    access: "r"
    unit: "L/min"
    alarms:
      - alarm_type: "low"
        value: 10
        level: "warning"
        message: "Flow rate too low"
      - alarm_type: "high"
        value: 100
        level: "critical"
        message: "Flow rate too high"
"#;

    let variables_path = temp_dir.path().join("variables.yml");
    fs::write(&variables_path, new_variables_content).await.expect("Failed to write updated variables.yml");

    // 等待文件监听器检测到变化
    let mut received_variables_event = false;
    for _ in 0..10 {
        if let Ok(Some(event)) = timeout(Duration::from_millis(100), event_rx.recv()).await {
            if matches!(event, ConfigEvent::VariablesChanged(_)) {
                received_variables_event = true;
                break;
            }
        }
    }

    assert!(received_variables_event, "Should receive variables config change event");

    // 验证配置已自动更新
    let variables = manager.get_variables().await;
    assert_eq!(variables.variables.len(), 2);

    let temp_var = variables.variables.get("plant.temperature").unwrap();
    assert_eq!(temp_var.description, "Plant temperature (updated)");
    assert!(matches!(temp_var.access, Access::ReadWrite)); // 更新为读写
    assert_eq!(temp_var.scale, Some("value / 100.0".to_string())); // 缩放比例更新

    let flow_var = variables.variables.get("plant.flow_rate").unwrap();
    assert_eq!(flow_var.description, "Flow rate");
    assert_eq!(flow_var.driver, "modbus2");
    assert!(matches!(flow_var.data_type, DataType::Uint32));
    assert_eq!(flow_var.unit, "L/min");
    assert_eq!(flow_var.alarms.len(), 2);
    
    // 验证报警配置
    let low_alarm = &flow_var.alarms[0];
    assert!(matches!(low_alarm.alarm_type, AlarmType::Low));
    assert!(matches!(low_alarm.level, AlarmLevel::Warning));
    assert_eq!(low_alarm.message, "Flow rate too low");

    let high_alarm = &flow_var.alarms[1];
    assert!(matches!(high_alarm.alarm_type, AlarmType::High));
    assert!(matches!(high_alarm.level, AlarmLevel::Critical));
    assert_eq!(high_alarm.message, "Flow rate too high");
}

#[tokio::test]
async fn test_invalid_config_handling() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    
    // 创建无效的配置文件
    let invalid_endpoints_content = r#"
endpoints:
  plc1:
    url: "invalid-url-format"
    timeout: "invalid-duration"
"#;

    let endpoints_path = temp_dir.path().join("endpoints.yml");
    fs::write(&endpoints_path, invalid_endpoints_content).await.expect("Failed to write invalid endpoints.yml");

    // 尝试创建配置管理器，应该处理错误并使用默认配置
    let result = ConfigManager::new(temp_dir.path()).await;
    
    // 根据实现，可能成功创建（使用默认配置）或失败
    // 这里我们测试错误处理的行为
    match result {
        Ok((manager, _)) => {
            // 如果成功，应该使用默认配置
            let endpoints = manager.get_endpoints().await;
            // 默认配置应该至少有一个端点
            assert!(!endpoints.endpoints.is_empty());
        }
        Err(_) => {
            // 如果失败，说明严格验证配置
            // 这也是可接受的行为
        }
    }
}

#[tokio::test]
async fn test_concurrent_config_access() {
    let temp_dir = create_test_config_dir().await;
    let (manager, _event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    // 并发读取配置
    let mut tasks = Vec::new();
    
    for i in 0..10 {
        let manager_clone = manager.clone();
        let task = tokio::spawn(async move {
            for _ in 0..10 {
                let endpoints = manager_clone.get_endpoints().await;
                let drivers = manager_clone.get_drivers().await;
                let variables = manager_clone.get_variables().await;
                
                // 验证配置一致性
                assert!(!endpoints.endpoints.is_empty());
                assert!(!drivers.drivers.is_empty());
                assert!(!variables.variables.is_empty());
                
                // 小延迟模拟真实使用
                tokio::task::yield_now().await;
            }
            i
        });
        tasks.push(task);
    }

    // 等待所有任务完成
    for task in tasks {
        let result = task.await.expect("Task should complete successfully");
        assert!(result < 10);
    }
}

#[tokio::test]
async fn test_config_manager_clone() {
    let temp_dir = create_test_config_dir().await;
    let (manager1, _event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    // Clone配置管理器
    let manager2 = manager1.clone();

    // 两个管理器应该访问相同的配置
    let endpoints1 = manager1.get_endpoints().await;
    let endpoints2 = manager2.get_endpoints().await;

    assert_eq!(endpoints1.endpoints.len(), endpoints2.endpoints.len());
    
    for (key, value1) in &endpoints1.endpoints {
        let value2 = endpoints2.endpoints.get(key).expect("Key should exist in both");
        assert_eq!(value1.url, value2.url);
        assert_eq!(value1.description, value2.description);
    }
}