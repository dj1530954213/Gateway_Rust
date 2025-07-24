//! 静态驱动注册表测试

use driver_manager::{Driver, DriverMeta, DriverKind, StaticDriverRegistry, register_static_driver};
use async_trait::async_trait;
use serde_json::Value;

/// 测试驱动1
struct TestDriver1 {
    id: u32,
}

#[async_trait]
impl Driver for TestDriver1 {
    fn meta(&self) -> DriverMeta {
        DriverMeta {
            name: "test_driver_1".to_string(),
            kind: DriverKind::Static,
            version: "1.0.0".to_string(),
            api_version: 1,
            description: "Test driver 1".to_string(),
            features: vec!["read".to_string()],
        }
    }

    async fn init(&mut self, _cfg: &Value) -> anyhow::Result<()> {
        Ok(())
    }

    async fn connect(&mut self, _pool: std::sync::Arc<endpoint_kit::EndpointHandle>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn read_loop(&mut self, _tx: frame_bus::FrameSender) -> anyhow::Result<()> {
        Ok(())
    }
}

/// 测试驱动2
struct TestDriver2 {
    id: u32,
}

#[async_trait]
impl Driver for TestDriver2 {
    fn meta(&self) -> DriverMeta {
        DriverMeta {
            name: "test_driver_2".to_string(),
            kind: DriverKind::Static,
            version: "2.0.0".to_string(),
            api_version: 1,
            description: "Test driver 2".to_string(),
            features: vec!["read".to_string(), "write".to_string()],
        }
    }

    async fn init(&mut self, _cfg: &Value) -> anyhow::Result<()> {
        Ok(())
    }

    async fn connect(&mut self, _pool: std::sync::Arc<endpoint_kit::EndpointHandle>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn read_loop(&mut self, _tx: frame_bus::FrameSender) -> anyhow::Result<()> {
        Ok(())
    }

    async fn write(&mut self, _cmd: frame_bus::CmdFrame) -> anyhow::Result<()> {
        Ok(())
    }
}

// 工厂函数
fn create_test_driver_1() -> Box<dyn Driver> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    Box::new(TestDriver1 { id })
}

fn create_test_driver_2() -> Box<dyn Driver> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    Box::new(TestDriver2 { id })
}

// 注册测试驱动
register_static_driver!("test_driver_1", create_test_driver_1);
register_static_driver!("test_driver_2", create_test_driver_2);

#[test]
fn test_registry_creation() {
    let registry = StaticDriverRegistry::new();
    
    // 获取已注册的驱动列表
    let drivers = registry.list();
    
    // 应该至少包含我们注册的测试驱动
    assert!(drivers.len() >= 2, "Registry should contain at least 2 drivers");
    assert!(drivers.contains(&"test_driver_1".to_string()));
    assert!(drivers.contains(&"test_driver_2".to_string()));
}

#[test]
fn test_driver_factory_retrieval() {
    let registry = StaticDriverRegistry::new();
    
    // 获取已注册的驱动工厂
    let factory1 = registry.get("test_driver_1");
    assert!(factory1.is_some(), "Should find test_driver_1");
    
    let factory2 = registry.get("test_driver_2");
    assert!(factory2.is_some(), "Should find test_driver_2");
    
    // 尝试获取不存在的驱动
    let non_existent = registry.get("non_existent_driver");
    assert!(non_existent.is_none(), "Should not find non-existent driver");
}

#[test]
fn test_driver_factory_functionality() {
    let registry = StaticDriverRegistry::new();
    
    // 获取并调用工厂函数
    let factory1 = registry.get("test_driver_1").expect("Should find test_driver_1");
    let driver1 = factory1();
    let meta1 = driver1.meta();
    
    assert_eq!(meta1.name, "test_driver_1");
    assert_eq!(meta1.version, "1.0.0");
    assert_eq!(meta1.features, vec!["read"]);
    
    let factory2 = registry.get("test_driver_2").expect("Should find test_driver_2");
    let driver2 = factory2();
    let meta2 = driver2.meta();
    
    assert_eq!(meta2.name, "test_driver_2");
    assert_eq!(meta2.version, "2.0.0");
    assert_eq!(meta2.features, vec!["read", "write"]);
}

#[test]
fn test_multiple_registry_instances() {
    // 创建多个注册表实例
    let registry1 = StaticDriverRegistry::new();
    let registry2 = StaticDriverRegistry::new();
    
    // 它们应该包含相同的驱动
    let drivers1 = registry1.list();
    let drivers2 = registry2.list();
    
    assert_eq!(drivers1.len(), drivers2.len());
    
    for driver_name in &drivers1 {
        assert!(drivers2.contains(driver_name));
    }
}

#[test]
fn test_registry_driver_list_consistency() {
    let registry = StaticDriverRegistry::new();
    
    let drivers = registry.list();
    
    // 验证列表中的每个驱动都可以通过get获取
    for driver_name in &drivers {
        let factory = registry.get(driver_name);
        assert!(factory.is_some(), "Driver {} should be retrievable", driver_name);
    }
}

#[test]
fn test_case_sensitive_driver_names() {
    let registry = StaticDriverRegistry::new();
    
    // 驱动名称应该是大小写敏感的
    let factory = registry.get("test_driver_1");
    assert!(factory.is_some());
    
    let factory_upper = registry.get("TEST_DRIVER_1");
    assert!(factory_upper.is_none(), "Driver names should be case sensitive");
    
    let factory_mixed = registry.get("Test_Driver_1");
    assert!(factory_mixed.is_none(), "Driver names should be case sensitive");
}

#[test]
fn test_driver_factory_creates_unique_instances() {
    let registry = StaticDriverRegistry::new();
    
    let factory = registry.get("test_driver_1").expect("Should find test_driver_1");
    
    // 创建多个驱动实例
    let driver1 = factory();
    let driver2 = factory();
    
    // 验证它们是不同的实例（通过地址比较）
    let ptr1 = driver1.as_ref() as *const dyn Driver;
    let ptr2 = driver2.as_ref() as *const dyn Driver;
    
    assert_ne!(ptr1, ptr2, "Factory should create unique instances");
    
    // 但它们应该有相同的元信息
    assert_eq!(driver1.meta().name, driver2.meta().name);
}

/// 测试注册表是否正确处理空名称或特殊字符
#[test]
fn test_special_driver_names() {
    let registry = StaticDriverRegistry::new();
    
    // 测试不存在的特殊名称
    assert!(registry.get("").is_none());
    assert!(registry.get(" ").is_none());
    assert!(registry.get("non-existent-driver").is_none());
}