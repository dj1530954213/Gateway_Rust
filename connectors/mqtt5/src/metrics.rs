//! MQTT5连接器Prometheus指标

use prometheus::{Counter, Histogram, IntGauge, Opts, HistogramOpts};
use once_cell::sync::Lazy;

pub static METRICS: Lazy<MqttMetrics> = Lazy::new(MqttMetrics::new);

pub struct MqttMetrics {
    pub connect_total: Counter,
    pub disconnect_total: Counter,
    pub reconnect_total: Counter,
    pub publish_total: Counter,
    pub publish_error_total: Counter,
    pub publish_latency: Histogram,
    pub message_size: Histogram,
    pub buffer_used: IntGauge,
    pub compression_ratio: Histogram,
    pub batch_size: Histogram,
}

impl MqttMetrics {
    fn new() -> Self {
        let registry = prometheus::default_registry();

        let connect_total = Counter::with_opts(
            Opts::new("mqtt_connect_total", "Total MQTT connections")
        ).unwrap();
        registry.register(Box::new(connect_total.clone())).unwrap();

        let disconnect_total = Counter::with_opts(
            Opts::new("mqtt_disconnect_total", "Total MQTT disconnections")
        ).unwrap();
        registry.register(Box::new(disconnect_total.clone())).unwrap();

        let reconnect_total = Counter::with_opts(
            Opts::new("mqtt_reconnect_total", "Total MQTT reconnections")
        ).unwrap();
        registry.register(Box::new(reconnect_total.clone())).unwrap();

        let publish_total = Counter::with_opts(
            Opts::new("mqtt_publish_total", "Total messages published")
        ).unwrap();
        registry.register(Box::new(publish_total.clone())).unwrap();

        let publish_error_total = Counter::with_opts(
            Opts::new("mqtt_publish_error_total", "Total publish errors")
        ).unwrap();
        registry.register(Box::new(publish_error_total.clone())).unwrap();

        let publish_latency = Histogram::with_opts(
            HistogramOpts::new(
                "mqtt_publish_latency_ms",
                "Message publish latency in milliseconds"
            ).buckets(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0])
        ).unwrap();
        registry.register(Box::new(publish_latency.clone())).unwrap();

        let message_size = Histogram::with_opts(
            HistogramOpts::new(
                "mqtt_message_size_bytes",
                "Message size in bytes"
            ).buckets(vec![128.0, 512.0, 1024.0, 4096.0, 16384.0, 65536.0])
        ).unwrap();
        registry.register(Box::new(message_size.clone())).unwrap();

        let buffer_used = IntGauge::with_opts(
            Opts::new("mqtt_buffer_used", "Number of messages in buffer")
        ).unwrap();
        registry.register(Box::new(buffer_used.clone())).unwrap();

        let compression_ratio = Histogram::with_opts(
            HistogramOpts::new(
                "mqtt_compression_ratio",
                "Compression ratio (compressed/original)"
            ).buckets(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0])
        ).unwrap();
        registry.register(Box::new(compression_ratio.clone())).unwrap();

        let batch_size = Histogram::with_opts(
            HistogramOpts::new(
                "mqtt_batch_size",
                "Number of messages per batch"
            ).buckets(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0])
        ).unwrap();
        registry.register(Box::new(batch_size.clone())).unwrap();

        Self {
            connect_total,
            disconnect_total,
            reconnect_total,
            publish_total,
            publish_error_total,
            publish_latency,
            message_size,
            buffer_used,
            compression_ratio,
            batch_size,
        }
    }
}