/*! 
# REST API Server

HTTP server implementation using Warp framework for command API.
*/

use std::net::SocketAddr;
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
use warp::http::StatusCode;
use anyhow::Result;
use tracing::{info, error};
use uuid::Uuid;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

// Repositories & models
use pg_repo::{
    device_repo::{DeviceRepoImpl as PgDeviceRepo, DeviceRepo as _},
    tag_repo::{TagRepoImpl as PgTagRepo, TagRepo as _},
    driver_config_repo::{DriverConfigRepoImpl as PgDriverCfgRepo, DriverConfigRepo as _},
    models::{
        NewDevice, DeviceFilter, DeviceUpdate, DbProtocolKind,
        NewTag, TagFilter,
        NewDriverConfig, DriverConfigFilter,
    },
};

use crate::auth::AuthManager;
use crate::handlers::{CommandHandler, HealthHandler};
use crate::middleware::auth_filter;
use crate::error::ApiError;

/// REST API server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub max_connections: usize,
    pub enable_cors: bool,
    pub enable_rate_limit: bool,
    pub rate_limit_per_minute: u32,
}

// ===== Devices API =====
#[derive(serde::Deserialize)]
struct DeviceCreateReq {
    name: String,
    protocol: String,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    endpoint: Option<String>,
    #[serde(default)]
    config: Option<serde_json::Value>,
    #[serde(default = "default_true")]
    enabled: bool,
}

#[derive(serde::Deserialize, Default)]
struct DeviceQuery {
    page: Option<i64>,
    size: Option<i64>,
    protocol: Option<String>,
    enabled: Option<bool>,
}

fn default_true() -> bool { true }

fn map_protocol(s: &str) -> Result<DbProtocolKind, Rejection> {
    match s {
        "ModbusTcp" => Ok(DbProtocolKind::ModbusTcp),
        "OpcUa" => Ok(DbProtocolKind::OpcUa),
        "Mqtt" => Ok(DbProtocolKind::Mqtt),
        _ => Err(warp::reject::custom(ApiError::BadRequest(format!("Unsupported protocol: {}", s))))
    }
}

async fn create_device(body: DeviceCreateReq, db: Pool<Postgres>) -> Result<impl Reply, Rejection> {
    let repo = PgDeviceRepo::new(db);
    let device = NewDevice {
        id: Uuid::new_v4(),
        name: body.name,
        protocol: map_protocol(&body.protocol)?,
        location: body.location,
        endpoint: body.endpoint,
        config: body.config,
        enabled: body.enabled,
    };
    let created = repo.create(device).await
        .map_err(|e| warp::reject::custom(ApiError::Internal(e.to_string())))?;
    Ok(warp::reply::json(&created))
}

async fn list_devices(query: DeviceQuery, db: Pool<Postgres>) -> Result<impl Reply, Rejection> {
    let repo = PgDeviceRepo::new(db.clone());
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * size;

    let protocol_opt = match query.protocol.as_deref() {
        Some(p) => Some(map_protocol(p)?),
        None => None,
    };
    let enabled_opt = query.enabled;

    let count_filter = DeviceFilter {
        protocol: protocol_opt.clone(),
        enabled: enabled_opt,
        limit: None,
        offset: None,
    };
    let list_filter = DeviceFilter {
        protocol: protocol_opt,
        enabled: enabled_opt,
        limit: Some(size),
        offset: Some(offset),
    };

    let total = repo.count(count_filter).await
        .map_err(|e| warp::reject::custom(ApiError::Internal(e.to_string())))?;
    let items = repo.list(list_filter).await
        .map_err(|e| warp::reject::custom(ApiError::Internal(e.to_string())))?;

    let pages = ((total + size - 1) / size).max(1);
    let resp = serde_json::json!({
        "items": items,
        "total": total,
        "page": page,
        "size": size,
        "pages": pages,
    });
    Ok(warp::reply::json(&resp))
}

// ===== DataPoints API (mapped to Tags) =====
#[derive(serde::Deserialize)]
struct DataPointCreateReq {
    name: String,
    address: String,
    #[serde(rename = "dataType")] 
    data_type: String,
    #[serde(alias = "deviceId")] 
    device_id: Option<Uuid>,
    #[serde(alias = "driverId")] 
    driver_id: Option<Uuid>,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    scaling: Option<f64>,
    #[serde(default, rename = "offset")] 
    offset_val: Option<f64>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_true")] 
    enabled: bool,
}

fn map_tag_type(s: &str) -> Result<pg_repo::models::DbTagDataType, Rejection> {
    match s.to_lowercase().as_str() {
        "bool" | "boolean" => Ok(pg_repo::models::DbTagDataType::Bool),
        "int" | "integer" => Ok(pg_repo::models::DbTagDataType::Int),
        "float" | "double" | "number" => Ok(pg_repo::models::DbTagDataType::Float),
        "string" => Ok(pg_repo::models::DbTagDataType::String),
        other => Err(warp::reject::custom(ApiError::BadRequest(format!("Unsupported dataType: {}", other))))
    }
}

async fn create_datapoint(body: DataPointCreateReq, db: Pool<Postgres>) -> Result<impl Reply, Rejection> {
    let repo = PgTagRepo::new(db);
    let device_id = body.device_id.or(body.driver_id)
        .ok_or_else(|| warp::reject::custom(ApiError::BadRequest("deviceId/driverId is required".into())))?;
    let tag = NewTag {
        id: Uuid::new_v4(),
        device_id,
        name: body.name,
        address: body.address,
        data_type: map_tag_type(&body.data_type)?,
        scaling: body.scaling,
        offset: body.offset_val,
        unit: body.unit,
        description: body.description,
        enabled: body.enabled,
    };
    let created = repo.create(tag).await
        .map_err(|e| warp::reject::custom(ApiError::Internal(e.to_string())))?;
    Ok(warp::reply::json(&created))
}

// ===== Driver Configs API =====
#[derive(serde::Deserialize, Default)]
struct DriverCfgQuery {
    protocol: Option<String>,
    enabled: Option<bool>,
    name_contains: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(serde::Deserialize)]
struct DriverConfigCreateReq {
    name: String,
    #[serde(default)]
    description: Option<String>,
    protocol: String,
    connection_type: String,
    #[serde(default = "default_true")] 
    enabled: bool,
    #[serde(default)]
    config: serde_json::Value,
    #[serde(default)]
    scan_interval: Option<i32>,
    #[serde(default)]
    timeout: Option<i32>,
    #[serde(default)]
    max_concurrent: Option<i32>,
    #[serde(default)]
    batch_size: Option<i32>,
}

async fn create_driver_config(body: DriverConfigCreateReq, db: Pool<Postgres>) -> Result<impl Reply, Rejection> {
    let repo = PgDriverCfgRepo::new(db);
    let new_cfg = NewDriverConfig {
        name: body.name,
        description: body.description,
        protocol: body.protocol,
        connection_type: body.connection_type,
        enabled: body.enabled,
        config: body.config,
        scan_interval: body.scan_interval.unwrap_or(1000),
        timeout: body.timeout.unwrap_or(5000),
        max_concurrent: body.max_concurrent.unwrap_or(10),
        batch_size: body.batch_size.unwrap_or(100),
        max_retries: 3,
        retry_interval: 1000,
        exponential_backoff: true,
        max_retry_interval: 10_000,
        log_level: "info".to_string(),
        enable_request_log: false,
        enable_response_log: false,
        max_log_size: 10,
        enable_ssl: false,
        verify_certificate: true,
        client_cert_path: None,
        client_key_path: None,
    };
    let created = repo.create_driver_config(new_cfg).await
        .map_err(|e| warp::reject::custom(ApiError::Internal(e.to_string())))?;
    Ok(warp::reply::json(&serde_json::json!({ "driver_config": created })))
}

async fn list_driver_configs(query: DriverCfgQuery, db: Pool<Postgres>) -> Result<impl Reply, Rejection> {
    let repo = PgDriverCfgRepo::new(db);
    let page = query.page.unwrap_or(1).max(1);
    let size = query.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * size;

    let filter = DriverConfigFilter {
        protocol: query.protocol.clone(),
        enabled: query.enabled,
        name_contains: query.name_contains.clone(),
        limit: Some(size),
        offset: Some(offset),
    };
    let total = repo.count_driver_configs(DriverConfigFilter { limit: None, offset: None, ..filter.clone() }).await
        .map_err(|e| warp::reject::custom(ApiError::Internal(e.to_string())))?;
    let items = repo.list_driver_configs(filter).await
        .map_err(|e| warp::reject::custom(ApiError::Internal(e.to_string())))?;
    let pages = ((total + size - 1) / size).max(1);
    Ok(warp::reply::json(&serde_json::json!({
        "driver_configs": items,
        "total": total,
        "page": page,
        "page_size": size,
        "total_pages": pages,
    })))
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8080".parse().unwrap(),
            max_connections: 1000,
            enable_cors: true,
            enable_rate_limit: true,
            rate_limit_per_minute: 60,
        }
    }
}

/// REST API server
pub struct ApiServer {
    config: ServerConfig,
    auth_manager: Arc<AuthManager>,
    command_handler: Arc<CommandHandler>,
    health_handler: Arc<HealthHandler>,
    db_pool: Pool<Postgres>,
}

impl ApiServer {
    /// Create new API server
    pub fn new(
        config: ServerConfig,
        auth_manager: Arc<AuthManager>,
        command_handler: Arc<CommandHandler>,
        health_handler: Arc<HealthHandler>,
    ) -> Self {
        // 初始化数据库连接池（懒连接），默认连接到 compose 中的 postgres 服务
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@postgres:5432/iot".to_string());
        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&db_url)
            .expect("failed to create lazy DB pool");

        Self {
            config,
            auth_manager,
            command_handler,
            health_handler,
            db_pool,
        }
    }

    /// Start the API server
    pub async fn start(&self) -> Result<()> {
        info!("Starting REST API server on {}", self.config.bind_addr);

        // 运行数据库迁移（若已执行过将快速跳过）
        if let Err(e) = sqlx::migrate!("../../schema/migrations").run(&self.db_pool).await {
            error!("DB migrations failed: {}", e);
        } else {
            info!("DB migrations applied (or already up-to-date)");
        }

        // Build route filters
        let routes = self.build_routes();
        let addr = self.config.bind_addr;

        // 启动Warp服务器为后台任务，避免阻塞后续组件启动
        tokio::spawn(async move {
            warp::serve(routes)
                .run(addr)
                .await;
        });

        Ok(())
    }

    /// Build all route filters
    fn build_routes(&self) -> impl Filter<Extract = impl Reply, Error = std::convert::Infallible> + Clone {
        // Health routes (no auth required)
        let health_routes = warp::path("health")
            .and(warp::path::end())
            .and(warp::get())
            .and(with_handler(self.health_handler.clone()))
            .and_then(health_check);

        // Healthz alias for compatibility (/healthz)
        let healthz_routes = warp::path("healthz")
            .and(warp::path::end())
            .and(warp::get())
            .and(with_handler(self.health_handler.clone()))
            .and_then(health_check);

        // Liveness probe: /health/live
        let health_live_routes = warp::path("health")
            .and(warp::path("live"))
            .and(warp::path::end())
            .and(warp::get())
            .map(|| warp::reply::json(&serde_json::json!({
                "status": "alive"
            })));

        // Readiness probe: /health/ready
        let health_ready_routes = warp::path("health")
            .and(warp::path("ready"))
            .and(warp::path::end())
            .and(warp::get())
            .map(|| warp::reply::json(&serde_json::json!({
                "status": "ready"
            })));

        // System routes (no auth required for now)
        let system_routes = warp::path("system")
            .and(
                warp::path("info")
                    .and(warp::get())
                    .and_then(system_info)
                .or(warp::path("metrics")
                    .and(warp::get())
                    .and_then(system_metrics))
                .or(warp::path("components")
                    .and(warp::path("status"))
                    .and(warp::get())
                    .and_then(component_status))
                .or(warp::path("health")
                    .and(warp::get())
                    .and_then(system_health))
            );

        // API v1 routes
        let api_v1_routes = warp::path("api")
            .and(warp::path("v1"))
            .and(
                warp::path("drivers")
                    .and(warp::get())
                    .and_then(list_drivers)
                .or(warp::path("system")
                    .and(warp::path("health"))
                    .and(warp::get())
                    .and_then(system_health))
                .or(warp::path("alerts")
                    .and(warp::get())
                    .and_then(list_alerts))
            );

        // Driver status routes (direct /drivers/status)
        let driver_routes = warp::path("drivers")
            .and(
                warp::path("status")
                    .and(warp::get())
                    .and_then(driver_status)
                .or(warp::get()
                    .and_then(list_drivers_v2))
            );

        // WebSocket route
        let ws_routes = warp::path("ws")
            .and(warp::ws())
            .map(|ws: warp::ws::Ws| {
                ws.on_upgrade(|websocket| async {
                    info!("WebSocket connection established");
                    // Handle WebSocket connection
                    let (_tx, mut rx) = websocket.split();
                    while let Some(result) = rx.next().await {
                        match result {
                            Ok(_msg) => {
                                // Handle message
                            }
                            Err(e) => {
                                error!("WebSocket error: {}", e);
                                break;
                            }
                        }
                    }
                })
            });

        // WebSocket telemetry alias: /ws/telemetry -> same handler as /ws
        let ws_telemetry_routes = warp::path("ws")
            .and(warp::path("telemetry"))
            .and(warp::ws())
            .map(|ws: warp::ws::Ws| {
                ws.on_upgrade(|websocket| async {
                    info!("WebSocket /ws/telemetry connection established");
                    let (_tx, mut rx) = websocket.split();
                    while let Some(result) = rx.next().await {
                        match result {
                            Ok(_msg) => {}
                            Err(e) => {
                                error!("WebSocket error: {}", e);
                                break;
                            }
                        }
                    }
                })
            });

        // Command routes (auth required)
        let command_routes = warp::path("commands")
            .and(warp::post())
            .and(warp::body::json())
            .and(auth_filter(self.auth_manager.clone()))
            .and(with_handler(self.command_handler.clone()))
            .and_then(execute_command);

        // ========== Minimal CRUD for Devices ==========
        let db_pool = self.db_pool.clone();
        let devices_create = warp::path("devices")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_db(db_pool.clone()))
            .and_then(create_device);

        let devices_list = warp::path("devices")
            .and(warp::get())
            .and(warp::query::<DeviceQuery>())
            .and(with_db(db_pool.clone()))
            .and_then(list_devices);

        // ========== Minimal CRUD for DataPoints (Tags) ==========
        let datapoints_create = warp::path("datapoints")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_db(db_pool.clone()))
            .and_then(create_datapoint);

        // ========== Driver Configs ==========
        let driver_cfg_create = warp::path("api")
            .and(warp::path("v1"))
            .and(warp::path("driver-configs"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_db(db_pool.clone()))
            .and_then(create_driver_config);

        let driver_cfg_list = warp::path("api")
            .and(warp::path("v1"))
            .and(warp::path("driver-configs"))
            .and(warp::get())
            .and(warp::query::<DriverCfgQuery>())
            .and(with_db(db_pool.clone()))
            .and_then(list_driver_configs);

        // Global OPTIONS route for CORS preflight
        let options_routes = warp::options()
            .and(warp::path::tail())
            .map(|_tail: warp::filters::path::Tail| warp::reply::with_status("", StatusCode::NO_CONTENT));

        // Apply CORS to all routes - 完全开放CORS配置 (0.0.0.0)
        let cors = warp::cors()
            .allow_any_origin()     // 允许所有源 (0.0.0.0)
            .allow_headers(vec![
                "content-type",
                "authorization",
                "x-requested-with",
                "accept",
                "origin",
                "user-agent",
                "x-csrf-token",
                "x-request-id",
                "accept-language",
            ])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH", "HEAD"])
            .allow_credentials(false); // Warp不支持任意源+凭据组合

        // Combine routes with CORS and error recovery
        health_routes
            .or(healthz_routes)
            .or(health_live_routes)
            .or(health_ready_routes)
            .or(system_routes)
            .or(api_v1_routes)
            .or(driver_routes)
            .or(ws_routes)
            .or(ws_telemetry_routes)
            .or(command_routes)
            .or(devices_create)
            .or(devices_list)
            .or(datapoints_create)
            .or(driver_cfg_create)
            .or(driver_cfg_list)
            .or(options_routes)
            .with(cors)
            .recover(crate::error::handle_rejection)
    }
}

// Helper functions
fn with_handler<T>(handler: Arc<T>) -> impl Filter<Extract = (Arc<T>,), Error = std::convert::Infallible> + Clone
where
    T: Send + Sync,
{
    warp::any().map(move || handler.clone())
}

fn with_db(pool: Pool<Postgres>) -> impl Filter<Extract = (Pool<Postgres>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

async fn health_check(
    handler: Arc<HealthHandler>,
) -> Result<impl Reply, Rejection> {
    match handler.check().await {
        Ok(status) => Ok(warp::reply::json(&status)),
        Err(e) => {
            error!("Health check failed: {}", e);
            Err(warp::reject::custom(ApiError::Internal(e.to_string())))
        }
    }
}

async fn execute_command(
    request: serde_json::Value,
    user_id: String,
    handler: Arc<CommandHandler>,
) -> Result<impl Reply, Rejection> {
    match handler.execute(request, &user_id).await {
        Ok(response) => Ok(warp::reply::json(&response)),
        Err(e) => {
            error!("Command execution failed: {}", e);
            Err(warp::reject::custom(ApiError::BadRequest(e.to_string())))
        }
    }
}

async fn system_info() -> Result<impl Reply, Rejection> {
    let info = serde_json::json!({
        "version": "1.0.0",
        "uptime": 86400,
        "memory_usage": 65.5,
        "cpu_usage": 12.3,
        "disk_usage": 45.2,
        "connected_devices": 5,
        "active_tags": 128,
        "alert_count": 2
    });
    Ok(warp::reply::json(&info))
}

async fn system_metrics() -> Result<impl Reply, Rejection> {
    let metrics = serde_json::json!({
        "cpuUsage": 45.2,
        "memoryUsage": 67.8,
        "diskUsage": 23.1,
        "activeConnections": 12,
        "messagesPerSecond": 456,
        "uptime": 86400,
        "networkIn": 2048576.0,
        "networkOut": 1048576.0
    });
    Ok(warp::reply::json(&metrics))
}

async fn component_status() -> Result<impl Reply, Rejection> {
    let status = serde_json::json!({
        "database": { "status": "healthy", "uptime": 86400, "connections": 5 },
        "messageQueue": { "status": "healthy", "queueSize": 0, "throughput": 100 },
        "webServer": { "status": "healthy", "activeConnections": 3, "requests": 1024 },
        "fileSystem": { "status": "healthy", "freeSpace": "15.2GB", "diskUsage": 75 }
    });
    Ok(warp::reply::json(&status))
}

async fn list_drivers() -> Result<impl Reply, Rejection> {
    let drivers = serde_json::json!([
        {
            "id": "modbus-tcp-001",
            "filename": "modbus_tcp.so",
            "file_size": 204800,
            "status": "Loaded",
            "uploaded_at": "2025-01-27T10:00:00Z",
            "info": {
                "name": "Modbus TCP Driver",
                "version": "1.0.0",
                "protocol": "Modbus TCP",
                "description": "Standard Modbus TCP/IP driver"
            }
        },
        {
            "id": "opcua-001", 
            "filename": "opcua_client.dll",
            "file_size": 512000,
            "status": "Loaded",
            "uploaded_at": "2025-01-27T09:30:00Z",
            "info": {
                "name": "OPC UA Client",
                "version": "2.1.0",
                "protocol": "OPC UA", 
                "description": "OPC UA client driver"
            }
        }
    ]);
    Ok(warp::reply::json(&drivers))
}

async fn list_drivers_v2() -> Result<impl Reply, Rejection> {
    // 同样的数据，但可能格式不同
    list_drivers().await
}

async fn system_health() -> Result<impl Reply, Rejection> {
    let health = serde_json::json!({
        "status": "healthy",
        "uptime": 86400,
        "services": {
            "database": "healthy",
            "message_queue": "healthy", 
            "web_server": "healthy"
        },
        "last_check": "2025-01-31T08:34:00Z"
    });
    Ok(warp::reply::json(&health))
}

async fn driver_status() -> Result<impl Reply, Rejection> {
    let status = serde_json::json!({
        "loaded_count": 2,
        "failed_count": 0,
        "unloaded_count": 0,
        "total_count": 2
    });
    Ok(warp::reply::json(&status))
}

async fn list_alerts() -> Result<impl Reply, Rejection> {
    let alerts = serde_json::json!([
        {
            "id": "alert-001",
            "type": "warning",
            "source": "Device-001",
            "message": "温度传感器读数异常",
            "timestamp": "2025-01-31T08:30:00Z",
            "severity": "medium",
            "acknowledged": false
        },
        {
            "id": "alert-002", 
            "type": "info",
            "source": "System",
            "message": "系统启动完成",
            "timestamp": "2025-01-31T08:00:00Z",
            "severity": "low",
            "acknowledged": true
        }
    ]);
    Ok(warp::reply::json(&alerts))
}

// WebSocket imports
use futures_util::{SinkExt, StreamExt};