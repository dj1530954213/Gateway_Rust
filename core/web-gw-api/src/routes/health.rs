//! routes/health.rs —— 健康检查路由
//!
//! 提供服务健康状态检查接口：
//! - GET /health: 基础健康检查
//! - GET /health/ready: 就绪探针
//! - GET /health/live: 存活探针
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::bootstrap::AppState;
use crate::dto::HealthResponse;
use crate::error::ApiResult;
use actix_web::{web, HttpResponse, Scope};
use chrono::Utc;
use tracing::info;
use utoipa::ToSchema;

/// 健康检查路由作用域
pub fn scope() -> Scope {
    web::scope("/health")
        .route("", web::get().to(health_check))
        .route("/ready", web::get().to(readiness_check))
        .route("/live", web::get().to(liveness_check))
}

/// 基础健康检查
/// 
/// 检查所有依赖服务的状态
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = HealthResponse)
    ),
    tag = "Health"
)]
async fn health_check(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
    info!("Health check requested");
    
    let services = state.health_check().await;
    let all_healthy = services.values().all(|status| status == "healthy");
    
    let response = HealthResponse {
        status: if all_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        services,
    };
    
    if all_healthy {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(response))
    }
}

/// 就绪探针
/// 
/// 检查服务是否准备好接受请求
#[utoipa::path(
    get,
    path = "/health/ready",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Service is not ready")
    ),
    tag = "Health"
)]
async fn readiness_check(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
    let services = state.health_check().await;
    
    // 检查关键服务：PostgreSQL必须健康
    let postgres_healthy = services.get("postgres")
        .map(|status| status == "healthy")
        .unwrap_or(false);
    
    if postgres_healthy {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "timestamp": Utc::now()
        })))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "timestamp": Utc::now(),
            "reason": "PostgreSQL not available"
        })))
    }
}

/// 存活探针
/// 
/// 简单检查服务是否还在运行
#[utoipa::path(
    get,
    path = "/health/live",
    responses(
        (status = 200, description = "Service is alive")
    ),
    tag = "Health"
)]
async fn liveness_check() -> ApiResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "timestamp": Utc::now()
    })))
}