//! config.rs —— API服务配置结构与加载
//!
//! - 运行时配置：HTTP地址、CORS、数据库连接等
//! - 环境变量覆盖：支持WEBGW_前缀的环境变量
//! - 多环境支持：default.yaml -> {ENV}.yaml -> 环境变量
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    /// HTTP 监听地址，默认 0.0.0.0:8080
    pub http_addr: SocketAddr,
    
    /// Metrics 监听地址，默认 0.0.0.0:8081  
    pub metrics_addr: SocketAddr,
    
    /// WebSocket 路径，默认 "/ws"
    pub ws_path: String,
    
    /// 允许的前端源，支持通配
    pub cors_allowed: Vec<String>,
    
    /// PostgreSQL DSN（元数据：设备、点位）
    pub pg_dsn: String,
    
    /// InfluxDB base url
    pub influx_url: String,
    
    /// InfluxDB token
    pub influx_token: String,
    
    /// InfluxDB organization
    pub influx_org: String,
    
    /// InfluxDB bucket
    pub influx_bucket: String,
    
    /// 框架内部总线 addr（nats://）
    pub bus_url: String,
    
    /// 驱动目录根
    pub drivers_dir: PathBuf,
    
    /// 日志级别
    pub log_level: String,
    
    /// 请求超时时间（秒）
    pub request_timeout: u64,
    
    /// 最大请求体大小（MB）
    pub max_request_size: usize,
    
    /// WebSocket最大连接数
    pub ws_max_connections: u32,
    
    /// WebSocket心跳超时时间（秒）
    pub ws_heartbeat_timeout: u64,
    
    /// WebSocket连接清理间隔（秒）
    pub ws_cleanup_interval: u64,
    
    /// Alert Engine服务地址
    pub alert_engine_url: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            http_addr: "0.0.0.0:8080".parse().unwrap(),
            metrics_addr: "0.0.0.0:8081".parse().unwrap(),
            ws_path: "/ws".to_string(),
            cors_allowed: vec!["http://localhost:5173".to_string()],
            pg_dsn: "postgres://postgres:postgres@localhost:5432/iot".to_string(),
            influx_url: "http://localhost:8086".to_string(),
            influx_token: "dev-token".to_string(),
            influx_org: "iot".to_string(),
            influx_bucket: "telemetry".to_string(),
            bus_url: "nats://localhost:4222".to_string(),
            drivers_dir: "./drivers".into(),
            log_level: "info".to_string(),
            request_timeout: 30,
            max_request_size: 10, // 10MB
            ws_max_connections: 1000,
            ws_heartbeat_timeout: 60,
            ws_cleanup_interval: 30,
            alert_engine_url: "http://localhost:9500".to_string(),
        }
    }
}

impl ApiConfig {
    /// 创建屏蔽敏感信息的配置副本，用于日志输出
    pub fn redacted(&self) -> ApiConfigRedacted {
        ApiConfigRedacted {
            http_addr: self.http_addr,
            metrics_addr: self.metrics_addr,
            ws_path: self.ws_path.clone(),
            cors_allowed: self.cors_allowed.clone(),
            pg_dsn: redact_dsn(&self.pg_dsn),
            influx_url: self.influx_url.clone(),
            influx_token: "***".to_string(),
            influx_org: self.influx_org.clone(),
            influx_bucket: self.influx_bucket.clone(),
            bus_url: self.bus_url.clone(),
            drivers_dir: self.drivers_dir.clone(),
            log_level: self.log_level.clone(),
            request_timeout: self.request_timeout,
            max_request_size: self.max_request_size,
            ws_max_connections: self.ws_max_connections,
            ws_heartbeat_timeout: self.ws_heartbeat_timeout,
            ws_cleanup_interval: self.ws_cleanup_interval,
            alert_engine_url: self.alert_engine_url.clone(),
        }
    }
}

/// 屏蔽敏感信息的配置结构
#[derive(Debug, Serialize)]
pub struct ApiConfigRedacted {
    pub http_addr: SocketAddr,
    pub metrics_addr: SocketAddr,
    pub ws_path: String,
    pub cors_allowed: Vec<String>,
    pub pg_dsn: String,
    pub influx_url: String,
    pub influx_token: String,
    pub influx_org: String,
    pub influx_bucket: String,
    pub bus_url: String,
    pub drivers_dir: PathBuf,
    pub log_level: String,
    pub request_timeout: u64,
    pub max_request_size: usize,
    pub ws_max_connections: u32,
    pub ws_heartbeat_timeout: u64,
    pub ws_cleanup_interval: u64,
    pub alert_engine_url: String,
}

/// 屏蔽DSN中的密码信息
fn redact_dsn(dsn: &str) -> String {
    if let Ok(url) = url::Url::parse(dsn) {
        let mut redacted = url.clone();
        if url.password().is_some() {
            let _ = redacted.set_password(Some("***"));
        }
        redacted.to_string()
    } else {
        "***invalid_dsn***".to_string()
    }
}