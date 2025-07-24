//! Modbus驱动Prometheus指标

use prometheus::{Counter, Histogram, Opts, HistogramOpts};
use once_cell::sync::Lazy;

pub static METRICS: Lazy<ModbusMetrics> = Lazy::new(ModbusMetrics::new);

pub struct ModbusMetrics {
    pub pdu_total: Counter,
    pub point_total: Counter,
    pub point_latency: Histogram,
    pub reconnect_total: Counter,
    pub exception_total: Counter,
}

impl ModbusMetrics {
    fn new() -> Self {
        let registry = prometheus::default_registry();

        let pdu_total = Counter::with_opts(
            Opts::new("modbus_pdu_total", "Total Modbus PDU requests")
        ).unwrap();
        registry.register(Box::new(pdu_total.clone())).unwrap();

        let point_total = Counter::with_opts(
            Opts::new("modbus_point_total", "Total points read")
        ).unwrap();
        registry.register(Box::new(point_total.clone())).unwrap();

        let point_latency = Histogram::with_opts(
            HistogramOpts::new(
                "modbus_point_latency_ms",
                "Point read latency in milliseconds"
            ).buckets(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0])
        ).unwrap();
        registry.register(Box::new(point_latency.clone())).unwrap();

        let reconnect_total = Counter::with_opts(
            Opts::new("modbus_reconnect_total", "Total reconnections")
        ).unwrap();
        registry.register(Box::new(reconnect_total.clone())).unwrap();

        let exception_total = Counter::with_opts(
            Opts::new("modbus_exception_total", "Total Modbus exceptions")
        ).unwrap();
        registry.register(Box::new(exception_total.clone())).unwrap();

        Self {
            pdu_total,
            point_total,
            point_latency,
            reconnect_total,
            exception_total,
        }
    }
}