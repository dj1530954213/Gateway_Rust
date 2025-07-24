//! Prometheus指标服务器
//! 
//! 提供HTTP端点暴露所有模块的Prometheus指标

pub mod server;

pub use server::{MetricsServer, MetricsServerConfig, create_default_server, start_background_server};

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
    
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    
    Ok(String::from_utf8(buffer)?)
}