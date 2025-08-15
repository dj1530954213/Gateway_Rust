/*! 
# REST API Server

HTTP server implementation using Warp framework for command API.
*/

use std::net::SocketAddr;
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
use anyhow::Result;
use tracing::{info, error};

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
}

impl ApiServer {
    /// Create new API server
    pub fn new(
        config: ServerConfig,
        auth_manager: Arc<AuthManager>,
        command_handler: Arc<CommandHandler>,
        health_handler: Arc<HealthHandler>,
    ) -> Self {
        Self {
            config,
            auth_manager,
            command_handler,
            health_handler,
        }
    }

    /// Start the API server
    pub async fn start(&self) -> Result<()> {
        info!("Starting REST API server on {}", self.config.bind_addr);

        // Build route filters
        let routes = self.build_routes();

        // Start server
        warp::serve(routes)
            .run(self.config.bind_addr)
            .await;

        Ok(())
    }

    /// Build all route filters
    fn build_routes(&self) -> impl Filter<Extract = impl Reply, Error = std::convert::Infallible> + Clone {
        // Health routes (no auth required)
        let health_routes = warp::path("health")
            .and(warp::get())
            .and(with_handler(self.health_handler.clone()))
            .and_then(health_check);

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

        // Command routes (auth required)
        let command_routes = warp::path("commands")
            .and(warp::post())
            .and(warp::body::json())
            .and(auth_filter(self.auth_manager.clone()))
            .and(with_handler(self.command_handler.clone()))
            .and_then(execute_command);

        // Apply CORS to all routes - 完全开放CORS配置 (0.0.0.0)
        let cors = warp::cors()
            .allow_any_origin()     // 允许所有源 (0.0.0.0)
            .allow_headers(vec!["content-type", "authorization", "x-requested-with", "accept", "origin", "user-agent", "x-csrf-token"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH", "HEAD"])
            .allow_credentials(false); // Warp不支持任意源+凭据组合

        // Combine routes with CORS and error recovery
        health_routes
            .or(system_routes)
            .or(api_v1_routes)
            .or(driver_routes)
            .or(ws_routes)
            .or(command_routes)
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