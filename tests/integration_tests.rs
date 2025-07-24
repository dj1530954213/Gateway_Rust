//! 集成测试主入口文件
//! 
//! 运行命令：
//! cargo test --test integration_tests -- --test-threads=1

mod integration;

use integration::common::DockerHelper;

/// 集成测试前置检查
#[tokio::test]
async fn test_prerequisites() {
    println!("🔧 检查集成测试前置条件...");
    
    // 检查Docker环境
    assert!(
        DockerHelper::check_requirements().is_ok(),
        "Docker环境不可用，请确保Docker正在运行"
    );
    
    println!("✅ 前置条件检查通过");
}

// 重新导出测试模块
pub use integration::e2e::*;