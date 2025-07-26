//! Prometheus指标服务器
//! 
//! 提供HTTP端点暴露所有模块的Prometheus指标
//! MVP-3: 包含完整的综合指标系统

pub mod server;
pub mod comprehensive;
pub mod integration;

pub use server::{MetricsServer, MetricsServerConfig, create_default_server, start_background_server};
pub use comprehensive::{ComprehensiveMetrics, SystemOverview, COMPREHENSIVE_METRICS, init_comprehensive_metrics};
pub use integration::{MetricsIntegrator, METRICS_INTEGRATOR, init_metrics_integration, WalOperation, DriverOperation};

/// 导出常用指标类型
pub use prometheus::{
    Counter, Gauge, Histogram, IntCounter, IntGauge,
    Opts, HistogramOpts, 
    register_counter, register_gauge, register_histogram,
    register_int_counter, register_int_gauge,
};

/// 获取所有指标的文本表示
pub async fn get_metrics_text() -> anyhow::Result<String> {
    use prometheus::{TextEncoder, Encoder};
    
    // 使用综合指标系统
    let encoder = TextEncoder::new();
    let comprehensive_families = COMPREHENSIVE_METRICS.registry().gather();
    let default_families = prometheus::gather();
    
    let mut buffer = Vec::new();
    
    // 编码综合指标
    encoder.encode(&comprehensive_families, &mut buffer)?;
    
    // 编码默认指标（用于兼容性）
    encoder.encode(&default_families, &mut buffer)?;
    
    Ok(String::from_utf8(buffer)?)
}

/// 启动完整的指标系统
pub async fn start_metrics_system() -> anyhow::Result<tokio::task::JoinHandle<()>> {
    use server::MetricsServerConfig;
    use std::net::SocketAddr;
    
    // 初始化综合指标系统
    init_comprehensive_metrics();
    
    // 启动指标集成
    let _integration_handle = init_metrics_integration().await;
    
    // 启动指标服务器
    let config = MetricsServerConfig {
        bind_addr: "0.0.0.0:9090".parse::<SocketAddr>()?,
        metrics_path: "/metrics".to_string(),
        health_path: "/health".to_string(),
    };
    
    let server_handle = tokio::spawn(async move {
        if let Err(e) = start_background_server(config).await {
            tracing::error!("Failed to start metrics server: {}", e);
        }
    });
    
    tracing::info!("Metrics system started - server on :9090/metrics");
    Ok(server_handle)
}