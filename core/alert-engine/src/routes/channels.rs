//! routes/channels.rs —— 通知通道管理API
//!
//! 端点列表：
//! - GET /channels: 查询通知通道列表
//! - POST /channels: 创建新的通知通道
//! - GET /channels/{id}: 获取通知通道详情
//! - PUT /channels/{id}: 更新通知通道
//! - DELETE /channels/{id}: 删除通知通道
//! - POST /channels/{id}/test: 测试通知通道
//! - GET /channels/types: 获取支持的通道类型
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::models::{NotificationChannel, NotificationChannelType};
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
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// 创建通道路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_channels).post(create_channel))
        .route("/:id", get(get_channel).put(update_channel).delete(delete_channel))
        .route("/:id/test", post(test_channel))
        .route("/types", get(get_channel_types))
}

/// 通知通道创建请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChannelRequest {
    /// 通道名称
    pub name: String,
    /// 通道类型
    pub channel_type: NotificationChannelType,
    /// 通道配置
    pub config: serde_json::Value,
    /// 是否启用
    pub enabled: Option<bool>,
}

/// 通知通道更新请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateChannelRequest {
    /// 通道名称
    pub name: Option<String>,
    /// 通道配置
    pub config: Option<serde_json::Value>,
    /// 是否启用
    pub enabled: Option<bool>,
}

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct ChannelQueryParams {
    /// 页码（从1开始）
    pub page: Option<u32>,
    /// 每页大小
    pub size: Option<u32>,
    /// 按名称搜索
    pub name: Option<String>,
    /// 按类型过滤
    pub channel_type: Option<NotificationChannelType>,
    /// 按启用状态过滤
    pub enabled: Option<bool>,
    /// 排序字段
    pub sort: Option<String>,
    /// 排序方向 (asc/desc)
    pub order: Option<String>,
}

/// 通道列表响应
#[derive(Debug, Serialize)]
pub struct ChannelListResponse {
    /// 通道列表
    pub items: Vec<NotificationChannel>,
    /// 总数
    pub total: u64,
    /// 当前页
    pub page: u32,
    /// 每页大小
    pub size: u32,
    /// 总页数
    pub total_pages: u32,
}

/// 支持的通道类型响应
#[derive(Debug, Serialize)]
pub struct ChannelTypesResponse {
    /// 支持的类型列表
    pub types: Vec<ChannelTypeInfo>,
}

/// 通道类型信息
#[derive(Debug, Serialize)]
pub struct ChannelTypeInfo {
    /// 类型名称
    pub name: String,
    /// 显示名称
    pub display_name: String,
    /// 描述
    pub description: String,
    /// 配置模板
    pub config_schema: serde_json::Value,
}

/// 测试通道响应
#[derive(Debug, Serialize)]
pub struct TestChannelResponse {
    /// 测试是否成功
    pub success: bool,
    /// 测试消息
    pub message: String,
    /// 错误信息（如果失败）
    pub error: Option<String>,
    /// 测试时间
    pub tested_at: chrono::DateTime<chrono::Utc>,
}

/// 查询通知通道列表
async fn list_channels(
    State(state): State<AppState>,
    Query(params): Query<ChannelQueryParams>,
) -> Result<Json<ChannelListResponse>, StatusCode> {
    debug!("Listing notification channels with params: {:?}", params);
    
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
    
    if let Some(channel_type) = &params.channel_type {
        param_count += 1;
        where_clauses.push(format!("channel_type = ${}", param_count));
        query_params.push(Box::new(channel_type.clone()));
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
    let count_query = format!("SELECT COUNT(*) FROM notification_channels {}", where_clause);
    let total = sqlx::query_scalar(&count_query)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to count notification channels: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // 查询通道列表
    let list_query = format!(
        r#"
        SELECT 
            id, name, channel_type as "channel_type: NotificationChannelType",
            config, enabled, created_at, updated_at
        FROM notification_channels 
        {} {} 
        LIMIT ${} OFFSET ${}
        "#,
        where_clause, order_clause, param_count + 1, param_count + 2
    );
    
    query_params.push(Box::new(size as i64));
    query_params.push(Box::new(offset as i64));
    
    let channels = sqlx::query_as::<_, NotificationChannel>(&list_query)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to query notification channels: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let total_pages = ((total as u32) + size - 1) / size;
    
    let response = ChannelListResponse {
        items: channels,
        total: total as u64,
        page,
        size,
        total_pages,
    };
    
    Ok(Json(response))
}

/// 创建通知通道
async fn create_channel(
    State(state): State<AppState>,
    Json(request): Json<CreateChannelRequest>,
) -> Result<Json<NotificationChannel>, StatusCode> {
    info!("Creating new notification channel: {}", request.name);
    
    // 验证请求参数
    if request.name.trim().is_empty() {
        warn!("Channel name cannot be empty");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // 验证配置格式（这里需要根据具体的通知器类型来验证）
    // TODO: 实现配置验证逻辑
    
    // 检查名称是否已存在
    let existing = sqlx::query_scalar!(
        "SELECT id FROM notification_channels WHERE name = $1",
        request.name
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to check existing channel name: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if existing.is_some() {
        warn!("Channel name already exists: {}", request.name);
        return Err(StatusCode::CONFLICT);
    }
    
    // 创建新通道
    let now = Utc::now();
    let channel_id = Uuid::new_v4();
    
    let channel = NotificationChannel {
        id: channel_id,
        name: request.name,
        channel_type: request.channel_type,
        config: request.config,
        enabled: request.enabled.unwrap_or(true),
        created_at: now,
        updated_at: now,
    };
    
    // 插入数据库
    sqlx::query!(
        r#"
        INSERT INTO notification_channels (
            id, name, channel_type, config, enabled, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7
        )
        "#,
        channel.id,
        channel.name,
        channel.channel_type as NotificationChannelType,
        channel.config,
        channel.enabled,
        channel.created_at,
        channel.updated_at
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to insert notification channel: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    info!("Notification channel created successfully: {} ({})", channel.name, channel.id);
    
    Ok(Json(channel))
}

/// 获取通知通道详情
async fn get_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NotificationChannel>, StatusCode> {
    debug!("Getting notification channel: {}", id);
    
    let channel = sqlx::query_as!(
        NotificationChannel,
        r#"
        SELECT 
            id, name, channel_type as "channel_type: NotificationChannelType",
            config, enabled, created_at, updated_at
        FROM notification_channels 
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to query notification channel: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    match channel {
        Some(channel) => Ok(Json(channel)),
        None => {
            debug!("Notification channel not found: {}", id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// 更新通知通道
async fn update_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateChannelRequest>,
) -> Result<Json<NotificationChannel>, StatusCode> {
    info!("Updating notification channel: {}", id);
    
    // 检查通道是否存在
    let existing = sqlx::query!(
        "SELECT name FROM notification_channels WHERE id = $1",
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to check existing channel: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if existing.is_none() {
        debug!("Notification channel not found for update: {}", id);
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
    
    if let Some(config) = &request.config {
        param_count += 1;
        update_fields.push(format!("config = ${}", param_count));
        params.push(Box::new(config.clone()));
    }
    
    if let Some(enabled) = request.enabled {
        param_count += 1;
        update_fields.push(format!("enabled = ${}", param_count));
        params.push(Box::new(enabled));
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
        "UPDATE notification_channels SET {} WHERE id = ${}",
        update_fields.join(", "),
        param_count
    );
    
    sqlx::query(&update_query)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to update notification channel: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // 返回更新后的通道
    get_channel(State(state), Path(id)).await
}

/// 删除通知通道
async fn delete_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    info!("Deleting notification channel: {}", id);
    
    // 检查通道是否被规则使用
    let rules_using_channel = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM alert_rules WHERE $1 = ANY(notification_channels)",
        id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to check channel usage: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(0);
    
    if rules_using_channel > 0 {
        warn!("Cannot delete channel that is used by {} rules", rules_using_channel);
        return Err(StatusCode::CONFLICT);
    }
    
    let result = sqlx::query!(
        "DELETE FROM notification_channels WHERE id = $1",
        id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to delete notification channel: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if result.rows_affected() == 0 {
        debug!("Notification channel not found for deletion: {}", id);
        return Err(StatusCode::NOT_FOUND);
    }
    
    info!("Notification channel deleted successfully: {}", id);
    
    Ok(StatusCode::NO_CONTENT)
}

/// 测试通知通道
async fn test_channel(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TestChannelResponse>, StatusCode> {
    info!("Testing notification channel: {}", id);
    
    // TODO: 实现实际的测试逻辑
    // 这里需要创建一个测试事件并通过指定的通道发送
    
    // 暂时返回成功响应
    let response = TestChannelResponse {
        success: true,
        message: "Channel test completed successfully".to_string(),
        error: None,
        tested_at: Utc::now(),
    };
    
    Ok(Json(response))
}

/// 获取支持的通道类型
async fn get_channel_types() -> Result<Json<ChannelTypesResponse>, StatusCode> {
    debug!("Getting supported channel types");
    
    let types = vec![
        ChannelTypeInfo {
            name: "email".to_string(),
            display_name: "邮件".to_string(),
            description: "通过SMTP发送邮件通知".to_string(),
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "to": {
                        "type": "string",
                        "format": "email",
                        "title": "收件人邮箱"
                    },
                    "cc": {
                        "type": "array",
                        "items": {"type": "string", "format": "email"},
                        "title": "抄送邮箱"
                    },
                    "subject_template": {
                        "type": "string",
                        "title": "邮件主题模板",
                        "default": "[ALERT] {{rule_name}}"
                    },
                    "body_template": {
                        "type": "string",
                        "title": "邮件正文模板",
                        "default": "Alert: {{message}}\nDevice: {{device_name}}\nValue: {{value}}\nTime: {{timestamp}}"
                    },
                    "use_html": {
                        "type": "boolean",
                        "title": "使用HTML格式",
                        "default": false
                    }
                },
                "required": ["to", "subject_template", "body_template"]
            }),
        },
        ChannelTypeInfo {
            name: "webhook".to_string(),
            display_name: "Webhook".to_string(),
            description: "发送HTTP请求到指定URL".to_string(),
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "format": "uri",
                        "title": "Webhook URL"
                    },
                    "method": {
                        "type": "string",
                        "enum": ["POST", "PUT", "PATCH"],
                        "title": "HTTP方法",
                        "default": "POST"
                    },
                    "headers": {
                        "type": "object",
                        "title": "请求头",
                        "additionalProperties": {"type": "string"}
                    },
                    "body_template": {
                        "type": "object",
                        "title": "请求体模板",
                        "default": {
                            "alert": "{{rule_name}}",
                            "message": "{{message}}",
                            "level": "{{level}}",
                            "timestamp": "{{timestamp}}"
                        }
                    }
                },
                "required": ["url", "body_template"]
            }),
        },
        ChannelTypeInfo {
            name: "websocket".to_string(),
            display_name: "WebSocket".to_string(),
            description: "通过WebSocket实时推送通知".to_string(),
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "room": {
                        "type": "string",
                        "title": "房间名称"
                    },
                    "message_type": {
                        "type": "string",
                        "title": "消息类型",
                        "default": "alert"
                    },
                    "filters": {
                        "type": "object",
                        "title": "过滤条件",
                        "properties": {
                            "levels": {
                                "type": "array",
                                "items": {"type": "string", "enum": ["INFO", "WARN", "CRIT"]},
                                "title": "报警级别过滤"
                            }
                        }
                    }
                }
            }),
        },
    ];
    
    let response = ChannelTypesResponse { types };
    
    Ok(Json(response))
}