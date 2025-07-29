/*!
# REST API Server v2

MVP-3 REST API服务器，实现CP-0/CP-1端点和RBAC
- CP-0: 只读端点 (Configuration Profile 0)
- CP-1: 读写端点 (Configuration Profile 1)
*/

use std::collections::HashMap;
use std::sync::Arc;
use std::convert::Infallible;

use warp::{Filter, Reply, Rejection, reject};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{Result, ApiError};
use crate::rbac::{RbacManager, Claims, Permission, Role};

/// API配置
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub enable_cors: bool,
    pub request_timeout: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: "gateway-jwt-secret-change-in-production".to_string(),
            enable_cors: true,
            request_timeout: 30,
        }
    }
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

/// 用户信息
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub roles: Vec<String>,
}

/// 配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub readonly: bool,
}

/// 系统状态
#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub status: String,
    pub uptime: u64,
    pub version: String,
    pub services: HashMap<String, ServiceStatus>,
}

/// 服务状态
#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub status: String,
    pub message: String,
    pub last_check: u64,
}

/// API错误响应
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
    pub message: String,
}

/// REST API Server v2
pub struct ApiServerV2 {
    config: ApiConfig,
    rbac: Arc<RbacManager>,
    config_store: Arc<RwLock<HashMap<String, ConfigItem>>>,
}

impl ApiServerV2 {
    /// 创建新的API服务器
    pub fn new(config: ApiConfig) -> Self {
        let rbac = Arc::new(RbacManager::new(config.jwt_secret.clone()));
        let config_store = Arc::new(RwLock::new(HashMap::new()));

        let server = Self {
            config,
            rbac,
            config_store,
        };

        // 初始化默认配置
        tokio::spawn({
            let config_store = server.config_store.clone();
            async move {
                Self::init_default_config(config_store).await;
            }
        });

        server
    }

    /// 初始化默认配置
    async fn init_default_config(config_store: Arc<RwLock<HashMap<String, ConfigItem>>>) {
        let mut store = config_store.write().await;
        
        store.insert("system.name".to_string(), ConfigItem {
            key: "system.name".to_string(),
            value: serde_json::Value::String("Gateway System".to_string()),
            description: Some("System name".to_string()),
            readonly: false,
        });

        store.insert("system.version".to_string(), ConfigItem {
            key: "system.version".to_string(),
            value: serde_json::Value::String("1.0.0".to_string()),
            description: Some("System version".to_string()),
            readonly: true,
        });

        store.insert("network.max_connections".to_string(), ConfigItem {
            key: "network.max_connections".to_string(),
            value: serde_json::Value::Number(1000.into()),
            description: Some("Maximum concurrent connections".to_string()),
            readonly: false,
        });

        tracing::info!("Default configuration initialized");
    }

    /// 启动API服务器
    pub async fn start(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port)
            .parse::<std::net::SocketAddr>()
            .map_err(|e| ApiError::Internal(format!("Invalid bind address: {}", e)))?;

        tracing::info!("Starting REST API Server v2 on {}", addr);

        // 创建路由
        let routes = self.create_routes();

        // 启动服务器
        warp::serve(routes)
            .run(addr)
            .await;

        Ok(())
    }

    /// 创建所有路由
    fn create_routes(&self) -> impl Filter<Extract = impl Reply, Error = std::convert::Infallible> + Clone {
        // 身份验证路由
        let auth_routes = self.auth_routes();
        
        // CP-0路由（只读）
        let cp0_routes = self.cp0_routes();
        
        // CP-1路由（读写）
        let cp1_routes = self.cp1_routes();
        
        // 健康检查路由
        let health_routes = self.health_routes();

        // CORS支持
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

        // 组合所有路由
        auth_routes
            .or(cp0_routes)
            .or(cp1_routes)
            .or(health_routes)
            .with(cors)
            .recover(handle_rejection)
    }

    /// 身份验证路由
    fn auth_routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let rbac = self.rbac.clone();

        // POST /auth/login
        let login = warp::path!("auth" / "login")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_rbac(rbac))
            .and_then(login_handler);

        login
    }

    /// CP-0路由（只读）
    fn cp0_routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let rbac = self.rbac.clone();
        let config_store = self.config_store.clone();

        // GET /cp0/config
        let get_config = warp::path!("cp0" / "config")
            .and(warp::get())
            .and(with_auth(rbac.clone()))
            .and(with_config_store(config_store.clone()))
            .and_then(get_config_handler);

        // GET /cp0/config/{key}
        let get_config_key = warp::path!("cp0" / "config" / String)
            .and(warp::get())
            .and(with_auth(rbac.clone()))
            .and(with_config_store(config_store.clone()))
            .and_then(get_config_key_handler);

        // GET /cp0/status
        let get_status = warp::path!("cp0" / "status")
            .and(warp::get())
            .and(with_auth(rbac.clone()))
            .and_then(get_status_handler);

        get_config.or(get_config_key).or(get_status)
    }

    /// CP-1路由（读写）
    fn cp1_routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let rbac = self.rbac.clone();
        let config_store = self.config_store.clone();

        // GET /cp1/config (same as CP-0)
        let get_config = warp::path!("cp1" / "config")
            .and(warp::get())
            .and(with_auth(rbac.clone()))
            .and(with_config_store(config_store.clone()))
            .and_then(get_config_handler);

        // PUT /cp1/config/{key}
        let put_config_key = warp::path!("cp1" / "config" / String)
            .and(warp::put())
            .and(warp::body::json())
            .and(with_auth_write(rbac.clone()))
            .and(with_config_store(config_store.clone()))
            .and_then(put_config_key_handler);

        // DELETE /cp1/config/{key}
        let delete_config_key = warp::path!("cp1" / "config" / String)
            .and(warp::delete())
            .and(with_auth_write(rbac.clone()))
            .and(with_config_store(config_store.clone()))
            .and_then(delete_config_key_handler);

        // POST /cp1/driver/{driver_id}/restart
        let restart_driver = warp::path!("cp1" / "driver" / String / "restart")
            .and(warp::post())
            .and(with_auth_write(rbac.clone()))
            .and_then(restart_driver_handler);

        get_config.or(put_config_key).or(delete_config_key).or(restart_driver)
    }

    /// 健康检查路由
    fn health_routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        // GET /healthz
        let healthz = warp::path!("healthz")
            .and(warp::get())
            .and_then(healthz_handler);

        // GET /metrics
        let metrics = warp::path!("metrics")
            .and(warp::get())
            .and_then(metrics_handler);

        healthz.or(metrics)
    }
}

// === Filter 辅助函数 ===

fn with_rbac(rbac: Arc<RbacManager>) -> impl Filter<Extract = (Arc<RbacManager>,), Error = Infallible> + Clone {
    warp::any().map(move || rbac.clone())
}

fn with_auth(rbac: Arc<RbacManager>) -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
    warp::header::<String>("authorization")
        .and(with_rbac(rbac))
        .and_then(|auth_header: String, rbac: Arc<RbacManager>| async move {
            let token = auth_header.strip_prefix("Bearer ")
                .ok_or_else(|| reject::custom(ApiError::Unauthorized("Invalid authorization header".to_string())))?;

            let claims = rbac.validate_token(token)
                .map_err(|e| reject::custom(e))?;

            rbac.check_permission(&claims, Permission::ConfigRead)
                .map_err(|e| reject::custom(e))?;

            Ok::<Claims, Rejection>(claims)
        })
}

fn with_auth_write(rbac: Arc<RbacManager>) -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
    warp::header::<String>("authorization")
        .and(with_rbac(rbac))
        .and_then(|auth_header: String, rbac: Arc<RbacManager>| async move {
            let token = auth_header.strip_prefix("Bearer ")
                .ok_or_else(|| reject::custom(ApiError::Unauthorized("Invalid authorization header".to_string())))?;

            let claims = rbac.validate_token(token)
                .map_err(|e| reject::custom(e))?;

            rbac.check_permission(&claims, Permission::ConfigWrite)
                .map_err(|e| reject::custom(e))?;

            Ok::<Claims, Rejection>(claims)
        })
}

fn with_config_store(store: Arc<RwLock<HashMap<String, ConfigItem>>>) -> impl Filter<Extract = (Arc<RwLock<HashMap<String, ConfigItem>>>,), Error = Infallible> + Clone {
    warp::any().map(move || store.clone())
}

// === Handler 函数 ===

async fn login_handler(request: LoginRequest, rbac: Arc<RbacManager>) -> std::result::Result<impl Reply, Rejection> {
    let token = rbac.login(&request.username, &request.password).await
        .map_err(|e| reject::custom(e))?;

    let user = rbac.get_user(&request.username).await
        .ok_or_else(|| reject::custom(ApiError::NotFound("User not found".to_string())))?;

    let response = LoginResponse {
        token,
        expires_in: 24 * 3600, // 24 hours
        user: UserInfo {
            id: user.id,
            username: user.username,
            roles: user.roles.iter().map(|r| r.to_string()).collect(),
        },
    };

    Ok(warp::reply::json(&response))
}

async fn get_config_handler(
    _claims: Claims,
    config_store: Arc<RwLock<HashMap<String, ConfigItem>>>,
) -> std::result::Result<impl Reply, Rejection> {
    let store = config_store.read().await;
    let configs: Vec<&ConfigItem> = store.values().collect();
    Ok(warp::reply::json(&configs))
}

async fn get_config_key_handler(
    key: String,
    _claims: Claims,
    config_store: Arc<RwLock<HashMap<String, ConfigItem>>>,
) -> std::result::Result<impl Reply, Rejection> {
    let store = config_store.read().await;
    
    if let Some(config) = store.get(&key) {
        Ok(warp::reply::json(config))
    } else {
        Err(reject::custom(ApiError::NotFound(format!("Config key not found: {}", key))))
    }
}

async fn put_config_key_handler(
    key: String,
    value: serde_json::Value,
    _claims: Claims,
    config_store: Arc<RwLock<HashMap<String, ConfigItem>>>,
) -> std::result::Result<impl Reply, Rejection> {
    let mut store = config_store.write().await;
    
    // 检查是否为只读配置
    if let Some(existing) = store.get(&key) {
        if existing.readonly {
            return Err(reject::custom(ApiError::BadRequest("Cannot modify readonly configuration".to_string())));
        }
    }

    let config_item = ConfigItem {
        key: key.clone(),
        value,
        description: None,
        readonly: false,
    };

    store.insert(key, config_item.clone());
    
    tracing::info!("Configuration updated: {}", config_item.key);
    Ok(warp::reply::json(&config_item))
}

async fn delete_config_key_handler(
    key: String,
    _claims: Claims,
    config_store: Arc<RwLock<HashMap<String, ConfigItem>>>,
) -> std::result::Result<impl Reply, Rejection> {
    let mut store = config_store.write().await;
    
    // 检查是否为只读配置
    if let Some(existing) = store.get(&key) {
        if existing.readonly {
            return Err(reject::custom(ApiError::BadRequest("Cannot delete readonly configuration".to_string())));
        }
    }

    if store.remove(&key).is_some() {
        tracing::info!("Configuration deleted: {}", key);
        Ok(warp::reply::with_status("", warp::http::StatusCode::NO_CONTENT))
    } else {
        Err(reject::custom(ApiError::NotFound(format!("Config key not found: {}", key))))
    }
}

async fn get_status_handler(_claims: Claims) -> std::result::Result<impl Reply, Rejection> {
    let mut services = HashMap::new();
    
    services.insert("api".to_string(), ServiceStatus {
        status: "healthy".to_string(),
        message: "API server is running".to_string(),
        last_check: chrono::Utc::now().timestamp() as u64,
    });

    services.insert("database".to_string(), ServiceStatus {
        status: "healthy".to_string(),
        message: "Database connection is active".to_string(),
        last_check: chrono::Utc::now().timestamp() as u64,
    });

    let status = SystemStatus {
        status: "healthy".to_string(),
        uptime: 12345, // TODO: 实际计算运行时间
        version: "1.0.0".to_string(),
        services,
    };

    Ok(warp::reply::json(&status))
}

async fn restart_driver_handler(driver_id: String, _claims: Claims) -> std::result::Result<impl Reply, Rejection> {
    // TODO: 实际实现驱动重启逻辑
    tracing::info!("Restarting driver: {}", driver_id);
    
    let response = serde_json::json!({
        "message": format!("Driver {} restart initiated", driver_id),
        "driver_id": driver_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(warp::reply::json(&response))
}

async fn healthz_handler() -> std::result::Result<impl Reply, Rejection> {
    let response = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    Ok(warp::reply::json(&response))
}

async fn metrics_handler() -> std::result::Result<impl Reply, Rejection> {
    // TODO: 实际实现Prometheus指标
    let metrics = r#"
# HELP gateway_api_requests_total Total number of API requests
# TYPE gateway_api_requests_total counter
gateway_api_requests_total{method="GET",endpoint="/cp0/config"} 100
gateway_api_requests_total{method="PUT",endpoint="/cp1/config"} 50

# HELP gateway_api_response_time_seconds API response time in seconds
# TYPE gateway_api_response_time_seconds histogram
gateway_api_response_time_seconds_bucket{le="0.001"} 50
gateway_api_response_time_seconds_bucket{le="0.01"} 100
gateway_api_response_time_seconds_bucket{le="0.1"} 150
gateway_api_response_time_seconds_bucket{le="+Inf"} 150
gateway_api_response_time_seconds_sum 5.0
gateway_api_response_time_seconds_count 150
"#;

    Ok(warp::reply::with_header(
        metrics,
        "content-type",
        "text/plain; version=0.0.4; charset=utf-8",
    ))
}

// === 错误处理 ===

async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (404, "Not Found".to_string())
    } else if let Some(api_error) = err.find::<ApiError>() {
        match api_error {
            ApiError::BadRequest(msg) => (400, msg.clone()),
            ApiError::Unauthorized(msg) => (401, msg.clone()),
            ApiError::Forbidden(msg) => (403, msg.clone()),
            ApiError::NotFound(msg) => (404, msg.clone()),
            ApiError::Internal(msg) => (500, msg.clone()),
        }
    } else {
        tracing::error!("Unhandled rejection: {:?}", err);
        (500, "Internal Server Error".to_string())
    };

    let error_response = ErrorResponse {
        error: "API Error".to_string(),
        code,
        message,
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&error_response),
        warp::http::StatusCode::from_u16(code).unwrap_or(warp::http::StatusCode::INTERNAL_SERVER_ERROR),
    ))
}

// === 自定义错误类型 ===
impl warp::reject::Reject for ApiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_server_creation() {
        let config = ApiConfig::default();
        let server = ApiServerV2::new(config);
        
        // 等待初始化完成
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let store = server.config_store.read().await;
        assert!(store.contains_key("system.name"));
        assert!(store.contains_key("system.version"));
    }

    #[tokio::test]
    async fn test_config_operations() {
        let config = ApiConfig::default();
        let server = ApiServerV2::new(config);
        
        // 等待初始化完成
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // 测试配置读取
        let store = server.config_store.read().await;
        let system_name = store.get("system.name").unwrap();
        assert_eq!(system_name.key, "system.name");
        
        // 测试只读配置
        let system_version = store.get("system.version").unwrap();
        assert!(system_version.readonly);
    }

    #[tokio::test]
    async fn test_rbac_integration() {
        let config = ApiConfig::default();
        let server = ApiServerV2::new(config);
        
        // 测试默认用户登录
        let token = server.rbac.login("admin", "admin123").await.unwrap();
        assert!(!token.is_empty());
        
        // 测试token验证
        let claims = server.rbac.validate_token(&token).unwrap();
        assert_eq!(claims.username, "admin");
        assert!(claims.has_permission(Permission::ConfigWrite));
    }
}