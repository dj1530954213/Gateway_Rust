//! routes/alerts.rs —— 报警管理API代理
//!
//! 作为到alert-engine服务的HTTP代理，提供：
//! - 报警规则管理
//! - 报警事件查询
//! - 通知通道管理
//! - 系统状态查询
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::error::{ApiError, ApiResult};
use crate::bootstrap::AppState;
use actix_web::{
    web, HttpRequest, HttpResponse, Scope, 
    http::Method,
};
use awc::Client;
use serde_json::Value;
use tracing::{debug, error, warn};
use url::Url;

/// 报警管理路由范围
pub fn scope() -> Scope {
    web::scope("/alerts")
        .route("", web::get().to(get_alerts_list))
        .route("/health", web::get().to(proxy_health))
        .route("/ready", web::get().to(proxy_ready))
        .route("/stats", web::get().to(proxy_stats))
        .route("/metrics", web::get().to(proxy_metrics))
        // 规则管理
        .service(
            web::scope("/rules")
                .route("", web::get().to(proxy_rules_list))
                .route("", web::post().to(proxy_rules_create))
                .route("/{id}", web::get().to(proxy_rules_get))
                .route("/{id}", web::put().to(proxy_rules_update))
                .route("/{id}", web::delete().to(proxy_rules_delete))
                .route("/{id}/enable", web::post().to(proxy_rules_enable))
                .route("/{id}/disable", web::post().to(proxy_rules_disable))
                .route("/reload", web::post().to(proxy_rules_reload))
        )
        // 事件查询
        .service(
            web::scope("/events")
                .route("", web::get().to(proxy_events_list))
                .route("/{id}", web::get().to(proxy_events_get))
                .route("/{id}/acknowledge", web::post().to(proxy_events_acknowledge))
                .route("/{id}/resolve", web::post().to(proxy_events_resolve))
                .route("/stats", web::get().to(proxy_events_stats))
                .route("/firing", web::get().to(proxy_events_firing))
        )
        // 通知通道
        .service(
            web::scope("/channels")
                .route("", web::get().to(proxy_channels_list))
                .route("", web::post().to(proxy_channels_create))
                .route("/{id}", web::get().to(proxy_channels_get))
                .route("/{id}", web::put().to(proxy_channels_update))
                .route("/{id}", web::delete().to(proxy_channels_delete))
                .route("/{id}/test", web::post().to(proxy_channels_test))
                .route("/types", web::get().to(proxy_channels_types))
        )
}

/// 通用代理处理器
async fn proxy_request(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: &str,
    body: Option<web::Bytes>,
) -> ApiResult<HttpResponse> {
    debug!("Proxying request to alert-engine: {}", path);
    
    // 构建目标URL
    let alert_engine_url = &state.config().alert_engine_url;
    
    // 添加查询参数
    let target_url = if req.query_string().is_empty() {
        format!("{}{}", alert_engine_url, path)
    } else {
        format!("{}{}?{}", alert_engine_url, path, req.query_string())
    };
    
    debug!("Target URL: {}", target_url);
    
    // 创建HTTP客户端
    let client = Client::default();
    
    // 构建请求
    let mut client_req = match req.method() {
        &Method::GET => client.get(&target_url),
        &Method::POST => client.post(&target_url),
        &Method::PUT => client.put(&target_url),
        &Method::DELETE => client.delete(&target_url),
        &Method::PATCH => client.patch(&target_url),
        _ => return Err(ApiError::bad_request("Unsupported HTTP method".to_string())),
    };
    
    // 转发请求头（排除Host等）
    for (name, value) in req.headers() {
        let header_name = name.as_str();
        if !should_skip_header(header_name) {
            if let Ok(header_str) = value.to_str() {
                client_req = client_req.insert_header((name.clone(), header_str));
            }
        }
    }
    
    // 发送请求
    let send_result = if let Some(body) = body {
        client_req
            .content_type("application/json")
            .send_body(body)
            .await
    } else {
        client_req.send().await
    };
    
    match send_result {
        Ok(mut response) => {
            debug!("Alert-engine response status: {}", response.status());
            
            // 构建响应
            let mut http_response = HttpResponse::build(response.status());
            
            // 转发响应头
            for (name, value) in response.headers() {
                if !should_skip_response_header(name.as_str()) {
                    http_response.insert_header((name.clone(), value.clone()));
                }
            }
            
            // 读取响应体
            match response.body().await {
                Ok(body) => Ok(http_response.body(body)),
                Err(e) => {
                    error!("Failed to read alert-engine response body: {}", e);
                    Err(ApiError::internal_error("Failed to read response from alert-engine".to_string()))
                }
            }
        }
        Err(e) => {
            error!("Failed to connect to alert-engine: {}", e);
            Err(ApiError::service_unavailable("Alert engine service unavailable".to_string()))
        }
    }
}

/// 判断是否应该跳过的请求头
fn should_skip_header(header_name: &str) -> bool {
    matches!(header_name.to_lowercase().as_str(), 
        "host" | "connection" | "content-length" | "transfer-encoding"
    )
}

/// 判断是否应该跳过的响应头
fn should_skip_response_header(header_name: &str) -> bool {
    matches!(header_name.to_lowercase().as_str(), 
        "connection" | "transfer-encoding"
    )
}

// 健康检查和系统状态代理
async fn proxy_health(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    proxy_request(state, req, "/health", None).await
}

async fn proxy_ready(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    proxy_request(state, req, "/ready", None).await
}

async fn proxy_stats(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    proxy_request(state, req, "/stats", None).await
}

async fn proxy_metrics(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    proxy_request(state, req, "/metrics", None).await
}

// 规则管理代理
async fn proxy_rules_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let path = format!("/rules{}", 
        if req.query_string().is_empty() { String::new() } 
        else { format!("?{}", req.query_string()) }
    );
    proxy_request(state, req, &path, None).await
}

async fn proxy_rules_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Bytes,
) -> ApiResult<HttpResponse> {
    proxy_request(state, req, "/rules", Some(body)).await
}

async fn proxy_rules_get(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let rule_path = format!("/rules/{}", path.into_inner());
    proxy_request(state, req, &rule_path, None).await
}

async fn proxy_rules_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Bytes,
) -> ApiResult<HttpResponse> {
    let rule_path = format!("/rules/{}", path.into_inner());
    proxy_request(state, req, &rule_path, Some(body)).await
}

async fn proxy_rules_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let rule_path = format!("/rules/{}", path.into_inner());
    proxy_request(state, req, &rule_path, None).await
}

async fn proxy_rules_enable(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let rule_path = format!("/rules/{}/enable", path.into_inner());
    proxy_request(state, req, &rule_path, None).await
}

async fn proxy_rules_disable(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let rule_path = format!("/rules/{}/disable", path.into_inner());
    proxy_request(state, req, &rule_path, None).await
}

async fn proxy_rules_reload(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    proxy_request(state, req, "/rules/reload", None).await
}

// 事件查询代理
async fn proxy_events_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let path = format!("/events{}", 
        if req.query_string().is_empty() { String::new() } 
        else { format!("?{}", req.query_string()) }
    );
    proxy_request(state, req, &path, None).await
}

async fn proxy_events_get(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let event_path = format!("/events/{}", path.into_inner());
    proxy_request(state, req, &event_path, None).await
}

async fn proxy_events_acknowledge(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let event_path = format!("/events/{}/acknowledge", path.into_inner());
    proxy_request(state, req, &event_path, None).await
}

async fn proxy_events_resolve(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let event_path = format!("/events/{}/resolve", path.into_inner());
    proxy_request(state, req, &event_path, None).await
}

async fn proxy_events_stats(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    proxy_request(state, req, "/events/stats", None).await
}

async fn proxy_events_firing(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    proxy_request(state, req, "/events/firing", None).await
}

// 通知通道代理
async fn proxy_channels_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let path = format!("/channels{}", 
        if req.query_string().is_empty() { String::new() } 
        else { format!("?{}", req.query_string()) }
    );
    proxy_request(state, req, &path, None).await
}

async fn proxy_channels_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Bytes,
) -> ApiResult<HttpResponse> {
    proxy_request(state, req, "/channels", Some(body)).await
}

async fn proxy_channels_get(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let channel_path = format!("/channels/{}", path.into_inner());
    proxy_request(state, req, &channel_path, None).await
}

async fn proxy_channels_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Bytes,
) -> ApiResult<HttpResponse> {
    let channel_path = format!("/channels/{}", path.into_inner());
    proxy_request(state, req, &channel_path, Some(body)).await
}

async fn proxy_channels_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let channel_path = format!("/channels/{}", path.into_inner());
    proxy_request(state, req, &channel_path, None).await
}

async fn proxy_channels_test(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let channel_path = format!("/channels/{}/test", path.into_inner());
    proxy_request(state, req, &channel_path, None).await
}

async fn proxy_channels_types(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    proxy_request(state, req, "/channels/types", None).await
}

/// 获取报警列表
async fn get_alerts_list(
    _state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    // 暂时返回空的报警列表，与驱动API格式保持一致
    let response = serde_json::json!({
        "alerts": [],
        "total": 0,
        "page": 1,
        "page_size": 20,
        "total_pages": 0
    });
    
    debug!("返回报警列表: {}", response);
    Ok(HttpResponse::Ok().json(response))
}