//! 配置热重载功能测试

use config_manager::{ConfigManager, ConfigEvent};
use config_manager::schemas::*;
use tempfile::tempdir;
use std::time::Duration;
use tokio::fs;
use tokio::time::{sleep, timeout};

/// 创建基础配置文件
async fn create_basic_config(temp_dir: &std::path::Path) {
    let endpoints_content = r#"
endpoints:
  device1:
    url: "tcp://[REAL_DEVICE_IP]:502"
    description: "Device 1"
"#;

    let drivers_content = r#"
drivers:
  driver1:
    driver_type: "modbus-tcp"
    endpoint: "device1"
    enabled: true
"#;

    let variables_content = r#"
variables:
  sensor1:
    description: "Sensor 1"
    driver: "driver1"
    data_type: "int16"
    address: "40001"
"#;

    fs::write(temp_dir.join("endpoints.yml"), endpoints_content).await.unwrap();
    fs::write(temp_dir.join("drivers.yml"), drivers_content).await.unwrap();
    fs::write(temp_dir.join("variables.yml"), variables_content).await.unwrap();
}

#[tokio::test]
async fn test_single_file_hotreload() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    create_basic_config(temp_dir.path()).await;

    let (manager, mut event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    // 等待初始化完成
    sleep(Duration::from_millis(100)).await;

    // 修改endpoints.yml
    let updated_content = r#"
endpoints:
  device1:
    url: "tcp://[REAL_DEVICE2_IP]:502"
    description: "Updated Device 1"
    timeout: "15s"
  device2:
    url: "tcp://[REAL_DEVICE3_IP]:502"
    description: "New Device 2"
"#;

    fs::write(temp_dir.path().join("endpoints.yml"), updated_content).await.unwrap();

    // 等待热重载事件
    let mut received_event = false;
    for _ in 0..20 { // 最多等待2秒
        if let Ok(Some(event)) = timeout(Duration::from_millis(100), event_rx.recv()).await {
            if let ConfigEvent::EndpointsChanged(endpoints_config) = event {
                received_event = true;
                
                // 验证事件中的配置
                assert_eq!(endpoints_config.endpoints.len(), 2);
                assert!(endpoints_config.endpoints.contains_key("device1"));
                assert!(endpoints_config.endpoints.contains_key("device2"));
                
                let device1 = &endpoints_config.endpoints["device1"];
                assert_eq!(device1.url, "tcp://[REAL_DEVICE2_IP]:502");
                assert_eq!(device1.description, "Updated Device 1");
                assert_eq!(device1.timeout, Duration::from_secs(15));
                
                break;
            }
        }
    }

    assert!(received_event, "Should receive endpoints changed event");

    // 验证管理器中的配置已更新
    let current_endpoints = manager.get_endpoints().await;
    assert_eq!(current_endpoints.endpoints.len(), 2);
    assert_eq!(current_endpoints.endpoints["device1"].url, "tcp://[REAL_DEVICE2_IP]:502");
}

#[tokio::test]
async fn test_multiple_file_hotreload() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    create_basic_config(temp_dir.path()).await;

    let (manager, mut event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    sleep(Duration::from_millis(100)).await;

    // 同时修改多个配置文件
    let new_drivers_content = r#"
drivers:
  driver1:
    driver_type: "modbus-tcp"
    endpoint: "device1"
    enabled: false
    polling: "2s"
  driver2:
    driver_type: "opcua"
    endpoint: "device2"
    enabled: true
"#;

    let new_variables_content = r#"
variables:
  sensor1:
    description: "Updated Sensor 1"
    driver: "driver1"
    data_type: "float32"
    address: "40001"
    scale: "value * 0.1"
  sensor2:
    description: "New Sensor 2"
    driver: "driver2"
    data_type: "bool"
    address: "ns=2;i=1001"
"#;

    // 写入文件，中间有小延迟
    fs::write(temp_dir.path().join("drivers.yml"), new_drivers_content).await.unwrap();
    sleep(Duration::from_millis(50)).await;
    fs::write(temp_dir.path().join("variables.yml"), new_variables_content).await.unwrap();

    // 收集事件
    let mut drivers_changed = false;
    let mut variables_changed = false;
    
    for _ in 0..20 {
        if let Ok(Some(event)) = timeout(Duration::from_millis(100), event_rx.recv()).await {
            match event {
                ConfigEvent::DriversChanged(drivers_config) => {
                    drivers_changed = true;
                    assert_eq!(drivers_config.drivers.len(), 2);
                    assert!(!drivers_config.drivers["driver1"].enabled);
                    assert_eq!(drivers_config.drivers["driver1"].polling, Duration::from_secs(2));
                    assert_eq!(drivers_config.drivers["driver2"].driver_type, "opcua");
                }
                ConfigEvent::VariablesChanged(variables_config) => {
                    variables_changed = true;
                    assert_eq!(variables_config.variables.len(), 2);
                    assert_eq!(variables_config.variables["sensor1"].description, "Updated Sensor 1");
                    assert!(matches!(variables_config.variables["sensor1"].data_type, DataType::Float32));
                    assert_eq!(variables_config.variables["sensor1"].scale, Some("value * 0.1".to_string()));
                }
                _ => {}
            }

            if drivers_changed && variables_changed {
                break;
            }
        }
    }

    assert!(drivers_changed, "Should receive drivers changed event");
    assert!(variables_changed, "Should receive variables changed event");

    // 验证配置已同步更新
    let current_drivers = manager.get_drivers().await;
    let current_variables = manager.get_variables().await;

    assert_eq!(current_drivers.drivers.len(), 2);
    assert_eq!(current_variables.variables.len(), 2);
    assert!(!current_drivers.drivers["driver1"].enabled);
    assert!(matches!(current_variables.variables["sensor1"].data_type, DataType::Float32));
}

#[tokio::test]
async fn test_rapid_file_changes() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    create_basic_config(temp_dir.path()).await;

    let (manager, mut event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    sleep(Duration::from_millis(100)).await;

    // 快速连续修改同一个文件
    for i in 1..=5 {
        let content = format!(r#"
endpoints:
  device1:
    url: "tcp://[REAL_DEVICE_IP_{}]:502"
    description: "Device iteration {}"
"#, 10 + i, i);

        fs::write(temp_dir.path().join("endpoints.yml"), content).await.unwrap();
        sleep(Duration::from_millis(20)).await; // 短暂延迟
    }

    // 收集所有事件
    let mut event_count = 0;
    let mut _last_url = String::new();

    for _ in 0..50 { // 最多等待5秒
        if let Ok(Some(event)) = timeout(Duration::from_millis(100), event_rx.recv()).await {
            if let ConfigEvent::EndpointsChanged(endpoints_config) = event {
                event_count += 1;
                _last_url = endpoints_config.endpoints["device1"].url.clone();
            }
        } else {
            break; // 超时，停止等待
        }
    }

    // 应该收到至少一个事件，可能由于防抖或事件合并，不会收到所有5个事件
    assert!(event_count >= 1, "Should receive at least one event");

    // 验证最终状态是最新的
    let final_endpoints = manager.get_endpoints().await;
    assert!(final_endpoints.endpoints["device1"].url.contains("[REAL_DEVICE_IP]")); // 应该是最新的值
}

#[tokio::test]
async fn test_file_deletion_and_recreation() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    create_basic_config(temp_dir.path()).await;

    let (manager, mut event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    sleep(Duration::from_millis(100)).await;

    // 删除配置文件
    fs::remove_file(temp_dir.path().join("endpoints.yml")).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // 重新创建配置文件
    let new_content = r#"
endpoints:
  recreated_device:
    url: "tcp://[REAL_DEVICE_99_IP]:502"
    description: "Recreated Device"
"#;

    fs::write(temp_dir.path().join("endpoints.yml"), new_content).await.unwrap();

    // 等待重载事件
    let mut received_recreation_event = false;
    for _ in 0..20 {
        if let Ok(Some(event)) = timeout(Duration::from_millis(100), event_rx.recv()).await {
            if let ConfigEvent::EndpointsChanged(endpoints_config) = event {
                if endpoints_config.endpoints.contains_key("recreated_device") {
                    received_recreation_event = true;
                    assert_eq!(endpoints_config.endpoints["recreated_device"].url, "tcp://[REAL_DEVICE_99_IP]:502");
                    break;
                }
            }
        }
    }

    assert!(received_recreation_event, "Should detect file recreation");

    // 验证配置已更新
    let current_endpoints = manager.get_endpoints().await;
    assert!(current_endpoints.endpoints.contains_key("recreated_device"));
}

#[tokio::test]
async fn test_invalid_config_hotreload() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    create_basic_config(temp_dir.path()).await;

    let (manager, mut event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    sleep(Duration::from_millis(100)).await;

    // 获取当前有效配置
    let original_endpoints = manager.get_endpoints().await;

    // 写入无效配置
    let invalid_content = r#"
endpoints:
  device1:
    url: "invalid-yaml-content
    description: "Missing quote above
"#;

    fs::write(temp_dir.path().join("endpoints.yml"), invalid_content).await.unwrap();

    // 等待一段时间，看是否收到事件
    let mut received_invalid_event = false;
    for _ in 0..10 {
        if let Ok(Some(_event)) = timeout(Duration::from_millis(100), event_rx.recv()).await {
            // 如果收到事件，检查是否为有效配置变更
            received_invalid_event = true;
            break;
        }
    }

    // 无效配置不应该触发配置变更事件，或者应该保持原有配置
    let current_endpoints = manager.get_endpoints().await;
    
    // 配置应该保持不变或恢复为有效状态
    // 具体行为取决于实现：可能保持原配置，可能恢复默认配置
    if received_invalid_event {
        // 如果收到事件，配置应该是有效的
        assert!(!current_endpoints.endpoints.is_empty());
    } else {
        // 如果没收到事件，配置应该保持原来的有效状态
        assert_eq!(current_endpoints.endpoints.len(), original_endpoints.endpoints.len());
    }
}

#[tokio::test]
async fn test_hotreload_with_complex_config() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    
    // 创建复杂的初始配置
    let complex_endpoints = r#"
endpoints:
  plc_main:
    url: "tcp://[REAL_PLC1_IP]:502"
    description: "Main PLC"
    timeout: "10s"
    pool:
      min_connections: 2
      max_connections: 8
      idle_timeout: "5m"
      max_lifetime: "1h"
    tls:
      server_name: "plc.factory.com"
      verify_cert: true
  hmi_station:
    url: "tcp://[REAL_PLC2_IP]:502"
    description: "HMI Station"
    timeout: "5s"
"#;

    let complex_variables = r#"
variables:
  production.line1.speed:
    description: "Production line 1 speed"
    driver: "plc_driver"
    data_type: "float32"
    address: "DB1.DBD0"
    access: "rw"
    scale: "value * 0.1"
    unit: "m/min"
    alarms:
      - alarm_type: "low"
        value: 5.0
        level: "warning"
        message: "Production speed too low"
      - alarm_type: "high"
        value: 50.0
        level: "critical"
        message: "Production speed too high"
  production.line1.temperature:
    description: "Production line 1 temperature"
    driver: "plc_driver"
    data_type: "int16"
    address: "DB1.DBW10"
    access: "r"
    scale: "value / 10.0"
    unit: "°C"
    alarms:
      - alarm_type: "highhigh"
        value: 80
        level: "critical"
        message: "Temperature critical"
"#;

    fs::write(temp_dir.path().join("endpoints.yml"), complex_endpoints).await.unwrap();
    fs::write(temp_dir.path().join("variables.yml"), complex_variables).await.unwrap();
    
    // 创建基本的drivers.yml
    let basic_drivers = r#"
drivers:
  plc_driver:
    driver_type: "modbus-tcp"
    endpoint: "plc_main"
    enabled: true
"#;
    fs::write(temp_dir.path().join("drivers.yml"), basic_drivers).await.unwrap();

    let (manager, mut event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    sleep(Duration::from_millis(100)).await;

    // 修改复杂的变量配置
    let updated_variables = r#"
variables:
  production.line1.speed:
    description: "Production line 1 speed (updated)"
    driver: "plc_driver"
    data_type: "float32"
    address: "DB1.DBD0"
    access: "rw"
    scale: "value * 0.05"
    unit: "m/s"
    alarms:
      - alarm_type: "low"
        value: 2.0
        level: "info"
        message: "Speed below optimal"
      - alarm_type: "high"
        value: 30.0
        level: "warning"
        message: "Speed above optimal"
      - alarm_type: "highhigh"
        value: 45.0
        level: "critical"
        message: "Speed dangerously high"
  production.line2.pressure:
    description: "Production line 2 pressure"
    driver: "plc_driver"
    data_type: "uint32"
    address: "DB2.DBD0"
    access: "r"
    unit: "Pa"
    alarms:
      - alarm_type: "lowlow"
        value: 1000
        level: "critical"
        message: "Pressure critically low"
"#;

    fs::write(temp_dir.path().join("variables.yml"), updated_variables).await.unwrap();

    // 等待变更事件
    let mut received_complex_update = false;
    for _ in 0..20 {
        if let Ok(Some(event)) = timeout(Duration::from_millis(100), event_rx.recv()).await {
            if let ConfigEvent::VariablesChanged(variables_config) = event {
                received_complex_update = true;
                
                // 验证复杂更新
                assert_eq!(variables_config.variables.len(), 2);
                
                let speed_var = &variables_config.variables["production.line1.speed"];
                assert_eq!(speed_var.description, "Production line 1 speed (updated)");
                assert_eq!(speed_var.scale, Some("value * 0.05".to_string()));
                assert_eq!(speed_var.unit, "m/s");
                assert_eq!(speed_var.alarms.len(), 3);
                
                let pressure_var = &variables_config.variables["production.line2.pressure"];
                assert_eq!(pressure_var.driver, "plc_driver");
                assert!(matches!(pressure_var.data_type, DataType::Uint32));
                assert_eq!(pressure_var.unit, "Pa");
                assert_eq!(pressure_var.alarms.len(), 1);
                assert!(matches!(pressure_var.alarms[0].alarm_type, AlarmType::LowLow));
                assert!(matches!(pressure_var.alarms[0].level, AlarmLevel::Critical));
                
                break;
            }
        }
    }

    assert!(received_complex_update, "Should receive complex variables update");

    // 验证管理器状态
    let final_variables = manager.get_variables().await;
    assert_eq!(final_variables.variables.len(), 2);
    assert!(final_variables.variables.contains_key("production.line1.speed"));
    assert!(final_variables.variables.contains_key("production.line2.pressure"));
}

#[tokio::test]
async fn test_hotreload_performance() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    create_basic_config(temp_dir.path()).await;

    let (manager, mut event_rx) = ConfigManager::new(temp_dir.path())
        .await
        .expect("Failed to create config manager");

    sleep(Duration::from_millis(100)).await;

    // 测试热重载性能
    let start_time = std::time::Instant::now();
    
    let updated_content = r#"
endpoints:
  performance_device:
    url: "tcp://[REAL_PLC3_IP]:502"
    description: "Performance test device"
"#;

    fs::write(temp_dir.path().join("endpoints.yml"), updated_content).await.unwrap();

    // 等待热重载完成
    let mut reload_completed = false;
    while !reload_completed {
        if let Ok(Some(event)) = timeout(Duration::from_millis(100), event_rx.recv()).await {
            if let ConfigEvent::EndpointsChanged(_) = event {
                reload_completed = true;
            }
        } else {
            break;
        }
    }

    let reload_duration = start_time.elapsed();
    
    assert!(reload_completed, "Reload should complete");
    assert!(reload_duration < Duration::from_secs(1), "Reload should be fast (< 1s), was {:?}", reload_duration);
    
    // 验证配置已更新
    let final_endpoints = manager.get_endpoints().await;
    assert!(final_endpoints.endpoints.contains_key("performance_device"));
}