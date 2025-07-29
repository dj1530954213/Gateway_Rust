//! routes/rules.rs —— 报警规则管理API
//!
//! 端点列表：
//! - GET /rules: 查询报警规则列表
//! - POST /rules: 创建新的报警规则
//! - GET /rules/{id}: 获取报警规则详情
//! - PUT /rules/{id}: 更新报警规则
//! - DELETE /rules/{id}: 删除报警规则
//! - POST /rules/{id}/enable: 启用报警规则
//! - POST /rules/{id}/disable: 禁用报警规则
//! - POST /rules/reload: 重新加载规则
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::models::{AlertRule, AlertLevel, CompareOperator};
use crate::routes::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// 创建规则路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_rules).post(create_rule))
        .route("/:id", get(get_rule).put(update_rule).delete(delete_rule))
        .route("/:id/enable", post(enable_rule))
        .route("/:id/disable", post(disable_rule))
        .route("/reload", post(reload_rules))
}

/// 报警规则创建请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRuleRequest {
    /// 规则名称
    pub name: String,
    /// 规则描述
    pub description: Option<String>,
    /// 目标设备ID（可选）
    pub device_id: Option<Uuid>,
    /// 目标点位ID（可选）
    pub tag_id: Option<Uuid>,
    /// 比较操作符
    pub operator: CompareOperator,
    /// 阈值
    pub threshold: f64,
    /// 报警级别
    pub level: AlertLevel,
    /// 评估间隔（秒）
    pub eval_every: Option<u64>,
    /// 持续时间（秒）
    pub eval_for: Option<u64>,
    /// 通知通道ID列表
    pub notification_channels: Option<Vec<Uuid>>,
    /// 静默期（秒）
    pub silence_duration: Option<u64>,
    /// 创建者
    pub created_by: Option<String>,
}

/// 报警规则更新请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRuleRequest {
    /// 规则名称
    pub name: Option<String>,
    /// 规则描述
    pub description: Option<String>,
    /// 目标设备ID
    pub device_id: Option<Uuid>,
    /// 目标点位ID
    pub tag_id: Option<Uuid>,
    /// 比较操作符
    pub operator: Option<CompareOperator>,
    /// 阈值
    pub threshold: Option<f64>,
    /// 报警级别
    pub level: Option<AlertLevel>,
    /// 评估间隔（秒）
    pub eval_every: Option<u64>,
    /// 持续时间（秒）
    pub eval_for: Option<u64>,
    /// 通知通道ID列表
    pub notification_channels: Option<Vec<Uuid>>,
    /// 静默期（秒）
    pub silence_duration: Option<u64>,
}

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct RuleQueryParams {
    /// 页码（从1开始）
    pub page: Option<u32>,
    /// 每页大小
    pub size: Option<u32>,
    /// 按名称搜索
    pub name: Option<String>,
    /// 按级别过滤
    pub level: Option<AlertLevel>,
    /// 按设备ID过滤
    pub device_id: Option<Uuid>,
    /// 按启用状态过滤
    pub enabled: Option<bool>,
    /// 排序字段
    pub sort: Option<String>,
    /// 排序方向 (asc/desc)
    pub order: Option<String>,
}

/// 规则列表响应
#[derive(Debug, Serialize)]
pub struct RuleListResponse {
    /// 规则列表
    pub items: Vec<AlertRule>,
    /// 总数
    pub total: u64,
    /// 当前页
    pub page: u32,
    /// 每页大小
    pub size: u32,
    /// 总页数
    pub total_pages: u32,
}

/// 查询报警规则列表
async fn list_rules(
    State(state): State<AppState>,
    Query(params): Query<RuleQueryParams>,
) -> Result<Json<RuleListResponse>, StatusCode> {
    debug!("Listing alert rules with params: {:?}", params);
    
    let page = params.page.unwrap_or(1).max(1);
    let size = params.size.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * size;
    
    // 构建查询条件
    let mut where_clauses = Vec::new();
    let mut query_params: Vec<Box<dyn sqlx::postgres::PgArgumentValue + Send + Sync>> = Vec::new();
    let mut param_count = 0;
    
    if let Some(name) = &params.name {
        param_count += 1;
        where_clauses.push(format!("name ILIKE ${}", param_count));
        query_params.push(Box::new(format!("%{}%", name)));
    }
    
    if let Some(level) = &params.level {
        param_count += 1;
        where_clauses.push(format!("level = ${}", param_count));
        query_params.push(Box::new(level.clone()));
    }
    
    if let Some(device_id) = params.device_id {
        param_count += 1;
        where_clauses.push(format!("device_id = ${}", param_count));
        query_params.push(Box::new(device_id));
    }
    
    if let Some(enabled) = params.enabled {
        param_count += 1;
        where_clauses.push(format!("enabled = ${}", param_count));
        query_params.push(Box::new(enabled));
    }
    
    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };
    
    // 构建排序
    let sort_field = params.sort.as_deref().unwrap_or("created_at");
    let sort_order = params.order.as_deref().unwrap_or("desc");
    let order_clause = format!("ORDER BY {} {}", sort_field, sort_order);
    
    // 查询总数
    let count_query = format!("SELECT COUNT(*) FROM alert_rules {}", where_clause);
    let total = sqlx::query_scalar(&count_query)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to count alert rules: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // 查询规则列表
    let list_query = format!(
        r#"
        SELECT 
            id, name, description, device_id, tag_id,
            operator as "operator: CompareOperator",
            threshold, level as "level: AlertLevel",
            eval_every, eval_for, enabled,
            notification_channels, silence_duration,
            created_by, created_at, updated_at,
            last_fired_at, fire_count
        FROM alert_rules 
        {} {} 
        LIMIT ${} OFFSET ${}
        "#,
        where_clause, order_clause, param_count + 1, param_count + 2
    );
    
    query_params.push(Box::new(size as i64));
    query_params.push(Box::new(offset as i64));
    
    let rules = sqlx::query_as::<_, AlertRule>(&list_query)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to query alert rules: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let total_pages = ((total as u32) + size - 1) / size;
    
    let response = RuleListResponse {
        items: rules,
        total: total as u64,
        page,
        size,
        total_pages,
    };
    
    Ok(Json(response))
}

/// 创建报警规则
async fn create_rule(
    State(state): State<AppState>,
    Json(request): Json<CreateRuleRequest>,
) -> Result<Json<AlertRule>, StatusCode> {
    info!("Creating new alert rule: {}", request.name);
    
    // 验证请求参数
    if request.name.trim().is_empty() {
        warn!("Rule name cannot be empty");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if request.threshold.is_nan() || request.threshold.is_infinite() {
        warn!("Invalid threshold value: {}", request.threshold);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 检查名称是否已存在
    let existing = sqlx::query_scalar!(
        "SELECT id FROM alert_rules WHERE name = $1",
        request.name
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to check existing rule name: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if existing.is_some() {
        warn!("Rule name already exists: {}", request.name);
        return Err(StatusCode::CONFLICT);
    }
    
    // 创建新规则
    let now = Utc::now();
    let rule_id = Uuid::new_v4();
    
    let rule = AlertRule {
        id: rule_id,
        name: request.name,
        description: request.description,
        device_id: request.device_id,
        tag_id: request.tag_id,
        operator: request.operator,
        threshold: request.threshold,
        level: request.level,
        eval_every: request.eval_every.unwrap_or(60),
        eval_for: request.eval_for,
        enabled: true,
        notification_channels: request.notification_channels.unwrap_or_default(),
        silence_duration: request.silence_duration,
        created_by: request.created_by,
        created_at: now,
        updated_at: now,
        last_fired_at: None,
        fire_count: 0,
    };
    
    // 插入数据库
    sqlx::query!(
        r#"
        INSERT INTO alert_rules (
            id, name, description, device_id, tag_id,
            operator, threshold, level, eval_every, eval_for, enabled,
            notification_channels, silence_duration, created_by,
            created_at, updated_at, last_fired_at, fire_count
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
        )
        "#,
        rule.id,
        rule.name,
        rule.description,
        rule.device_id,
        rule.tag_id,
        rule.operator as CompareOperator,
        rule.threshold,
        rule.level as AlertLevel,
        rule.eval_every as i64,
        rule.eval_for.map(|v| v as i64),
        rule.enabled,
        &rule.notification_channels,
        rule.silence_duration.map(|v| v as i64),
        rule.created_by,
        rule.created_at,
        rule.updated_at,
        rule.last_fired_at,
        rule.fire_count as i64
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to insert alert rule: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // 触发规则重新加载
    if let Err(e) = state.alert_engine.reload_rules().await {
        warn!("Failed to reload rules after creation: {}", e);
    }
    
    info!("Alert rule created successfully: {} ({})", rule.name, rule.id);
    
    Ok(Json(rule))
}

/// 获取报警规则详情
async fn get_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AlertRule>, StatusCode> {
    debug!("Getting alert rule: {}", id);
    
    let rule = sqlx::query_as!(
        AlertRule,
        r#"
        SELECT 
            id, name, description, device_id, tag_id,
            operator as "operator: CompareOperator",
            threshold, level as "level: AlertLevel",
            eval_every, eval_for, enabled,
            notification_channels, silence_duration,
            created_by, created_at, updated_at,
            last_fired_at, fire_count
        FROM alert_rules 
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to query alert rule: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    match rule {
        Some(rule) => Ok(Json(rule)),
        None => {
            debug!("Alert rule not found: {}", id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// 更新报警规则
async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateRuleRequest>,
) -> Result<Json<AlertRule>, StatusCode> {
    info!("Updating alert rule: {}", id);
    
    // 检查规则是否存在
    let existing_rule = sqlx::query!(
        "SELECT name FROM alert_rules WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to check existing rule: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if existing_rule.is_none() {
        debug!("Alert rule not found for update: {}", id);
        return Err(StatusCode::NOT_FOUND);
    }
    
    // 构建更新字段
    let mut update_fields = Vec::new();
    let mut params: Vec<Box<dyn sqlx::postgres::PgArgumentValue + Send + Sync>> = Vec::new();
    let mut param_count = 0;
    
    if let Some(name) = &request.name {
        if name.trim().is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
        param_count += 1;
        update_fields.push(format!("name = ${}", param_count));
        params.push(Box::new(name.clone()));
    }
    
    if let Some(description) = &request.description {
        param_count += 1;
        update_fields.push(format!("description = ${}", param_count));
        params.push(Box::new(description.clone()));
    }
    
    if let Some(device_id) = request.device_id {
        param_count += 1;
        update_fields.push(format!("device_id = ${}", param_count));
        params.push(Box::new(device_id));
    }
    
    if let Some(tag_id) = request.tag_id {
        param_count += 1;
        update_fields.push(format!("tag_id = ${}", param_count));
        params.push(Box::new(tag_id));
    }
    
    if let Some(operator) = &request.operator {
        param_count += 1;
        update_fields.push(format!("operator = ${}", param_count));
        params.push(Box::new(operator.clone()));
    }
    
    if let Some(threshold) = request.threshold {
        if threshold.is_nan() || threshold.is_infinite() {
            return Err(StatusCode::BAD_REQUEST);
        }
        param_count += 1;
        update_fields.push(format!("threshold = ${}", param_count));
        params.push(Box::new(threshold));
    }
    
    if let Some(level) = &request.level {
        param_count += 1;
        update_fields.push(format!("level = ${}", param_count));
        params.push(Box::new(level.clone()));
    }
    
    if let Some(eval_every) = request.eval_every {
        param_count += 1;
        update_fields.push(format!("eval_every = ${}", param_count));
        params.push(Box::new(eval_every as i64));
    }
    
    if let Some(eval_for) = request.eval_for {
        param_count += 1;
        update_fields.push(format!("eval_for = ${}", param_count));
        params.push(Box::new(eval_for as i64));
    }
    
    if let Some(notification_channels) = &request.notification_channels {
        param_count += 1;
        update_fields.push(format!("notification_channels = ${}", param_count));
        params.push(Box::new(notification_channels.clone()));
    }
    
    if let Some(silence_duration) = request.silence_duration {
        param_count += 1;
        update_fields.push(format!("silence_duration = ${}", param_count));
        params.push(Box::new(silence_duration as i64));
    }
    
    if update_fields.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 添加updated_at和id参数
    param_count += 1;
    update_fields.push(format!("updated_at = ${}", param_count));
    params.push(Box::new(Utc::now()));
    
    param_count += 1;
    params.push(Box::new(id));
    
    // 执行更新
    let update_query = format!(
        "UPDATE alert_rules SET {} WHERE id = ${}",
        update_fields.join(", "),
        param_count
    );
    
    sqlx::query(&update_query)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to update alert rule: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // 触发规则重新加载
    if let Err(e) = state.alert_engine.reload_rules().await {
        warn!("Failed to reload rules after update: {}", e);
    }
    
    // 返回更新后的规则
    get_rule(State(state), Path(id)).await
}

/// 删除报警规则
async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    info!("Deleting alert rule: {}", id);
    
    let result = sqlx::query!(
        "DELETE FROM alert_rules WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to delete alert rule: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if result.rows_affected() == 0 {
        debug!("Alert rule not found for deletion: {}", id);
        return Err(StatusCode::NOT_FOUND);
    }
    
    // 触发规则重新加载
    if let Err(e) = state.alert_engine.reload_rules().await {
        warn!("Failed to reload rules after deletion: {}", e);
    }
    
    info!("Alert rule deleted successfully: {}", id);
    
    Ok(StatusCode::NO_CONTENT)
}

/// 启用报警规则
async fn enable_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Enabling alert rule: {}", id);
    
    let result = sqlx::query!(
        "UPDATE alert_rules SET enabled = true, updated_at = $1 WHERE id = $2",
        Utc::now(),
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to enable alert rule: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if result.rows_affected() == 0 {
        debug!("Alert rule not found for enabling: {}", id);
        return Err(StatusCode::NOT_FOUND);
    }
    
    // 触发规则重新加载
    if let Err(e) = state.alert_engine.reload_rules().await {
        warn!("Failed to reload rules after enabling: {}", e);
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Rule enabled successfully"
    })))
}

/// 禁用报警规则
async fn disable_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Disabling alert rule: {}", id);
    
    let result = sqlx::query!(
        "UPDATE alert_rules SET enabled = false, updated_at = $1 WHERE id = $2",
        Utc::now(),
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to disable alert rule: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if result.rows_affected() == 0 {
        debug!("Alert rule not found for disabling: {}", id);
        return Err(StatusCode::NOT_FOUND);
    }
    
    // 触发规则重新加载
    if let Err(e) = state.alert_engine.reload_rules().await {
        warn!("Failed to reload rules after disabling: {}", e);
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Rule disabled successfully"
    })))
}

/// 重新加载所有规则
async fn reload_rules(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Reloading all alert rules");
    
    match state.alert_engine.reload_rules().await {
        Ok(()) => {
            info!("Alert rules reloaded successfully");
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Rules reloaded successfully",
                "timestamp": Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to reload alert rules: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}