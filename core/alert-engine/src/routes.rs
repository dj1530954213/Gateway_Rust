//! routes.rs —— 报警引擎HTTP API路由
//!
//! API端点：
//! - /rules: 报警规则CRUD
//! - /events: 报警事件查询
//! - /channels: 通知通道管理
//! - /stats: 统计信息
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

pub mod rules;
pub mod events;
pub mod channels;

use axum::{
    routing::get,
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use crate::engine::AlertEngine;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub alert_engine: Arc<AlertEngine>,
}

/// 创建API路由
pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/metrics", get(metrics_handler))
        .route("/stats", get(stats_handler))
        .nest("/rules", rules::create_routes())
        .nest("/events", events::create_routes())
        .nest("/channels", channels::create_routes())
        .with_state(state)
}

/// 健康检查端点
async fn health_check(
    axum::extract::State(state): axum::extract::State<AppState>
) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
    // 检查数据库连接
    let db_status = match sqlx::query("SELECT 1").execute(&state.db_pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };
    
    // 检查报警引擎状态
    let engine_status = if state.alert_engine.is_running() {
        "running"
    } else {
        "stopped"
    };
    
    let response = serde_json::json!({
        "status": if db_status == "healthy" && engine_status == "running" { "healthy" } else { "unhealthy" },
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "components": {
            "database": db_status,
            "alert_engine": engine_status
        }
    });
    
    Ok(axum::Json(response))
}

/// 就绪检查端点
async fn readiness_check(
    axum::extract::State(state): axum::extract::State<AppState>
) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
    let ready = state.alert_engine.is_ready();
    
    let response = serde_json::json!({
        "ready": ready,
        "timestamp": chrono::Utc::now()
    });
    
    if ready {
        Ok(axum::Json(response))
    } else {
        Err(axum::http::StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// 指标端点
async fn metrics_handler() -> String {
    "# Metrics are exposed on the configured metrics port\n".to_string()
}

/// 统计信息端点
async fn stats_handler(
    axum::extract::State(state): axum::extract::State<AppState>
) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
    match state.alert_engine.get_statistics().await {
        Ok(stats) => Ok(axum::Json(serde_json::to_value(stats).unwrap_or_default())),
        Err(e) => {
            tracing::error!("Failed to get statistics: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}