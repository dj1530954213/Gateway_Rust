//! routes/events.rs —— 报警事件查询API
//!
//! 端点列表：
//! - GET /events: 查询报警事件列表
//! - GET /events/{id}: 获取报警事件详情
//! - POST /events/{id}/acknowledge: 确认报警事件
//! - POST /events/{id}/resolve: 手动解决报警事件
//! - GET /events/stats: 获取事件统计信息
//! - GET /events/firing: 获取正在触发的事件
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::models::{AlertEvent, AlertLevel, AlertEventStatus};
use crate::routes::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// 创建事件路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_events))
        .route("/:id", get(get_event))
        .route("/:id/acknowledge", post(acknowledge_event))
        .route("/:id/resolve", post(resolve_event))
        .route("/stats", get(get_event_stats))
        .route("/firing", get(get_firing_events))
}

/// 事件查询参数
#[derive(Debug, Deserialize)]
pub struct EventQueryParams {
    /// 页码（从1开始）
    pub page: Option<u32>,
    /// 每页大小
    pub size: Option<u32>,
    /// 按规则名称搜索
    pub rule_name: Option<String>,
    /// 按级别过滤
    pub level: Option<AlertLevel>,
    /// 按状态过滤
    pub status: Option<AlertEventStatus>,
    /// 按设备ID过滤
    pub device_id: Option<Uuid>,
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 排序字段
    pub sort: Option<String>,
    /// 排序方向 (asc/desc)
    pub order: Option<String>,
}

/// 事件列表响应
#[derive(Debug, Serialize)]
pub struct EventListResponse {
    /// 事件列表
    pub items: Vec<AlertEvent>,
    /// 总数
    pub total: u64,
    /// 当前页
    pub page: u32,
    /// 每页大小
    pub size: u32,
    /// 总页数
    pub total_pages: u32,
}

/// 事件统计响应
#[derive(Debug, Serialize)]
pub struct EventStatsResponse {
    /// 总事件数
    pub total_events: u64,
    /// 正在触发的事件数
    pub firing_events: u64,
    /// 已解决的事件数
    pub resolved_events: u64,
    /// 已确认的事件数
    pub acknowledged_events: u64,
    /// 按级别统计
    pub by_level: HashMap<AlertLevel, u64>,
    /// 按状态统计
    pub by_status: HashMap<AlertEventStatus, u64>,
    /// 今日新增事件数
    pub today_count: u64,
    /// 本周新增事件数
    pub week_count: u64,
    /// 本月新增事件数
    pub month_count: u64,
    /// 平均响应时间（分钟）
    pub avg_response_time_minutes: f64,
    /// 统计时间
    pub generated_at: DateTime<Utc>,
}

/// 查询报警事件列表
async fn list_events(
    State(state): State<AppState>,
    Query(params): Query<EventQueryParams>,
) -> Result<Json<EventListResponse>, StatusCode> {
    debug!("Listing alert events with params: {:?}", params);
    
    let page = params.page.unwrap_or(1).max(1);
    let size = params.size.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * size;
    
    // 构建查询条件
    let mut where_clauses = Vec::new();
    let mut query_params: Vec<Box<dyn sqlx::postgres::PgArgumentValue + Send + Sync>> = Vec::new();
    let mut param_count = 0;
    
    if let Some(rule_name) = &params.rule_name {
        param_count += 1;
        where_clauses.push(format!("rule_name ILIKE ${}", param_count));
        query_params.push(Box::new(format!("%{}%", rule_name)));
    }
    
    if let Some(level) = &params.level {
        param_count += 1;
        where_clauses.push(format!("level = ${}", param_count));
        query_params.push(Box::new(level.clone()));
    }
    
    if let Some(status) = &params.status {
        param_count += 1;
        where_clauses.push(format!("status = ${}", param_count));
        query_params.push(Box::new(status.clone()));
    }
    
    if let Some(device_id) = params.device_id {
        param_count += 1;
        where_clauses.push(format!("device_id = ${}", param_count));
        query_params.push(Box::new(device_id));
    }
    
    if let Some(start_time) = params.start_time {
        param_count += 1;
        where_clauses.push(format!("fired_at >= ${}", param_count));
        query_params.push(Box::new(start_time));
    }
    
    if let Some(end_time) = params.end_time {
        param_count += 1;
        where_clauses.push(format!("fired_at <= ${}", param_count));
        query_params.push(Box::new(end_time));
    }
    
    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };
    
    // 构建排序
    let sort_field = params.sort.as_deref().unwrap_or("fired_at");
    let sort_order = params.order.as_deref().unwrap_or("desc");
    let order_clause = format!("ORDER BY {} {}", sort_field, sort_order);
    
    // 查询总数
    let count_query = format!("SELECT COUNT(*) FROM alert_events {}", where_clause);
    let total = sqlx::query_scalar(&count_query)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to count alert events: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // 查询事件列表
    let list_query = format!(
        r#"
        SELECT 
            id, rule_id, rule_name, device_id, tag_id,
            fired_at, resolved_at, value, threshold,
            level as "level: AlertLevel", status as "status: AlertEventStatus",
            message, context, notification_status
        FROM alert_events 
        {} {} 
        LIMIT ${} OFFSET ${}
        "#,
        where_clause, order_clause, param_count + 1, param_count + 2
    );
    
    query_params.push(Box::new(size as i64));
    query_params.push(Box::new(offset as i64));
    
    let events = sqlx::query_as::<_, AlertEvent>(&list_query)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to query alert events: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let total_pages = ((total as u32) + size - 1) / size;
    
    let response = EventListResponse {
        items: events,
        total: total as u64,
        page,
        size,
        total_pages,
    };
    
    Ok(Json(response))
}

/// 获取报警事件详情
async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AlertEvent>, StatusCode> {
    debug!("Getting alert event: {}", id);
    
    let event = sqlx::query_as!(
        AlertEvent,
        r#"
        SELECT 
            id, rule_id, rule_name, device_id, tag_id,
            fired_at, resolved_at, value, threshold,
            level as "level: AlertLevel", status as "status: AlertEventStatus",
            message, context, notification_status
        FROM alert_events 
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to query alert event: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    match event {
        Some(event) => Ok(Json(event)),
        None => {
            debug!("Alert event not found: {}", id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// 确认报警事件
async fn acknowledge_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Acknowledging alert event: {}", id);
    
    // 检查事件是否存在且状态为firing
    let existing = sqlx::query!(
        "SELECT status FROM alert_events WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to check existing event: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let current_status = match existing {
        Some(row) => row.status,
        None => {
            debug!("Alert event not found for acknowledgment: {}", id);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    
    // 只有firing状态的事件可以被确认
    if current_status != AlertEventStatus::Firing as _ {
        warn!("Cannot acknowledge event with status: {:?}", current_status);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 更新事件状态
    let result = sqlx::query!(
        "UPDATE alert_events SET status = $1 WHERE id = $2",
        AlertEventStatus::Acknowledged as AlertEventStatus,
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to acknowledge alert event: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // 通知报警引擎
    if let Err(e) = state.alert_engine.acknowledge_event(id).await {
        warn!("Failed to notify alert engine about acknowledgment: {}", e);
    }
    
    info!("Alert event acknowledged successfully: {}", id);
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Event acknowledged successfully",
        "event_id": id,
        "timestamp": Utc::now()
    })))
}

/// 手动解决报警事件
async fn resolve_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Resolving alert event: {}", id);
    
    // 检查事件是否存在
    let existing = sqlx::query!(
        "SELECT status FROM alert_events WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to check existing event: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if existing.is_none() {
        debug!("Alert event not found for resolution: {}", id);
        return Err(StatusCode::NOT_FOUND);
    }
    
    // 更新事件状态为已解决
    let now = Utc::now();
    let result = sqlx::query!(
        "UPDATE alert_events SET status = $1, resolved_at = $2 WHERE id = $3",
        AlertEventStatus::Resolved as AlertEventStatus,
        now,
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to resolve alert event: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // 通知报警引擎
    if let Err(e) = state.alert_engine.resolve_event(id).await {
        warn!("Failed to notify alert engine about resolution: {}", e);
    }
    
    info!("Alert event resolved successfully: {}", id);
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Event resolved successfully",
        "event_id": id,
        "resolved_at": now
    })))
}

/// 获取事件统计信息
async fn get_event_stats(
    State(state): State<AppState>,
) -> Result<Json<EventStatsResponse>, StatusCode> {
    debug!("Getting alert event statistics");
    
    let now = Utc::now();
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    let week_start = now - chrono::Duration::days(7);
    let month_start = now - chrono::Duration::days(30);
    
    // 查询总事件数
    let total_events = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM alert_events"
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to count total events: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(0) as u64;
    
    // 查询各状态事件数
    let firing_events = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM alert_events WHERE status = $1",
        AlertEventStatus::Firing as AlertEventStatus
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to count firing events: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(0) as u64;
    
    let resolved_events = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM alert_events WHERE status = $1",
        AlertEventStatus::Resolved as AlertEventStatus
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to count resolved events: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(0) as u64;
    
    let acknowledged_events = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM alert_events WHERE status = $1",
        AlertEventStatus::Acknowledged as AlertEventStatus
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to count acknowledged events: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(0) as u64;
    
    // 按级别统计
    let level_stats = sqlx::query!(
        "SELECT level, COUNT(*) as count FROM alert_events GROUP BY level"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to get level stats: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let mut by_level = HashMap::new();
    for row in level_stats {
        let level: AlertLevel = row.level;
        by_level.insert(level, row.count.unwrap_or(0) as u64);
    }
    
    // 按状态统计
    let status_stats = sqlx::query!(
        "SELECT status, COUNT(*) as count FROM alert_events GROUP BY status"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to get status stats: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let mut by_status = HashMap::new();
    for row in status_stats {
        let status: AlertEventStatus = row.status;
        by_status.insert(status, row.count.unwrap_or(0) as u64);
    }
    
    // 时间段统计
    let today_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM alert_events WHERE fired_at >= $1",
        today_start
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to count today's events: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(0) as u64;
    
    let week_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM alert_events WHERE fired_at >= $1",
        week_start
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to count week's events: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(0) as u64;
    
    let month_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM alert_events WHERE fired_at >= $1",
        month_start
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to count month's events: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(0) as u64;
    
    // 计算平均响应时间（已解决事件的响应时间）
    let avg_response_time_minutes = sqlx::query_scalar!(
        r#"
        SELECT AVG(EXTRACT(EPOCH FROM (resolved_at - fired_at)) / 60.0)
        FROM alert_events 
        WHERE resolved_at IS NOT NULL
        "#
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to calculate average response time: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(0.0);
    
    let response = EventStatsResponse {
        total_events,
        firing_events,
        resolved_events,
        acknowledged_events,
        by_level,
        by_status,
        today_count,
        week_count,
        month_count,
        avg_response_time_minutes,
        generated_at: now,
    };
    
    Ok(Json(response))
}

/// 获取正在触发的事件
async fn get_firing_events(
    State(state): State<AppState>,
) -> Result<Json<Vec<AlertEvent>>, StatusCode> {
    debug!("Getting firing alert events");
    
    // 首先尝试从alert engine获取
    let firing_events = state.alert_engine.get_firing_events().await;
    
    if !firing_events.is_empty() {
        return Ok(Json(firing_events));
    }
    
    // 如果engine中没有，从数据库查询
    let events = sqlx::query_as!(
        AlertEvent,
        r#"
        SELECT 
            id, rule_id, rule_name, device_id, tag_id,
            fired_at, resolved_at, value, threshold,
            level as "level: AlertLevel", status as "status: AlertEventStatus",
            message, context, notification_status
        FROM alert_events 
        WHERE status = $1 
        ORDER BY fired_at DESC
        "#,
        AlertEventStatus::Firing as AlertEventStatus
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to query firing events: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(events))
}