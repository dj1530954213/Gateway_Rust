//! FrameBus Prometheus指标

use prometheus::{Counter, IntGauge, Histogram, Opts, HistogramOpts};
use once_cell::sync::Lazy;

/// 全局指标
pub static METRICS: Lazy<BusMetrics> = Lazy::new(BusMetrics::new);

pub struct BusMetrics {
    pub ring_used: IntGauge,
    pub publish_total: Counter,
    pub drop_total: Counter,
    pub backlog_lag: IntGauge,
    pub wal_bytes: IntGauge,
    pub wal_flush_duration: Histogram,
}

impl BusMetrics {
    fn new() -> Self {
        let registry = prometheus::default_registry();

        let ring_used = IntGauge::with_opts(
            Opts::new("framebus_ring_used", "Number of frames in ring buffer")
        ).unwrap();
        registry.register(Box::new(ring_used.clone())).unwrap();

        let publish_total = Counter::with_opts(
            Opts::new("framebus_publish_total", "Total frames published")
        ).unwrap();
        registry.register(Box::new(publish_total.clone())).unwrap();

        let drop_total = Counter::with_opts(
            Opts::new("framebus_drop_total", "Total frames dropped")
        ).unwrap();
        registry.register(Box::new(drop_total.clone())).unwrap();

        let backlog_lag = IntGauge::with_opts(
            Opts::new("framebus_backlog_lag", "Consumer backlog lag")
        ).unwrap();
        registry.register(Box::new(backlog_lag.clone())).unwrap();

        let wal_bytes = IntGauge::with_opts(
            Opts::new("framebus_wal_bytes", "WAL size in bytes")
        ).unwrap();
        registry.register(Box::new(wal_bytes.clone())).unwrap();

        let wal_flush_duration = Histogram::with_opts(
            HistogramOpts::new(
                "framebus_wal_flush_duration_ms",
                "WAL flush duration in milliseconds"
            ).buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0])
        ).unwrap();
        registry.register(Box::new(wal_flush_duration.clone())).unwrap();

        Self {
            ring_used,
            publish_total,
            drop_total,
            backlog_lag,
            wal_bytes,
            wal_flush_duration,
        }
    }
}

/// 初始化指标
pub fn init_metrics() {
    Lazy::force(&METRICS);
}