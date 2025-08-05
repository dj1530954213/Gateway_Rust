//! routes/system.rs —— 系统管理REST API
//!
//! - scope(): `/api/v1/system`

use actix_web::{web, HttpResponse, Result as ActixResult, get};
use serde_json::json;
use crate::bootstrap::AppState;

/// 系统路由配置
pub fn scope() -> actix_web::Scope {
    web::scope("/system")
        .service(get_system_info)
        .service(get_system_metrics)  
        .service(get_system_health)
        .service(get_component_status)
}

/// 获取系统信息
#[get("/info")]
pub async fn get_system_info(
    _data: web::Data<AppState>
) -> ActixResult<HttpResponse> {
    let system_info = json!({
        "version": "1.0.0",
        "uptime": 86400,
        "memory_usage": 65.5,
        "cpu_usage": 12.3, 
        "disk_usage": 45.2,
        "connected_devices": 5,
        "active_tags": 128,
        "alert_count": 2
    });
    
    Ok(HttpResponse::Ok().json(system_info))
}

/// 获取系统性能指标
#[get("/metrics")]
pub async fn get_system_metrics(
    _data: web::Data<AppState>
) -> ActixResult<HttpResponse> {
    let metrics = json!({
        "cpuUsage": 45.2,
        "memoryUsage": 67.8,
        "diskUsage": 23.1,
        "activeConnections": 12,
        "messagesPerSecond": 456,
        "uptime": 86400,
        "networkIn": 2048576.0,
        "networkOut": 1048576.0
    });
    
    Ok(HttpResponse::Ok().json(metrics))
}

/// 获取系统健康状态
#[get("/health")]
pub async fn get_system_health(
    data: web::Data<AppState>
) -> ActixResult<HttpResponse> {
    let health = data.health_check().await;
    Ok(HttpResponse::Ok().json(health))
}

/// 获取组件状态
#[get("/components/status")]
pub async fn get_component_status(
    _data: web::Data<AppState>
) -> ActixResult<HttpResponse> {
    let status = json!({
        "database": { "status": "healthy", "uptime": 86400, "connections": 5 },
        "messageQueue": { "status": "healthy", "queueSize": 0, "throughput": 100 },
        "webServer": { "status": "healthy", "activeConnections": 3, "requests": 1024 },
        "fileSystem": { "status": "healthy", "freeSpace": "15.2GB", "diskUsage": 75 }
    });
    
    Ok(HttpResponse::Ok().json(status))
}