//! 通用测试工具和辅助函数

pub mod mock_plc;
pub mod test_env;
pub mod performance_monitor;
pub mod mqtt_helper;

use std::time::Duration;
use tokio::time::timeout;

/// 等待条件成立，带超时
pub async fn wait_for_condition<F, Fut>(
    mut condition: F,
    timeout_duration: Duration,
) -> anyhow::Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let result = timeout(timeout_duration, async {
        loop {
            if condition().await {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await;
    
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow::anyhow!("Condition not met within timeout")),
    }
}

/// 创建临时目录
pub fn create_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// 生成测试用的DataFrame
pub fn create_test_frame(tag: &str, value: i64) -> frame_bus::DataFrame {
    frame_bus::DataFrame::new(tag, frame_bus::Value::int(value))
}

/// 生成大量测试数据
pub fn generate_test_data(count: usize) -> Vec<frame_bus::DataFrame> {
    (0..count)
        .map(|i| create_test_frame(&format!("test.point.{:04}", i), i as i64))
        .collect()
}