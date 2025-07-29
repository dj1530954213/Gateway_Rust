//! main.rs —— 报警引擎服务主入口
//!
//! 独立运行的报警引擎服务，负责：
//! - 初始化服务组件
//! - 启动数据监听
//! - 运行规则评估引擎
//! - 处理通知发送
//! - 提供健康检查和指标
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use alert_engine::{AlertEngineConfig, AlertEngineService, AlertResult};
use anyhow::Context;
use std::env;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    init_logging().context("Failed to initialize logging")?;
    
    info!("Starting Alert Engine Service...");
    
    // 加载配置
    let config_path = env::var("ALERT_ENGINE_CONFIG")
        .unwrap_or_else(|_| "config/alert-engine.yaml".to_string());
    
    let mut config = if std::path::Path::new(&config_path).exists() {
        AlertEngineConfig::load_from_file(&config_path)
            .context("Failed to load configuration file")?
    } else {
        warn!("Config file {} not found, using default configuration", config_path);
        AlertEngineConfig::default()
    };
    
    // 应用环境变量覆盖
    config.apply_env_overrides();
    
    info!("Configuration loaded: {:?}", config.redacted());
    
    // 创建和启动服务
    let mut service = AlertEngineService::new(config)
        .await
        .context("Failed to create alert engine service")?;
    
    // 启动服务
    let service_task = tokio::spawn(async move {
        if let Err(e) = service.start().await {
            error!("Alert engine service failed: {}", e);
        }
    });
    
    info!("Alert Engine Service started successfully");
    
    // 等待停止信号
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received SIGINT, shutting down...");
        }
        _ = signal_term() => {
            info!("Received SIGTERM, shutting down...");
        }
        result = service_task => {
            match result {
                Ok(_) => info!("Service task completed"),
                Err(e) => error!("Service task failed: {}", e),
            }
        }
    }
    
    // 优雅关闭
    info!("Shutting down Alert Engine Service...");
    // 注意：service已被移动到task中，这里只是等待task完成
    info!("Alert Engine Service stopped");
    
    Ok(())
}

/// 初始化日志系统
fn init_logging() -> AlertResult<()> {
    let log_level = env::var("ALERT_ENGINE_LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string());
    
    let log_format = env::var("ALERT_ENGINE_LOG_FORMAT")
        .unwrap_or_else(|_| "pretty".to_string());
    
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(log_level.parse().unwrap_or(tracing::Level::INFO))
        .with_current_span(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE);
    
    if log_format == "json" {
        subscriber.json().init();
    } else {
        subscriber.pretty().init();
    }
    
    Ok(())
}

/// 等待SIGTERM信号（Unix系统）
#[cfg(unix)]
async fn signal_term() {
    use tokio::signal::unix::{signal, SignalKind};
    
    let mut stream = signal(SignalKind::terminate()).unwrap();
    stream.recv().await;
}

/// Windows下的SIGTERM等效实现
#[cfg(not(unix))]
async fn signal_term() {
    // Windows下只支持Ctrl+C
    std::future::pending::<()>().await;
}