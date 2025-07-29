//! routes/mod.rs —— 路由模块入口
//!
//! 子模块：
//! - rules: 报警规则管理API
//! - events: 报警事件查询API
//! - channels: 通知通道管理API
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use crate::AlertEngine;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

pub mod rules;
pub mod events;
pub mod channels;

// 重新导出主要类型
pub use rules::{CreateRuleRequest, UpdateRuleRequest, RuleQueryParams, RuleListResponse};
pub use events::{EventQueryParams, EventListResponse, EventStatsResponse};
pub use channels::{CreateChannelRequest, UpdateChannelRequest, ChannelQueryParams, ChannelListResponse};

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub alert_engine: Arc<AlertEngine>,
}

/// 创建主路由
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

/// 健康检查
async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Health check requested");
    
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "alert-engine",
        "timestamp": chrono::Utc::now()
    })))
}

/// 就绪检查
async fn readiness_check(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Readiness check requested");
    
    // 检查数据库连接
    match sqlx::query("SELECT 1").execute(&state.db_pool).await {
        Ok(_) => {
            // 检查报警引擎状态
            let engine_ready = state.alert_engine.is_ready();
            
            if engine_ready {
                Ok(Json(serde_json::json!({
                    "status": "ready",
                    "database": "connected",
                    "alert_engine": "ready",
                    "timestamp": chrono::Utc::now()
                })))
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
            }
        }
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// Prometheus指标
async fn metrics_handler() -> Result<String, StatusCode> {
    debug!("Metrics requested");
    
    // 这里应该返回Prometheus格式的指标
    // 实际实现需要与prometheus库集成
    Ok("# TYPE alert_engine_up gauge\nalert_engine_up 1\n".to_string())
}

/// 系统统计信息
async fn stats_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Stats requested");
    
    match state.alert_engine.get_statistics().await {
        Ok(stats) => Ok(Json(serde_json::to_value(stats).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}