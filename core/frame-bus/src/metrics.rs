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
    // 批量处理指标
    pub batch_size: Histogram,
    pub batch_flush_duration: Histogram,
    pub batch_send_duration: Histogram,
    pub batch_memory_usage: IntGauge,
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

        // 批量处理指标
        let batch_size = Histogram::with_opts(
            HistogramOpts::new(
                "framebus_batch_size",
                "Batch size distribution"
            ).buckets(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0])
        ).unwrap();
        registry.register(Box::new(batch_size.clone())).unwrap();

        let batch_flush_duration = Histogram::with_opts(
            HistogramOpts::new(
                "framebus_batch_flush_duration_ms",
                "Batch flush duration in milliseconds"
            ).buckets(vec![0.01, 0.1, 0.5, 1.0, 5.0, 10.0, 25.0])
        ).unwrap();
        registry.register(Box::new(batch_flush_duration.clone())).unwrap();

        let batch_send_duration = Histogram::with_opts(
            HistogramOpts::new(
                "framebus_batch_send_duration_ms",
                "Batch send operation duration in milliseconds"
            ).buckets(vec![0.01, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0])
        ).unwrap();
        registry.register(Box::new(batch_send_duration.clone())).unwrap();

        let batch_memory_usage = IntGauge::with_opts(
            Opts::new("framebus_batch_memory_bytes", "Batch buffer memory usage in bytes")
        ).unwrap();
        registry.register(Box::new(batch_memory_usage.clone())).unwrap();

        Self {
            ring_used,
            publish_total,
            drop_total,
            backlog_lag,
            wal_bytes,
            wal_flush_duration,
            batch_size,
            batch_flush_duration,
            batch_send_duration,
            batch_memory_usage,
        }
    }
}

/// 初始化指标
pub fn init_metrics() {
    Lazy::force(&METRICS);
}