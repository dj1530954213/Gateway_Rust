//! Prometheus指标

use prometheus::{Counter, Histogram, IntGauge, Opts, HistogramOpts};
use once_cell::sync::Lazy;

/// 全局指标注册表
pub static METRICS: Lazy<EndpointMetrics> = Lazy::new(EndpointMetrics::new);

pub struct EndpointMetrics {
    pub acquire_latency: Histogram,
    pub pool_size: IntGauge,
    pub reconnect_total: Counter,
    pub pause_total: Counter,
    pub timeout_total: Counter,
}

impl EndpointMetrics {
    fn new() -> Self {
        let registry = prometheus::default_registry();
        
        let acquire_latency = Histogram::with_opts(
            HistogramOpts::new(
                "endpoint_acquire_latency_us",
                "Endpoint connection acquire latency in microseconds"
            ).buckets(vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0])
        ).unwrap();
        registry.register(Box::new(acquire_latency.clone())).unwrap();

        let pool_size = IntGauge::with_opts(
            Opts::new("endpoint_pool_size", "Number of connections in pool")
        ).unwrap();
        registry.register(Box::new(pool_size.clone())).unwrap();

        let reconnect_total = Counter::with_opts(
            Opts::new("endpoint_reconnect_total", "Total number of reconnections")
        ).unwrap();
        registry.register(Box::new(reconnect_total.clone())).unwrap();

        let pause_total = Counter::with_opts(
            Opts::new("endpoint_pause_total", "Total number of pause events")
        ).unwrap();
        registry.register(Box::new(pause_total.clone())).unwrap();

        let timeout_total = Counter::with_opts(
            Opts::new("endpoint_timeout_total", "Total number of timeout events")
        ).unwrap();
        registry.register(Box::new(timeout_total.clone())).unwrap();

        Self {
            acquire_latency,
            pool_size,
            reconnect_total,
            pause_total,
            timeout_total,
        }
    }
}

/// 初始化指标
pub fn init_metrics() {
    Lazy::force(&METRICS);
}