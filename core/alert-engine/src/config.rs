//! config.rs —— 报警引擎配置管理
//!
//! 配置结构：
//! - 数据库连接配置
//! - 通知器配置（SMTP、Webhook）
//! - 引擎运行参数
//! - 监控指标配置
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

/// 报警引擎配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlertEngineConfig {
    /// 服务基础配置
    pub server: ServerConfig,
    
    /// 数据库配置
    pub database: DatabaseConfig,
    
    /// Frame Bus配置
    pub frame_bus: FrameBusConfig,
    
    /// 通知器配置
    pub notifiers: NotifiersConfig,
    
    /// 引擎配置
    pub engine: EngineConfig,
    
    /// 监控配置
    pub monitoring: MonitoringConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// HTTP监听地址（健康检查、指标）
    pub http_addr: SocketAddr,
    
    /// 日志级别
    pub log_level: String,
    
    /// 日志格式（pretty/json）
    pub log_format: String,
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// PostgreSQL DSN
    pub pg_dsn: String,
    
    /// 连接池大小
    pub max_connections: u32,
    
    /// 连接超时（秒）
    pub connect_timeout: u64,
}

/// Frame Bus配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrameBusConfig {
    /// Frame Bus URL
    pub url: String,
    
    /// 订阅过滤器
    pub subscription_filter: String,
    
    /// 重连间隔（秒）
    pub reconnect_interval: u64,
}

/// 通知器配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotifiersConfig {
    /// SMTP邮件配置
    pub smtp: Option<SmtpConfig>,
    
    /// Webhook配置列表
    pub webhooks: Vec<WebhookConfig>,
    
    /// WebSocket配置
    pub websocket: WebSocketConfig,
}

/// SMTP邮件配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmtpConfig {
    /// SMTP服务器地址
    pub host: String,
    
    /// SMTP端口
    pub port: u16,
    
    /// 用户名
    pub username: String,
    
    /// 密码
    pub password: String,
    
    /// 发件人邮箱
    pub from_email: String,
    
    /// 发件人名称
    pub from_name: String,
    
    /// 是否启用TLS
    pub enable_tls: bool,
    
    /// 连接超时（秒）
    pub timeout: u64,
}

/// Webhook配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhookConfig {
    /// Webhook名称
    pub name: String,
    
    /// 目标URL
    pub url: String,
    
    /// HTTP方法
    pub method: String,
    
    /// 请求头
    pub headers: HashMap<String, String>,
    
    /// 超时时间（秒）
    pub timeout: u64,
    
    /// 重试次数
    pub retry_count: u32,
    
    /// 重试间隔（秒）
    pub retry_interval: u64,
}

/// WebSocket配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebSocketConfig {
    /// WebSocket API端点
    pub api_endpoint: String,
    
    /// 连接超时（秒）
    pub connect_timeout: u64,
}

/// 引擎配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EngineConfig {
    /// 规则评估间隔（秒）
    pub evaluation_interval: u64,
    
    /// 批量处理大小
    pub batch_size: u32,
    
    /// 最大并发评估数
    pub max_concurrent_evaluations: u32,
    
    /// 历史数据保留天数
    pub history_retention_days: u32,
    
    /// 规则缓存大小
    pub rule_cache_size: u32,
    
    /// 事件队列大小
    pub event_queue_size: u32,
}

/// 监控配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    /// 是否启用Prometheus指标
    pub enable_metrics: bool,
    
    /// 指标端口
    pub metrics_port: u16,
    
    /// 健康检查间隔（秒）
    pub health_check_interval: u64,
}

impl Default for AlertEngineConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                http_addr: "127.0.0.1:9500".parse().unwrap(),
                log_level: "info".to_string(),
                log_format: "pretty".to_string(),
            },
            database: DatabaseConfig {
                pg_dsn: "postgres://postgres:postgres@localhost:5432/iot".to_string(),
                max_connections: 10,
                connect_timeout: 30,
            },
            frame_bus: FrameBusConfig {
                url: "nats://localhost:4222".to_string(),
                subscription_filter: "telemetry.*".to_string(),
                reconnect_interval: 5,
            },
            notifiers: NotifiersConfig {
                smtp: None,
                webhooks: vec![],
                websocket: WebSocketConfig {
                    api_endpoint: "http://localhost:8080/api/v1/websocket/broadcast".to_string(),
                    connect_timeout: 10,
                },
            },
            engine: EngineConfig {
                evaluation_interval: 10,
                batch_size: 100,
                max_concurrent_evaluations: 50,
                history_retention_days: 30,
                rule_cache_size: 1000,
                event_queue_size: 10000,
            },
            monitoring: MonitoringConfig {
                enable_metrics: true,
                metrics_port: 9501,
                health_check_interval: 30,
            },
        }
    }
}

impl AlertEngineConfig {
    /// 从文件加载配置
    pub fn load_from_file(path: &str) -> crate::AlertResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::AlertError::config_error(format!("Failed to read config file {}: {}", path, e)))?;
        
        let config: AlertEngineConfig = serde_yaml::from_str(&content)
            .map_err(|e| crate::AlertError::config_error(format!("Failed to parse config: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// 验证配置
    pub fn validate(&self) -> crate::AlertResult<()> {
        // 验证数据库连接字符串
        if self.database.pg_dsn.is_empty() {
            return Err(crate::AlertError::config_error("Database DSN cannot be empty"));
        }
        
        // 验证Frame Bus URL
        if self.frame_bus.url.is_empty() {
            return Err(crate::AlertError::config_error("Frame Bus URL cannot be empty"));
        }
        
        // 验证SMTP配置
        if let Some(smtp) = &self.notifiers.smtp {
            if smtp.host.is_empty() || smtp.username.is_empty() || smtp.from_email.is_empty() {
                return Err(crate::AlertError::config_error("SMTP configuration incomplete"));
            }
        }
        
        // 验证Webhook配置
        for webhook in &self.notifiers.webhooks {
            if webhook.url.is_empty() || webhook.name.is_empty() {
                return Err(crate::AlertError::config_error("Webhook configuration incomplete"));
            }
        }
        
        // 验证引擎参数
        if self.engine.evaluation_interval == 0 {
            return Err(crate::AlertError::config_error("Evaluation interval must be > 0"));
        }
        
        if self.engine.batch_size == 0 {
            return Err(crate::AlertError::config_error("Batch size must be > 0"));
        }
        
        Ok(())
    }
    
    /// 从环境变量覆盖配置
    pub fn apply_env_overrides(&mut self) {
        if let Ok(dsn) = std::env::var("ALERT_ENGINE_PG_DSN") {
            self.database.pg_dsn = dsn;
        }
        
        if let Ok(frame_bus_url) = std::env::var("ALERT_ENGINE_FRAME_BUS_URL") {
            self.frame_bus.url = frame_bus_url;
        }
        
        if let Ok(log_level) = std::env::var("ALERT_ENGINE_LOG_LEVEL") {
            self.server.log_level = log_level;
        }
        
        if let Ok(http_addr) = std::env::var("ALERT_ENGINE_HTTP_ADDR") {
            if let Ok(addr) = http_addr.parse() {
                self.server.http_addr = addr;
            }
        }
    }
    
    /// 创建屏蔽敏感信息的配置副本
    pub fn redacted(&self) -> AlertEngineConfigRedacted {
        AlertEngineConfigRedacted {
            server: self.server.clone(),
            database: DatabaseConfigRedacted {
                pg_dsn: redact_dsn(&self.database.pg_dsn),
                max_connections: self.database.max_connections,
                connect_timeout: self.database.connect_timeout,
            },
            frame_bus: self.frame_bus.clone(),
            notifiers: NotifiersConfigRedacted {
                smtp: self.notifiers.smtp.as_ref().map(|smtp| SmtpConfigRedacted {
                    host: smtp.host.clone(),
                    port: smtp.port,
                    username: smtp.username.clone(),
                    password: "***".to_string(),
                    from_email: smtp.from_email.clone(),
                    from_name: smtp.from_name.clone(),
                    enable_tls: smtp.enable_tls,
                    timeout: smtp.timeout,
                }),
                webhooks: self.notifiers.webhooks.clone(),
                websocket: self.notifiers.websocket.clone(),
            },
            engine: self.engine.clone(),
            monitoring: self.monitoring.clone(),
        }
    }
}

/// 屏蔽敏感信息的配置结构
#[derive(Debug, Serialize)]
pub struct AlertEngineConfigRedacted {
    pub server: ServerConfig,
    pub database: DatabaseConfigRedacted,
    pub frame_bus: FrameBusConfig,
    pub notifiers: NotifiersConfigRedacted,
    pub engine: EngineConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Serialize)]
pub struct DatabaseConfigRedacted {
    pub pg_dsn: String,
    pub max_connections: u32,
    pub connect_timeout: u64,
}

#[derive(Debug, Serialize)]
pub struct NotifiersConfigRedacted {
    pub smtp: Option<SmtpConfigRedacted>,
    pub webhooks: Vec<WebhookConfig>,
    pub websocket: WebSocketConfig,
}

#[derive(Debug, Serialize)]
pub struct SmtpConfigRedacted {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub enable_tls: bool,
    pub timeout: u64,
}

/// 屏蔽DSN中的密码信息
fn redact_dsn(dsn: &str) -> String {
    if let Ok(mut url) = url::Url::parse(dsn) {
        if url.password().is_some() {
            let _ = url.set_password(Some("***"));
        }
        url.to_string()
    } else {
        "***invalid_dsn***".to_string()
    }
}