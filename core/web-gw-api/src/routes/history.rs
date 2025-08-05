//! history.rs —— 历史数据查询路由
//!
//! - 时间序列数据查询：GET /api/v1/history/points
//! - 聚合统计查询：GET /api/v1/history/stats
//! - CSV数据导出：POST /api/v1/history/export
//! - 健康检查：GET /api/v1/history/health
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::bootstrap::AppState;
use crate::dto::{HistoryQuery, HistoryPointVO, HistoryStatsVO, HistoryExportRequest, AggregatedQuery};
use crate::error::{ApiError, ApiResult};
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use tracing::{info, debug, warn};
use uuid::Uuid;

/// 配置历史数据路由
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/history")
            .route("/points", web::get().to(query_points))
            .route("/stats", web::get().to(query_stats))
            .route("/aggregated", web::get().to(query_aggregated))
            .route("/export", web::post().to(export_csv))
            .route("/health", web::get().to(health_check))
    );
}

/// 查询历史数据点
/// 
/// GET /api/v1/history/points?device_id=xxx&tag_id=xxx&start_time=xxx&end_time=xxx
/// 
/// # Parameters
/// - device_id: 设备ID（可选）
/// - tag_id: 点位ID（可选）
/// - start_time: 开始时间（ISO8601格式）
/// - end_time: 结束时间（ISO8601格式）
/// - aggregation_window: 聚合窗口（可选，如"5m"）
/// - offset: 分页偏移量（可选，默认0）
/// - limit: 分页大小（可选，默认1000）
/// 
/// # Returns
/// ```json
/// {
///   "points": [
///     {
///       "device_id": "uuid",
///       "tag_id": "uuid", 
///       "timestamp": "2025-01-27T10:00:00Z",
///       "value": 23.5,
///       "unit": "celsius"
///     }
///   ],
///   "total": 1500,
///   "returned": 1000
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/history/points",
    tag = "History",
    summary = "查询历史数据点",
    params(
        ("device_id" = Option<Uuid>, Query, description = "设备ID"),
        ("tag_id" = Option<Uuid>, Query, description = "点位ID"),
        ("start_time" = String, Query, description = "开始时间（ISO8601）"),
        ("end_time" = String, Query, description = "结束时间（ISO8601）"),
        ("aggregation_window" = Option<String>, Query, description = "聚合窗口"),
        ("offset" = Option<u64>, Query, description = "分页偏移量"),
        ("limit" = Option<u64>, Query, description = "分页大小"),
    ),
    responses(
        (status = 200, description = "查询成功", body = Vec<HistoryPointVO>),
        (status = 400, description = "参数错误"),
        (status = 500, description = "服务器错误")
    )
)]
async fn query_points(
    query: web::Query<HistoryQuery>,
    app_state: web::Data<AppState>,
) -> ApiResult<impl Responder> {
    debug!(
        device_id = ?query.device_id,
        tag_id = ?query.tag_id,
        start = %query.start_time,
        end = %query.end_time,
        "Received history points query"
    );

    // 验证查询参数
    if query.start_time >= query.end_time {
        return Err(ApiError::bad_request("start_time must be before end_time"));
    }

    let duration = query.end_time - query.start_time;
    if duration.num_days() > 30 {
        return Err(ApiError::bad_request("Time range cannot exceed 30 days"));
    }

    // 执行查询
    let query_data = query.clone().into_inner();
    let points = app_state.history_service.query_points(query_data).await?;
    let returned_count = points.len();

    info!(
        returned = returned_count,
        device_id = ?query.device_id,
        tag_id = ?query.tag_id,
        "History points query completed"
    );

    Ok(HttpResponse::Ok().json(json!({
        "points": points,
        "returned": returned_count,
        "limit": query.limit.unwrap_or(1000),
        "offset": query.offset.unwrap_or(0)
    })))
}

/// 查询聚合统计数据
/// 
/// GET /api/v1/history/stats?device_id=xxx&tag_id=xxx&start_time=xxx&end_time=xxx&aggregation_window=5m
/// 
/// # Parameters
/// - device_id: 设备ID（可选）
/// - tag_id: 点位ID（可选）
/// - start_time: 开始时间（ISO8601格式）
/// - end_time: 结束时间（ISO8601格式）
/// - aggregation_window: 聚合窗口（必需，如"5m", "1h", "1d"）
/// 
/// # Returns
/// ```json
/// {
///   "stats": [
///     {
///       "device_id": "uuid",
///       "tag_id": "uuid",
///       "timestamp": "2025-01-27T10:00:00Z",
///       "min_value": 20.0,
///       "max_value": 25.0,
///       "avg_value": 22.5,
///       "count": 300
///     }
///   ],
///   "window": "5m",
///   "total_windows": 288
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/history/stats",
    tag = "History",
    summary = "查询聚合统计数据",
    params(
        ("device_id" = Option<Uuid>, Query, description = "设备ID"),
        ("tag_id" = Option<Uuid>, Query, description = "点位ID"),
        ("start_time" = String, Query, description = "开始时间（ISO8601）"),
        ("end_time" = String, Query, description = "结束时间（ISO8601）"),
        ("aggregation_window" = String, Query, description = "聚合窗口（必需）"),
    ),
    responses(
        (status = 200, description = "查询成功", body = Vec<HistoryStatsVO>),
        (status = 400, description = "参数错误"),
        (status = 500, description = "服务器错误")
    )
)]
async fn query_stats(
    query: web::Query<HistoryQuery>,
    app_state: web::Data<AppState>,
) -> ApiResult<impl Responder> {
    debug!(
        device_id = ?query.device_id,
        tag_id = ?query.tag_id,
        window = ?query.aggregation_window,
        "Received history stats query"
    );

    // 验证聚合窗口参数
    if query.aggregation_window.is_none() {
        return Err(ApiError::bad_request("aggregation_window is required for stats query"));
    }

    let window = query.aggregation_window.as_ref().unwrap();
    if !is_valid_window(window) {
        return Err(ApiError::bad_request("Invalid aggregation_window format. Use formats like '1m', '5m', '1h', '1d'"));
    }

    // 验证时间范围
    if query.start_time >= query.end_time {
        return Err(ApiError::bad_request("start_time must be before end_time"));
    }

    let duration = query.end_time - query.start_time;
    if duration.num_days() > 90 {
        return Err(ApiError::bad_request("Time range for stats cannot exceed 90 days"));
    }

    // 执行聚合查询
    let query_data = query.clone().into_inner();
    let stats = app_state.history_service.query_stats(query_data).await?;
    let total_windows = stats.len();

    info!(
        windows = total_windows,
        window = window,
        device_id = ?query.device_id,
        tag_id = ?query.tag_id,
        "History stats query completed"
    );

    Ok(HttpResponse::Ok().json(json!({
        "stats": stats,
        "window": window,
        "total_windows": total_windows
    })))
}

/// 查询聚合统计数据（简化版）
/// 
/// GET /api/v1/history/aggregated?device_id=xxx&tag_id=xxx&start_time=xxx&end_time=xxx&window=5m&function=mean
/// 
/// # Parameters
/// - device_id: 设备ID（可选）
/// - tag_id: 点位ID（可选）  
/// - start_time: 开始时间（ISO8601格式）
/// - end_time: 结束时间（ISO8601格式）
/// - window: 聚合窗口（如"5m", "1h", "1d"）
/// - function: 聚合函数（mean/min/max/sum/count）
/// 
/// # Returns
/// ```json
/// {
///   "function": "mean",
///   "window": "5m", 
///   "points": [
///     {
///       "timestamp": "2025-01-27T10:00:00Z",
///       "value": 22.5
///     }
///   ]
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/history/aggregated",
    tag = "History",
    summary = "查询聚合统计数据（简化版）",
    params(
        ("device_id" = Option<Uuid>, Query, description = "设备ID"),
        ("tag_id" = Option<Uuid>, Query, description = "点位ID"),
        ("start_time" = String, Query, description = "开始时间（ISO8601）"),
        ("end_time" = String, Query, description = "结束时间（ISO8601）"),
        ("window" = String, Query, description = "聚合窗口"),
        ("function" = String, Query, description = "聚合函数"),
    ),
    responses(
        (status = 200, description = "查询成功"),
        (status = 400, description = "参数错误"),
        (status = 500, description = "服务器错误")
    )
)]
async fn query_aggregated(
    query: web::Query<AggregatedQuery>,
    app_state: web::Data<AppState>,
) -> ApiResult<impl Responder> {
    debug!(
        device_id = ?query.device_id,
        tag_id = ?query.tag_id,
        window = %query.window,
        function = %query.function,
        "Received aggregated query"
    );

    // 验证参数
    if query.start_time >= query.end_time {
        return Err(ApiError::bad_request("start_time must be before end_time"));
    }

    if !is_valid_window(&query.window) {
        return Err(ApiError::bad_request("Invalid window format"));
    }

    if !is_valid_aggregate_function(&query.function) {
        return Err(ApiError::bad_request("Invalid aggregate function. Supported: mean, min, max, sum, count"));
    }

    // 转换为HistoryQuery
    let history_query = HistoryQuery {
        device_id: query.device_id,
        tag_id: query.tag_id,
        start_time: query.start_time,
        end_time: query.end_time,
        aggregation_window: Some(query.window.clone()),
        offset: None,
        limit: None,
    };

    // 根据聚合函数调用不同的查询方法
    let result = match query.function.as_str() {
        "mean" | "min" | "max" | "count" => {
            // 使用现有的stats查询
            let stats = app_state.history_service.query_stats(history_query).await?;
            
            // 提取对应的聚合值
            let points: Vec<serde_json::Value> = stats.into_iter().map(|stat| {
                let value = match query.function.as_str() {
                    "mean" => stat.avg_value.unwrap_or(0.0),
                    "min" => stat.min_value.unwrap_or(0.0),
                    "max" => stat.max_value.unwrap_or(0.0),
                    "count" => stat.count as f64,
                    _ => 0.0,
                };
                json!({
                    "timestamp": stat.timestamp,
                    "value": value,
                    "device_id": stat.device_id,
                    "tag_id": stat.tag_id
                })
            }).collect();
            
            points
        },
        _ => vec![]
    };

    info!(
        points = result.len(),
        function = %query.function,
        window = %query.window,
        "Aggregated query completed"
    );

    Ok(HttpResponse::Ok().json(json!({
        "function": query.function,
        "window": query.window,
        "points": result
    })))
}

/// 导出历史数据为CSV
/// 
/// POST /api/v1/history/export
/// 
/// # Request Body
/// ```json
/// {
///   "query": {
///     "device_id": "uuid",
///     "tag_id": "uuid",
///     "start_time": "2025-01-27T00:00:00Z",
///     "end_time": "2025-01-27T23:59:59Z"
///   },
///   "include_headers": true,
///   "timestamp_format": "ISO8601"
/// }
/// ```
/// 
/// # Returns
/// CSV格式的历史数据文件
#[utoipa::path(
    post,
    path = "/api/v1/history/export",
    tag = "History",
    summary = "导出历史数据为CSV",
    request_body = HistoryExportRequest,
    responses(
        (status = 200, description = "导出成功", content_type = "text/csv"),
        (status = 400, description = "参数错误"),
        (status = 413, description = "数据量过大"),
        (status = 500, description = "服务器错误")
    )
)]
async fn export_csv(
    request: web::Json<HistoryExportRequest>,
    app_state: web::Data<AppState>,
) -> ApiResult<impl Responder> {
    info!(
        device_id = ?request.query.device_id,
        tag_id = ?request.query.tag_id,
        format = %request.timestamp_format,
        "Received CSV export request"
    );

    // 验证导出请求
    if request.query.start_time >= request.query.end_time {
        return Err(ApiError::bad_request("start_time must be before end_time"));
    }

    let duration = request.query.end_time - request.query.start_time;
    if duration.num_days() > 7 {
        return Err(ApiError::bad_request("CSV export time range cannot exceed 7 days"));
    }

    // 验证时间戳格式
    if !is_valid_timestamp_format(&request.timestamp_format) {
        return Err(ApiError::bad_request("Invalid timestamp_format. Supported: ISO8601, UNIX, UNIX_MS, FORMATTED"));
    }

    // 执行导出
    let csv_data = app_state.history_service.export_csv(request.into_inner()).await?;
    
    // 检查数据大小
    if csv_data.len() > 10 * 1024 * 1024 { // 10MB限制
        warn!(size = csv_data.len(), "CSV export data size exceeds limit");
        return Err(ApiError::payload_too_large("CSV export data too large. Please reduce time range"));
    }

    info!(
        size = csv_data.len(),
        lines = csv_data.lines().count(),
        "CSV export completed"
    );

    Ok(HttpResponse::Ok()
        .content_type("text/csv")
        .insert_header(("Content-Disposition", "attachment; filename=\"history_export.csv\""))
        .body(csv_data))
}

/// 检查历史数据服务健康状态
/// 
/// GET /api/v1/history/health
/// 
/// # Returns
/// ```json
/// {
///   "status": "healthy",
///   "timestamp": "2025-01-27T10:00:00Z",
///   "influxdb": "healthy",
///   "org": "iot",
///   "bucket": "telemetry"
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/history/health",
    tag = "History",
    summary = "检查历史数据服务健康状态",
    responses(
        (status = 200, description = "健康检查成功"),
        (status = 503, description = "服务不可用")
    )
)]
async fn health_check(
    app_state: web::Data<AppState>,
) -> ApiResult<impl Responder> {
    debug!("Performing history service health check");

    let health = app_state.history_service.health_check().await?;
    
    let overall_status = if health.get("influxdb").unwrap_or(&"unknown".to_string()) == "healthy" {
        "healthy"
    } else {
        "unhealthy"
    };

    let mut response = json!({
        "status": overall_status,
        "timestamp": chrono::Utc::now(),
    });

    // 合并健康检查结果
    for (key, value) in health {
        response[key] = json!(value);
    }

    info!(status = overall_status, "History service health check completed");

    if overall_status == "healthy" {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(response))
    }
}

/// 验证聚合窗口格式
fn is_valid_window(window: &str) -> bool {
    // 简单的窗口格式验证：数字+单位
    let re = regex::Regex::new(r"^\d+[smhd]$").unwrap_or_else(|_| {
        // 如果正则表达式编译失败，返回一个匹配所有的正则表达式
        regex::Regex::new(r".*").unwrap()
    });
    re.is_match(window)
}

/// 验证时间戳格式
fn is_valid_timestamp_format(format: &str) -> bool {
    matches!(format, "ISO8601" | "UNIX" | "UNIX_MS" | "FORMATTED")
}

/// 验证聚合函数
fn is_valid_aggregate_function(function: &str) -> bool {
    matches!(function, "mean" | "min" | "max" | "sum" | "count")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_window() {
        assert!(is_valid_window("1m"));
        assert!(is_valid_window("5m"));
        assert!(is_valid_window("1h"));
        assert!(is_valid_window("24h"));
        assert!(is_valid_window("1d"));
        assert!(is_valid_window("30d"));
        
        assert!(!is_valid_window("1"));
        assert!(!is_valid_window("m"));
        assert!(!is_valid_window("1x"));
        assert!(!is_valid_window(""));
    }

    #[test]
    fn test_is_valid_timestamp_format() {
        assert!(is_valid_timestamp_format("ISO8601"));
        assert!(is_valid_timestamp_format("UNIX"));
        assert!(is_valid_timestamp_format("UNIX_MS"));
        assert!(is_valid_timestamp_format("FORMATTED"));
        
        assert!(!is_valid_timestamp_format("invalid"));
        assert!(!is_valid_timestamp_format(""));
        assert!(!is_valid_timestamp_format("iso8601"));
    }

    #[test]
    fn test_is_valid_aggregate_function() {
        assert!(is_valid_aggregate_function("mean"));
        assert!(is_valid_aggregate_function("min"));
        assert!(is_valid_aggregate_function("max"));
        assert!(is_valid_aggregate_function("sum"));
        assert!(is_valid_aggregate_function("count"));
        
        assert!(!is_valid_aggregate_function("invalid"));
        assert!(!is_valid_aggregate_function(""));
        assert!(!is_valid_aggregate_function("MEAN"));
    }
}