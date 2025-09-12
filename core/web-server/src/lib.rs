/*!
# Web Server

Static file server and web interface for the Edge Gateway management UI.
*/

use std::net::SocketAddr;
use std::path::PathBuf;
// use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    response::Html,
    routing::{get, get_service},
    Router,
    http::StatusCode,
};
use axum::Server;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};

/// Web server configuration
#[derive(Debug, Clone)]
pub struct WebServerConfig {
    pub bind_addr: SocketAddr,
    pub static_dir: PathBuf,
    pub enable_cors: bool,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8090".parse().unwrap(),
            static_dir: PathBuf::from("web"),
            enable_cors: true,
        }
    }
}

/// Web server for management interface
pub struct WebServer {
    config: WebServerConfig,
    app: Router,
}

impl WebServer {
    /// Create new web server
    pub fn new(config: WebServerConfig) -> Result<Self> {
        // Create the main router
        let mut app = Router::new()
            .route("/", get(serve_index))
            .route("/health", get(health_check))
            .route("/healthz", get(health_check))
            .route("/api/status", get(get_status));

        // 静态目录与 SPA 回退：优先让显式路由命中（上面三个），其余路径交给静态文件服务
        // /app/web 挂载了 Vite 构建产物 dist，包含 index.html 与 /assets/*
        let static_root = PathBuf::from("/app/web");
        let app = app.fallback_service(
            get_service(
                ServeDir::new(&static_root)
                    .not_found_service(ServeFile::new(static_root.join("index.html")))
            )
        );

        // Add middleware
        let app = app.layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));
        
        let app = if config.enable_cors {
            app.layer(CorsLayer::permissive())
        } else {
            app
        };

        Ok(Self { config, app })
    }

    /// Start the web server
    pub async fn start(&self) -> Result<()> {
        info!("Starting web server on {}", self.config.bind_addr);

        info!("Web management interface available at: http://{}", self.config.bind_addr);

        // 后台启动 Axum Server（不阻塞网关启动）。
        let addr = self.config.bind_addr;
        let app = self.app.clone();
        tokio::spawn(async move {
            if let Err(e) = Server::bind(&addr)
                .serve(app.into_make_service())
                .await
            {
                tracing::error!("Web server error: {}", e);
            }
        });

        Ok(())
    }

    /// Get the bind address
    pub fn bind_addr(&self) -> SocketAddr {
        self.config.bind_addr
    }
}

/// Serve the main index page
async fn serve_index() -> Result<Html<String>, StatusCode> {
    // Prefer serving built SPA index.html if present
    let index_path = PathBuf::from("/app/web").join("index.html");
    if index_path.exists() {
        match tokio::fs::read_to_string(&index_path).await {
            Ok(content) => return Ok(Html(content)),
            Err(_e) => {
                // fall through to embedded minimal page
            }
        }
    }

    // Embedded minimal page as fallback
    let html_content = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Edge Gateway</title>
    <meta http-equiv="refresh" content="3">
    <style>body{font-family:system-ui,-apple-system,Segoe UI,Roboto,Ubuntu;max-width:880px;margin:40px auto;padding:0 16px}</style>
</head>
<body>
    <h1>Edge Gateway 工业网关系统</h1>
    <p>前端构建产物尚未检测到，若你已执行构建，将自动提供静态页面。</p>
    <ul>
        <li>REST 健康: <code>/healthz</code></li>
        <li>Web 健康: <code>http://localhost:8090/healthz</code></li>
        <li>指标: <code>http://localhost:9090/metrics</code></li>
    </ul>
</body>
</html>"#;
    Ok(Html(html_content.to_string()))
}

/// Health check endpoint
async fn health_check() -> Result<axum::Json<serde_json::Value>, StatusCode> {
    Ok(axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "web-server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Get system status
async fn get_status() -> Result<axum::Json<serde_json::Value>, StatusCode> {
    // This would normally fetch real status from the gateway
    Ok(axum::Json(serde_json::json!({
        "gateway": {
            "status": "running",
            "uptime": "2h 15m",
            "version": "1.0.0"
        },
        "drivers": {
            "static": 2,
            "dynamic": 0,
            "active": 2
        },
        "statistics": {
            "data_points_today": 12547,
            "commands_executed": 23,
            "success_rate": 98.9
        },
        "security": {
            "signature_verification": true,
            "permission_control": true,
            "api_auth": "jwt"
        }
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_web_server_config() {
        let config = WebServerConfig::default();
        assert_eq!(config.bind_addr.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(config.bind_addr.port(), 8090);
        assert!(config.enable_cors);
    }

    #[test]
    fn test_web_server_creation() {
        let config = WebServerConfig::default();
        let server = WebServer::new(config);
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert!(response.is_ok());
        
        let json = response.unwrap();
        assert_eq!(json["status"], "healthy");
        assert_eq!(json["service"], "web-server");
    }

    #[tokio::test]
    async fn test_get_status() {
        let config = WebServerConfig::default();
        let app_state = AppState { config };
        
        let response = get_status(State(app_state)).await;
        assert!(response.is_ok());
        
        let json = response.unwrap();
        assert_eq!(json["gateway"]["status"], "running");
        assert_eq!(json["drivers"]["static"], 2);
    }
}