//! DriverManager驱动注册和生命周期管理测试

use driver_manager::{Driver, DriverMeta, DriverKind, DriverState, DriverManager, register_static_driver};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration, timeout};

/// 测试用的Mock驱动
struct MockDriver {
    name: String,
    initialized: Arc<AtomicBool>,
    connected: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    read_count: Arc<AtomicU32>,
    should_fail_init: bool,
    should_fail_connect: bool,
}

impl MockDriver {
    fn new(name: String) -> Self {
        Self {
            name,
            initialized: Arc::new(AtomicBool::new(false)),
            connected: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(false)),
            read_count: Arc::new(AtomicU32::new(0)),
            should_fail_init: false,
            should_fail_connect: false,
        }
    }

    fn with_init_failure(mut self) -> Self {
        self.should_fail_init = true;
        self
    }

    fn with_connect_failure(mut self) -> Self {
        self.should_fail_connect = true;
        self
    }

    fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Relaxed)
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn read_count(&self) -> u32 {
        self.read_count.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl Driver for MockDriver {
    fn meta(&self) -> DriverMeta {
        DriverMeta {
            name: self.name.clone(),
            kind: DriverKind::Static,
            version: "1.0.0".to_string(),
            api_version: 1,
            description: "Mock driver for testing".to_string(),
            features: vec!["read".to_string()],
        }
    }

    async fn init(&mut self, _cfg: &Value) -> anyhow::Result<()> {
        if self.should_fail_init {
            return Err(anyhow::anyhow!("Mock init failure"));
        }
        
        self.initialized.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn connect(&mut self, _pool: std::sync::Arc<endpoint_kit::EndpointHandle>) -> anyhow::Result<()> {
        if self.should_fail_connect {
            return Err(anyhow::anyhow!("Mock connect failure"));
        }
        
        self.connected.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn read_loop(&mut self, _tx: frame_bus::FrameSender) -> anyhow::Result<()> {
        self.running.store(true, Ordering::Relaxed);
        
        // 模拟读取循环
        let mut count = 0;
        while self.running.load(Ordering::Relaxed) && count < 10 {
            count += 1;
            self.read_count.store(count, Ordering::Relaxed);
            sleep(Duration::from_millis(10)).await;
        }
        
        Ok(())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }
}

// 创建Mock驱动工厂函数
fn create_mock_driver() -> Box<dyn Driver> {
    Box::new(MockDriver::new("mock_driver".to_string()))
}

fn create_failing_driver() -> Box<dyn Driver> {
    Box::new(MockDriver::new("failing_driver".to_string()).with_init_failure())
}

// 注册测试驱动
register_static_driver!("mock_driver", create_mock_driver);
register_static_driver!("failing_driver", create_failing_driver);

#[tokio::test]
async fn test_driver_manager_creation() {
    let manager = DriverManager::new().expect("Failed to create DriverManager");
    
    // 验证初始状态
    let drivers = manager.list_drivers().await;
    assert!(drivers.is_empty(), "Manager should start with no drivers");
}

#[tokio::test]
async fn test_static_driver_loading() {
    let manager = DriverManager::new().expect("Failed to create DriverManager");
    
    // 加载静态驱动
    let config = serde_json::json!({
        "endpoint": "tcp://127.0.0.1:502",
        "poll_interval": 1000
    });
    
    let result = manager.load_static_driver(
        "test_driver_1".to_string(),
        "mock_driver",
        config
    ).await;
    
    assert!(result.is_ok(), "Should load static driver successfully");
    
    // 验证驱动已加载
    let drivers = manager.list_drivers().await;
    assert_eq!(drivers.len(), 1);
    
    let (id, meta, state) = &drivers[0];
    assert_eq!(id, "test_driver_1");
    assert_eq!(meta.name, "mock_driver");
    assert_eq!(meta.kind, DriverKind::Static);
    assert_eq!(*state, DriverState::Init);
}

#[tokio::test]
async fn test_driver_state_transitions() {
    let manager = DriverManager::new().expect("Failed to create DriverManager");
    
    let config = serde_json::json!({});
    
    // 加载驱动
    manager.load_static_driver(
        "state_test".to_string(),
        "mock_driver",
        config
    ).await.expect("Failed to load driver");
    
    // 检查初始状态
    let state = manager.get_driver_state("state_test").await;
    assert_eq!(state, Some(DriverState::Init));
    
    // 启动驱动
    manager.start_driver("state_test").await.expect("Failed to start driver");
    
    // 检查活跃状态
    let state = manager.get_driver_state("state_test").await;
    assert_eq!(state, Some(DriverState::Active));
    
    // 停止驱动
    manager.stop_driver("state_test").await.expect("Failed to stop driver");
    
    // 检查关闭状态
    let state = manager.get_driver_state("state_test").await;
    assert_eq!(state, Some(DriverState::Shutdown));
}

#[tokio::test]
async fn test_driver_not_found_errors() {
    let manager = DriverManager::new().expect("Failed to create DriverManager");
    
    // 尝试加载不存在的驱动
    let result = manager.load_static_driver(
        "nonexistent".to_string(),
        "nonexistent_driver",
        serde_json::json!({})
    ).await;
    
    assert!(result.is_err(), "Should fail to load nonexistent driver");
    assert!(result.unwrap_err().to_string().contains("not found"));
    
    // 尝试启动不存在的驱动
    let result = manager.start_driver("nonexistent").await;
    assert!(result.is_err(), "Should fail to start nonexistent driver");
    
    // 尝试停止不存在的驱动
    let result = manager.stop_driver("nonexistent").await;
    assert!(result.is_err(), "Should fail to stop nonexistent driver");
    
    // 检查不存在的驱动状态
    let state = manager.get_driver_state("nonexistent").await;
    assert_eq!(state, None);
}

#[tokio::test]
async fn test_driver_initialization_failure() {
    let manager = DriverManager::new().expect("Failed to create DriverManager");
    
    // 尝试加载会失败的驱动
    let result = manager.load_static_driver(
        "failing_test".to_string(),
        "failing_driver",
        serde_json::json!({})
    ).await;
    
    assert!(result.is_err(), "Should fail to load driver with init failure");
    
    // 验证驱动未被添加到管理器
    let drivers = manager.list_drivers().await;
    assert!(drivers.is_empty(), "Failed driver should not be added");
}

#[tokio::test]
async fn test_multiple_drivers_management() {
    let manager = DriverManager::new().expect("Failed to create DriverManager");
    
    let config = serde_json::json!({});
    
    // 加载多个驱动
    for i in 1..=3 {
        manager.load_static_driver(
            format!("driver_{}", i),
            "mock_driver",
            config.clone()
        ).await.expect("Failed to load driver");
    }
    
    // 验证所有驱动已加载
    let drivers = manager.list_drivers().await;
    assert_eq!(drivers.len(), 3);
    
    // 启动所有驱动
    for i in 1..=3 {
        manager.start_driver(&format!("driver_{}", i)).await.expect("Failed to start driver");
    }
    
    // 验证所有驱动都在运行
    for i in 1..=3 {
        let state = manager.get_driver_state(&format!("driver_{}", i)).await;
        assert_eq!(state, Some(DriverState::Active));
    }
    
    // 停止部分驱动
    manager.stop_driver("driver_2").await.expect("Failed to stop driver");
    
    // 验证状态
    assert_eq!(manager.get_driver_state("driver_1").await, Some(DriverState::Active));
    assert_eq!(manager.get_driver_state("driver_2").await, Some(DriverState::Shutdown));
    assert_eq!(manager.get_driver_state("driver_3").await, Some(DriverState::Active));
}

#[tokio::test]
async fn test_driver_restart_prevention() {
    let manager = DriverManager::new().expect("Failed to create DriverManager");
    
    let config = serde_json::json!({});
    
    // 加载并启动驱动
    manager.load_static_driver(
        "restart_test".to_string(),
        "mock_driver",
        config
    ).await.expect("Failed to load driver");
    
    manager.start_driver("restart_test").await.expect("Failed to start driver");
    
    // 尝试再次启动同一个驱动
    let result = manager.start_driver("restart_test").await;
    assert!(result.is_err(), "Should not allow restarting already running driver");
    assert!(result.unwrap_err().to_string().contains("already started"));
}

#[tokio::test]
async fn test_manager_shutdown() {
    let manager = DriverManager::new().expect("Failed to create DriverManager");
    
    let config = serde_json::json!({});
    
    // 加载并启动多个驱动
    for i in 1..=3 {
        manager.load_static_driver(
            format!("shutdown_test_{}", i),
            "mock_driver",
            config.clone()
        ).await.expect("Failed to load driver");
        
        manager.start_driver(&format!("shutdown_test_{}", i)).await.expect("Failed to start driver");
    }
    
    // 执行全局关闭
    manager.shutdown().await.expect("Failed to shutdown manager");
    
    // 验证所有驱动都已停止
    for i in 1..=3 {
        let state = manager.get_driver_state(&format!("shutdown_test_{}", i)).await;
        assert_eq!(state, Some(DriverState::Shutdown));
    }
}

#[tokio::test]
async fn test_driver_task_lifecycle() {
    let manager = DriverManager::new().expect("Failed to create DriverManager");
    
    let config = serde_json::json!({});
    
    manager.load_static_driver(
        "task_test".to_string(),
        "mock_driver",
        config
    ).await.expect("Failed to load driver");
    
    // 启动驱动任务
    manager.start_driver("task_test").await.expect("Failed to start driver");
    
    // 等待任务开始运行
    sleep(Duration::from_millis(50)).await;
    
    // 停止驱动
    manager.stop_driver("task_test").await.expect("Failed to stop driver");
    
    // 验证任务已停止
    let state = manager.get_driver_state("task_test").await;
    assert_eq!(state, Some(DriverState::Shutdown));
}

#[tokio::test]
async fn test_concurrent_driver_operations() {
    let manager = Arc::new(DriverManager::new().expect("Failed to create DriverManager"));
    
    let config = serde_json::json!({});
    
    // 并发加载多个驱动
    let mut tasks = Vec::new();
    
    for i in 0..10 {
        let manager_clone = manager.clone();
        let config_clone = config.clone();
        
        let task = tokio::spawn(async move {
            let driver_id = format!("concurrent_test_{}", i);
            let result = manager_clone.load_static_driver(
                driver_id.clone(),
                "mock_driver",
                config_clone
            ).await;
            
            if result.is_ok() {
                let _ = manager_clone.start_driver(&driver_id).await;
                sleep(Duration::from_millis(10)).await;
                let _ = manager_clone.stop_driver(&driver_id).await;
            }
            
            result
        });
        
        tasks.push(task);
    }
    
    // 等待所有任务完成
    let mut success_count = 0;
    for task in tasks {
        let result = task.await.expect("Task panicked");
        if result.is_ok() {
            success_count += 1;
        }
    }
    
    // 验证至少大部分操作成功
    assert!(success_count >= 8, "Most concurrent operations should succeed");
    
    // 清理
    manager.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
async fn test_driver_metadata_consistency() {
    let manager = DriverManager::new().expect("Failed to create DriverManager");
    
    let config = serde_json::json!({});
    
    manager.load_static_driver(
        "meta_test".to_string(),
        "mock_driver",
        config
    ).await.expect("Failed to load driver");
    
    let drivers = manager.list_drivers().await;
    assert_eq!(drivers.len(), 1);
    
    let (id, meta, _state) = &drivers[0];
    assert_eq!(id, "meta_test");
    assert_eq!(meta.name, "mock_driver");
    assert_eq!(meta.kind, DriverKind::Static);
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(meta.api_version, 1);
    assert_eq!(meta.description, "Mock driver for testing");
    assert_eq!(meta.features, vec!["read"]);
}

#[tokio::test]
async fn test_graceful_stop_behavior() {
    let manager = DriverManager::new().expect("Failed to create DriverManager");
    
    let config = serde_json::json!({});
    
    manager.load_static_driver(
        "graceful_test".to_string(),
        "mock_driver",
        config
    ).await.expect("Failed to load driver");
    
    manager.start_driver("graceful_test").await.expect("Failed to start driver");
    
    // 等待驱动开始运行
    sleep(Duration::from_millis(50)).await;
    
    // 测试优雅停止
    let stop_start = std::time::Instant::now();
    manager.stop_driver("graceful_test").await.expect("Failed to stop driver");
    let stop_duration = stop_start.elapsed();
    
    // 停止应该很快完成（因为我们abort了任务）
    assert!(stop_duration < Duration::from_millis(100), "Stop should be fast");
    
    let state = manager.get_driver_state("graceful_test").await;
    assert_eq!(state, Some(DriverState::Shutdown));
}