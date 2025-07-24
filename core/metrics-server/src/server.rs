//! Prometheus指标HTTP服务器

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, Method, StatusCode};
use prometheus::{Encoder, TextEncoder};
use anyhow::Result;
use tracing::{info, error};

/// 指标服务器配置
#[derive(Debug, Clone)]
pub struct MetricsServerConfig {
    /// 绑定地址
    pub bind_addr: SocketAddr,
    /// 指标路径
    pub metrics_path: String,
    /// 健康检查路径
    pub health_path: String,
}

impl Default for MetricsServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:9090".parse().unwrap(),
            metrics_path: "/metrics".to_string(),
            health_path: "/health".to_string(),
        }
    }
}

/// Prometheus指标服务器
pub struct MetricsServer {
    config: MetricsServerConfig,
}

impl MetricsServer {
    pub fn new(config: MetricsServerConfig) -> Self {
        Self { config }
    }

    /// 启动指标服务器
    pub async fn start(&self) -> Result<()> {
        info!("Starting metrics server on {}", self.config.bind_addr);

        let metrics_path = self.config.metrics_path.clone();
        let health_path = self.config.health_path.clone();

        let make_svc = make_service_fn(move |_conn| {
            let metrics_path = metrics_path.clone();
            let health_path = health_path.clone();
            
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    handle_request(req, metrics_path.clone(), health_path.clone())
                }))
            }
        });

        let server = Server::bind(&self.config.bind_addr).serve(make_svc);

        info!("Metrics server started on {}", self.config.bind_addr);
        info!("Metrics endpoint: http://{}{}", self.config.bind_addr, self.config.metrics_path);
        info!("Health endpoint: http://{}{}", self.config.bind_addr, self.config.health_path);

        if let Err(e) = server.await {
            error!("Metrics server error: {}", e);
            return Err(anyhow::anyhow!("Metrics server failed: {}", e));
        }

        Ok(())
    }
}

/// 处理HTTP请求
async fn handle_request(
    req: Request<Body>,
    metrics_path: String,
    health_path: String,
) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    let method = req.method();

    match (method, path) {
        (&Method::GET, path) if path == metrics_path => {
            match gather_metrics().await {
                Ok(metrics_body) => Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
                    .body(Body::from(metrics_body))
                    .unwrap()),
                Err(e) => {
                    error!("Failed to gather metrics: {}", e);
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(format!("Error gathering metrics: {}", e)))
                        .unwrap())
                }
            }
        }
        (&Method::GET, path) if path == health_path => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"status":"ok","service":"edge-gateway"}"#))
                .unwrap())
        }
        _ => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap())
        }
    }
}

/// 收集所有Prometheus指标
async fn gather_metrics() -> Result<String> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    
    Ok(String::from_utf8(buffer)?)
}

/// 创建默认指标服务器
pub fn create_default_server() -> MetricsServer {
    MetricsServer::new(MetricsServerConfig::default())
}

/// 在后台启动指标服务器
pub async fn start_background_server(config: MetricsServerConfig) -> Result<()> {
    let server = MetricsServer::new(config);
    
    tokio::spawn(async move {
        if let Err(e) = server.start().await {
            error!("Metrics server failed: {}", e);
        }
    });
    
    Ok(())
}