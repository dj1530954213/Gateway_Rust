//! 通用测试工具和辅助函数
//! 
//! 🚫 重要提示：本系统为生产级产品，测试必须使用真实设备和真实数据

pub mod test_env;

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

// 🚫 模拟数据生成函数已移除
// 
// 原有的create_test_frame()和generate_test_data()函数已删除，
// 因为本系统为生产级产品，必须使用真实设备的真实数据进行测试。
//
// 如需测试，请连接真实的工业设备和传感器。