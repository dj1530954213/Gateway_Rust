// 简单的REST API测试
use std::sync::Arc;
use crate::auth::AuthManager;
use crate::handlers::{CommandHandler, HealthHandler};
use crate::server::{ApiServer, ServerConfig};
use frame_bus::command::CommandProcessor;
use frame_bus::permissions::PermissionManager;

/// 创建测试用的API服务器实例
pub fn create_test_server() -> ApiServer {
    // 创建必要的组件
    let (tx, _rx) = tokio::sync::broadcast::channel(1024);
    let command_processor = Arc::new(CommandProcessor::new(tx));
    
    let permission_manager = Arc::new(PermissionManager::new());
    let auth_manager = Arc::new(AuthManager::new("test-secret", permission_manager));
    
    let command_handler = Arc::new(CommandHandler::new(command_processor.clone()));
    let health_handler = Arc::new(HealthHandler::new(command_processor));
    
    let config = ServerConfig {
        bind_addr: "127.0.0.1:0".parse().unwrap(), // 使用随机端口
        ..Default::default()
    };
    
    ApiServer::new(config, auth_manager, command_handler, health_handler)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let _server = create_test_server();
        // 如果能成功创建服务器实例，说明基本组件都能正常工作
    }

    #[tokio::test]
    async fn test_health_handler() {
        let _server = create_test_server();
        // 测试健康检查处理器是否能正常工作
        // 这里只是确保不会panic
    }
}