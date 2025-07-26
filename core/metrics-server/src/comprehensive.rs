/*!
# Comprehensive Metrics System

MVP-3完整性能指标系统，包含所有组件的监控指标
*/

use prometheus::{
    Counter, Histogram, IntGauge, Gauge, HistogramOpts, Opts, 
    Registry, TextEncoder,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// 综合指标系统
pub struct ComprehensiveMetrics {
    registry: Registry,
    
    // === 系统级指标 ===
    pub system_uptime: IntGauge,
    pub system_memory_usage: IntGauge,
    pub system_cpu_usage: Gauge,
    pub system_disk_usage: IntGauge,
    
    // === 连接池指标 ===
    pub pool_connections_active: IntGauge,
    pub pool_connections_idle: IntGauge,
    pub pool_acquire_duration: Histogram,
    pub pool_acquire_total: Counter,
    pub pool_acquire_timeouts: Counter,
    
    // === 熔断器指标 ===
    pub circuit_breaker_state: IntGauge, // 0=Closed, 1=Open, 2=Half-Open
    pub circuit_breaker_failures: Counter,
    pub circuit_breaker_successes: Counter,
    pub circuit_breaker_timeouts: Counter,
    pub circuit_breaker_trips: Counter,
    
    // === 背压控制指标 ===
    pub backpressure_pause_events: Counter,
    pub backpressure_resume_events: Counter,
    pub backpressure_queue_depth: IntGauge,
    pub backpressure_queue_utilization: Gauge,
    
    // === FrameBus指标 ===
    pub framebus_frames_published: Counter,
    pub framebus_frames_consumed: Counter,
    pub framebus_frames_dropped: Counter,
    pub framebus_ring_utilization: Gauge,
    pub framebus_consumer_lag: IntGauge,
    
    // === WAL指标 ===
    pub wal_size_bytes: IntGauge,
    pub wal_write_duration: Histogram,
    pub wal_flush_duration: Histogram,
    pub wal_gc_runs: Counter,
    pub wal_compactions: Counter,
    pub wal_batch_size: Histogram,
    
    // === 驱动管理器指标 ===
    pub drivers_active: IntGauge,
    pub drivers_failed: IntGauge,
    pub driver_restart_total: Counter,
    pub driver_execution_duration: Histogram,
    
    // === 协议桥接指标 ===
    pub bridge_connections: IntGauge,
    pub bridge_requests_total: Counter,
    pub bridge_requests_failed: Counter,
    pub bridge_response_duration: Histogram,
    pub bridge_data_points: IntGauge,
    
    // === MQTT连接器指标 ===
    pub mqtt_messages_sent: Counter,
    pub mqtt_messages_received: Counter,
    pub mqtt_connection_state: IntGauge, // 0=Disconnected, 1=Connected
    pub mqtt_reconnects: Counter,
    pub mqtt_publish_duration: Histogram,
    
    // === REST API指标 ===
    pub api_requests_total: Counter,
    pub api_requests_duration: Histogram,
    pub api_auth_failures: Counter,
    pub api_rate_limit_hits: Counter,
    
    // === 插件系统指标 ===
    pub plugins_loaded: IntGauge,
    pub plugin_execution_duration: Histogram,
    pub plugin_errors: Counter,
    pub plugin_memory_usage: IntGauge,
    
    // === 业务指标 ===
    pub data_points_total: IntGauge,
    pub data_updates_per_second: Gauge,
    pub command_execution_duration: Histogram,
    pub command_success_rate: Gauge,
    
    // === 错误和警告指标 ===
    pub errors_total: Counter,
    pub warnings_total: Counter,
    pub critical_errors: Counter,
    
    // 动态标签指标
    labeled_metrics: Arc<RwLock<HashMap<String, Box<dyn prometheus::core::Collector + Send + Sync>>>>,
}

impl ComprehensiveMetrics {
    /// 创建新的综合指标系统
    pub fn new() -> prometheus::Result<Self> {
        let registry = Registry::new();
        
        // === 系统级指标 ===
        let system_uptime = IntGauge::with_opts(
            Opts::new("system_uptime_seconds", "System uptime in seconds")
        )?;
        registry.register(Box::new(system_uptime.clone()))?;
        
        let system_memory_usage = IntGauge::with_opts(
            Opts::new("system_memory_usage_bytes", "System memory usage in bytes")
        )?;
        registry.register(Box::new(system_memory_usage.clone()))?;
        
        let system_cpu_usage = Gauge::with_opts(
            Opts::new("system_cpu_usage_percent", "System CPU usage percentage")
        )?;
        registry.register(Box::new(system_cpu_usage.clone()))?;
        
        let system_disk_usage = IntGauge::with_opts(
            Opts::new("system_disk_usage_bytes", "System disk usage in bytes")
        )?;
        registry.register(Box::new(system_disk_usage.clone()))?;
        
        // === 连接池指标 ===
        let pool_connections_active = IntGauge::with_opts(
            Opts::new("pool_connections_active", "Active connections in pool")
        )?;
        registry.register(Box::new(pool_connections_active.clone()))?;
        
        let pool_connections_idle = IntGauge::with_opts(
            Opts::new("pool_connections_idle", "Idle connections in pool")
        )?;
        registry.register(Box::new(pool_connections_idle.clone()))?;
        
        let pool_acquire_duration = Histogram::with_opts(
            HistogramOpts::new(
                "pool_acquire_duration_microseconds",
                "Connection acquire duration in microseconds (MVP-3: ≤1µs Hot Acquire)"
            ).buckets(vec![0.5, 1.0, 2.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0])
        )?;
        registry.register(Box::new(pool_acquire_duration.clone()))?;
        
        let pool_acquire_total = Counter::with_opts(
            Opts::new("pool_acquire_total", "Total connection acquisitions")
        )?;
        registry.register(Box::new(pool_acquire_total.clone()))?;
        
        let pool_acquire_timeouts = Counter::with_opts(
            Opts::new("pool_acquire_timeouts", "Connection acquire timeouts")
        )?;
        registry.register(Box::new(pool_acquire_timeouts.clone()))?;
        
        // === 熔断器指标 ===
        let circuit_breaker_state = IntGauge::with_opts(
            Opts::new("circuit_breaker_state", "Circuit breaker state (0=Closed, 1=Open, 2=Half-Open)")
        )?;
        registry.register(Box::new(circuit_breaker_state.clone()))?;
        
        let circuit_breaker_failures = Counter::with_opts(
            Opts::new("circuit_breaker_failures_total", "Circuit breaker failure count")
        )?;
        registry.register(Box::new(circuit_breaker_failures.clone()))?;
        
        let circuit_breaker_successes = Counter::with_opts(
            Opts::new("circuit_breaker_successes_total", "Circuit breaker success count")
        )?;
        registry.register(Box::new(circuit_breaker_successes.clone()))?;
        
        let circuit_breaker_timeouts = Counter::with_opts(
            Opts::new("circuit_breaker_timeouts_total", "Circuit breaker timeout count")
        )?;
        registry.register(Box::new(circuit_breaker_timeouts.clone()))?;
        
        let circuit_breaker_trips = Counter::with_opts(
            Opts::new("circuit_breaker_trips_total", "Circuit breaker trip count")
        )?;
        registry.register(Box::new(circuit_breaker_trips.clone()))?;
        
        // === 背压控制指标 ===
        let backpressure_pause_events = Counter::with_opts(
            Opts::new("backpressure_pause_events_total", "Backpressure pause events (80% threshold)")
        )?;
        registry.register(Box::new(backpressure_pause_events.clone()))?;
        
        let backpressure_resume_events = Counter::with_opts(
            Opts::new("backpressure_resume_events_total", "Backpressure resume events (60% threshold)")
        )?;
        registry.register(Box::new(backpressure_resume_events.clone()))?;
        
        let backpressure_queue_depth = IntGauge::with_opts(
            Opts::new("backpressure_queue_depth", "Current queue depth")
        )?;
        registry.register(Box::new(backpressure_queue_depth.clone()))?;
        
        let backpressure_queue_utilization = Gauge::with_opts(
            Opts::new("backpressure_queue_utilization_percent", "Queue utilization percentage")
        )?;
        registry.register(Box::new(backpressure_queue_utilization.clone()))?;
        
        // === FrameBus指标 ===
        let framebus_frames_published = Counter::with_opts(
            Opts::new("framebus_frames_published_total", "Total frames published to FrameBus")
        )?;
        registry.register(Box::new(framebus_frames_published.clone()))?;
        
        let framebus_frames_consumed = Counter::with_opts(
            Opts::new("framebus_frames_consumed_total", "Total frames consumed from FrameBus")
        )?;
        registry.register(Box::new(framebus_frames_consumed.clone()))?;
        
        let framebus_frames_dropped = Counter::with_opts(
            Opts::new("framebus_frames_dropped_total", "Total frames dropped")
        )?;
        registry.register(Box::new(framebus_frames_dropped.clone()))?;
        
        let framebus_ring_utilization = Gauge::with_opts(
            Opts::new("framebus_ring_utilization_percent", "Ring buffer utilization percentage")
        )?;
        registry.register(Box::new(framebus_ring_utilization.clone()))?;
        
        let framebus_consumer_lag = IntGauge::with_opts(
            Opts::new("framebus_consumer_lag", "Consumer lag in frames")
        )?;
        registry.register(Box::new(framebus_consumer_lag.clone()))?;
        
        // === WAL指标 ===
        let wal_size_bytes = IntGauge::with_opts(
            Opts::new("wal_size_bytes", "WAL size in bytes (MVP-3: 4GiB auto-compaction)")
        )?;
        registry.register(Box::new(wal_size_bytes.clone()))?;
        
        let wal_write_duration = Histogram::with_opts(
            HistogramOpts::new(
                "wal_write_duration_milliseconds",
                "WAL write duration in milliseconds"
            ).buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0])
        )?;
        registry.register(Box::new(wal_write_duration.clone()))?;
        
        let wal_flush_duration = Histogram::with_opts(
            HistogramOpts::new(
                "wal_flush_duration_milliseconds",
                "WAL flush duration in milliseconds (MVP-3: 5k fps flush <5ms)"
            ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 50.0])
        )?;
        registry.register(Box::new(wal_flush_duration.clone()))?;
        
        let wal_gc_runs = Counter::with_opts(
            Opts::new("wal_gc_runs_total", "WAL garbage collection runs")
        )?;
        registry.register(Box::new(wal_gc_runs.clone()))?;
        
        let wal_compactions = Counter::with_opts(
            Opts::new("wal_compactions_total", "WAL compaction runs")
        )?;
        registry.register(Box::new(wal_compactions.clone()))?;
        
        let wal_batch_size = Histogram::with_opts(
            HistogramOpts::new(
                "wal_batch_size",
                "WAL batch size (MVP-3: 100ms batch timeout)"
            ).buckets(vec![1.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0])
        )?;
        registry.register(Box::new(wal_batch_size.clone()))?;
        
        // === 驱动管理器指标 ===
        let drivers_active = IntGauge::with_opts(
            Opts::new("drivers_active", "Number of active drivers")
        )?;
        registry.register(Box::new(drivers_active.clone()))?;
        
        let drivers_failed = IntGauge::with_opts(
            Opts::new("drivers_failed", "Number of failed drivers")
        )?;
        registry.register(Box::new(drivers_failed.clone()))?;
        
        let driver_restart_total = Counter::with_opts(
            Opts::new("driver_restart_total", "Total driver restarts")
        )?;
        registry.register(Box::new(driver_restart_total.clone()))?;
        
        let driver_execution_duration = Histogram::with_opts(
            HistogramOpts::new(
                "driver_execution_duration_milliseconds",
                "Driver execution duration in milliseconds"
            ).buckets(vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0])
        )?;
        registry.register(Box::new(driver_execution_duration.clone()))?;
        
        // === 协议桥接指标 ===
        let bridge_connections = IntGauge::with_opts(
            Opts::new("bridge_connections", "Number of protocol bridge connections")
        )?;
        registry.register(Box::new(bridge_connections.clone()))?;
        
        let bridge_requests_total = Counter::with_opts(
            Opts::new("bridge_requests_total", "Total protocol bridge requests")
        )?;
        registry.register(Box::new(bridge_requests_total.clone()))?;
        
        let bridge_requests_failed = Counter::with_opts(
            Opts::new("bridge_requests_failed_total", "Failed protocol bridge requests")
        )?;
        registry.register(Box::new(bridge_requests_failed.clone()))?;
        
        let bridge_response_duration = Histogram::with_opts(
            HistogramOpts::new(
                "bridge_response_duration_milliseconds",
                "Protocol bridge response duration in milliseconds"
            ).buckets(vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0])
        )?;
        registry.register(Box::new(bridge_response_duration.clone()))?;
        
        let bridge_data_points = IntGauge::with_opts(
            Opts::new("bridge_data_points", "Number of data points in protocol bridges")
        )?;
        registry.register(Box::new(bridge_data_points.clone()))?;
        
        // === MQTT连接器指标 ===
        let mqtt_messages_sent = Counter::with_opts(
            Opts::new("mqtt_messages_sent_total", "Total MQTT messages sent")
        )?;
        registry.register(Box::new(mqtt_messages_sent.clone()))?;
        
        let mqtt_messages_received = Counter::with_opts(
            Opts::new("mqtt_messages_received_total", "Total MQTT messages received")
        )?;
        registry.register(Box::new(mqtt_messages_received.clone()))?;
        
        let mqtt_connection_state = IntGauge::with_opts(
            Opts::new("mqtt_connection_state", "MQTT connection state (0=Disconnected, 1=Connected)")
        )?;
        registry.register(Box::new(mqtt_connection_state.clone()))?;
        
        let mqtt_reconnects = Counter::with_opts(
            Opts::new("mqtt_reconnects_total", "Total MQTT reconnections")
        )?;
        registry.register(Box::new(mqtt_reconnects.clone()))?;
        
        let mqtt_publish_duration = Histogram::with_opts(
            HistogramOpts::new(
                "mqtt_publish_duration_milliseconds",
                "MQTT publish duration in milliseconds"
            ).buckets(vec![0.5, 1.0, 5.0, 10.0, 50.0, 100.0, 500.0])
        )?;
        registry.register(Box::new(mqtt_publish_duration.clone()))?;
        
        // === REST API指标 ===
        let api_requests_total = Counter::with_opts(
            Opts::new("api_requests_total", "Total API requests")
        )?;
        registry.register(Box::new(api_requests_total.clone()))?;
        
        let api_requests_duration = Histogram::with_opts(
            HistogramOpts::new(
                "api_requests_duration_milliseconds",
                "API request duration in milliseconds"
            ).buckets(vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0])
        )?;
        registry.register(Box::new(api_requests_duration.clone()))?;
        
        let api_auth_failures = Counter::with_opts(
            Opts::new("api_auth_failures_total", "Total API authentication failures")
        )?;
        registry.register(Box::new(api_auth_failures.clone()))?;
        
        let api_rate_limit_hits = Counter::with_opts(
            Opts::new("api_rate_limit_hits_total", "Total API rate limit hits")
        )?;
        registry.register(Box::new(api_rate_limit_hits.clone()))?;
        
        // === 插件系统指标 ===
        let plugins_loaded = IntGauge::with_opts(
            Opts::new("plugins_loaded", "Number of loaded plugins")
        )?;
        registry.register(Box::new(plugins_loaded.clone()))?;
        
        let plugin_execution_duration = Histogram::with_opts(
            HistogramOpts::new(
                "plugin_execution_duration_milliseconds",
                "Plugin execution duration in milliseconds"
            ).buckets(vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0])
        )?;
        registry.register(Box::new(plugin_execution_duration.clone()))?;
        
        let plugin_errors = Counter::with_opts(
            Opts::new("plugin_errors_total", "Total plugin errors")
        )?;
        registry.register(Box::new(plugin_errors.clone()))?;
        
        let plugin_memory_usage = IntGauge::with_opts(
            Opts::new("plugin_memory_usage_bytes", "Plugin memory usage in bytes")
        )?;
        registry.register(Box::new(plugin_memory_usage.clone()))?;
        
        // === 业务指标 ===
        let data_points_total = IntGauge::with_opts(
            Opts::new("data_points_total", "Total number of data points")
        )?;
        registry.register(Box::new(data_points_total.clone()))?;
        
        let data_updates_per_second = Gauge::with_opts(
            Opts::new("data_updates_per_second", "Data updates per second")
        )?;
        registry.register(Box::new(data_updates_per_second.clone()))?;
        
        let command_execution_duration = Histogram::with_opts(
            HistogramOpts::new(
                "command_execution_duration_milliseconds",
                "Command execution duration in milliseconds"
            ).buckets(vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0])
        )?;
        registry.register(Box::new(command_execution_duration.clone()))?;
        
        let command_success_rate = Gauge::with_opts(
            Opts::new("command_success_rate_percent", "Command success rate percentage")
        )?;
        registry.register(Box::new(command_success_rate.clone()))?;
        
        // === 错误和警告指标 ===
        let errors_total = Counter::with_opts(
            Opts::new("errors_total", "Total error count")
        )?;
        registry.register(Box::new(errors_total.clone()))?;
        
        let warnings_total = Counter::with_opts(
            Opts::new("warnings_total", "Total warning count")
        )?;
        registry.register(Box::new(warnings_total.clone()))?;
        
        let critical_errors = Counter::with_opts(
            Opts::new("critical_errors_total", "Total critical error count")
        )?;
        registry.register(Box::new(critical_errors.clone()))?;
        
        Ok(Self {
            registry,
            
            // 系统级指标
            system_uptime,
            system_memory_usage,
            system_cpu_usage,
            system_disk_usage,
            
            // 连接池指标
            pool_connections_active,
            pool_connections_idle,
            pool_acquire_duration,
            pool_acquire_total,
            pool_acquire_timeouts,
            
            // 熔断器指标
            circuit_breaker_state,
            circuit_breaker_failures,
            circuit_breaker_successes,
            circuit_breaker_timeouts,
            circuit_breaker_trips,
            
            // 背压控制指标
            backpressure_pause_events,
            backpressure_resume_events,
            backpressure_queue_depth,
            backpressure_queue_utilization,
            
            // FrameBus指标
            framebus_frames_published,
            framebus_frames_consumed,
            framebus_frames_dropped,
            framebus_ring_utilization,
            framebus_consumer_lag,
            
            // WAL指标
            wal_size_bytes,
            wal_write_duration,
            wal_flush_duration,
            wal_gc_runs,
            wal_compactions,
            wal_batch_size,
            
            // 驱动管理器指标
            drivers_active,
            drivers_failed,
            driver_restart_total,
            driver_execution_duration,
            
            // 协议桥接指标
            bridge_connections,
            bridge_requests_total,
            bridge_requests_failed,
            bridge_response_duration,
            bridge_data_points,
            
            // MQTT连接器指标
            mqtt_messages_sent,
            mqtt_messages_received,
            mqtt_connection_state,
            mqtt_reconnects,
            mqtt_publish_duration,
            
            // REST API指标
            api_requests_total,
            api_requests_duration,
            api_auth_failures,
            api_rate_limit_hits,
            
            // 插件系统指标
            plugins_loaded,
            plugin_execution_duration,
            plugin_errors,
            plugin_memory_usage,
            
            // 业务指标
            data_points_total,
            data_updates_per_second,
            command_execution_duration,
            command_success_rate,
            
            // 错误和警告指标
            errors_total,
            warnings_total,
            critical_errors,
            
            labeled_metrics: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 导出Prometheus格式的指标
    pub fn export(&self) -> prometheus::Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode_to_string(&metric_families)
    }

    /// 获取指标注册表
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// 获取系统概览
    pub fn get_system_overview(&self) -> SystemOverview {
        SystemOverview {
            uptime: self.system_uptime.get() as u64,
            memory_usage: self.system_memory_usage.get() as u64,
            cpu_usage: self.system_cpu_usage.get(),
            disk_usage: self.system_disk_usage.get() as u64,
            active_connections: self.pool_connections_active.get() as u32,
            active_drivers: self.drivers_active.get() as u32,
            total_data_points: self.data_points_total.get() as u32,
            error_rate: self.errors_total.get(),
        }
    }
}

/// 系统概览结构
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemOverview {
    pub uptime: u64,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub disk_usage: u64,
    pub active_connections: u32,
    pub active_drivers: u32,
    pub total_data_points: u32,
    pub error_rate: f64,
}

/// 全局指标实例
use once_cell::sync::Lazy;

pub static COMPREHENSIVE_METRICS: Lazy<ComprehensiveMetrics> = Lazy::new(|| {
    ComprehensiveMetrics::new().expect("Failed to initialize comprehensive metrics")
});

/// 初始化综合指标系统
pub fn init_comprehensive_metrics() {
    Lazy::force(&COMPREHENSIVE_METRICS);
    tracing::info!("Comprehensive metrics system initialized with {} metrics", 
        COMPREHENSIVE_METRICS.registry().gather().len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comprehensive_metrics_creation() {
        let metrics = ComprehensiveMetrics::new().unwrap();
        
        // 测试指标注册
        assert!(metrics.export().is_ok());
        
        // 测试指标更新
        metrics.system_uptime.set(3600);
        metrics.pool_connections_active.set(10);
        metrics.framebus_frames_published.inc();
        
        let overview = metrics.get_system_overview();
        assert_eq!(overview.uptime, 3600);
        assert_eq!(overview.active_connections, 10);
    }

    #[test]
    fn test_metrics_export() {
        let metrics = ComprehensiveMetrics::new().unwrap();
        
        // 设置一些指标值
        metrics.system_memory_usage.set(1024 * 1024 * 1024); // 1GB
        metrics.pool_acquire_duration.observe(0.5); // 0.5 microseconds
        metrics.circuit_breaker_state.set(0); // Closed
        
        let exported = metrics.export().unwrap();
        
        // 验证导出的指标包含预期内容
        assert!(exported.contains("system_memory_usage_bytes"));
        assert!(exported.contains("pool_acquire_duration_microseconds"));
        assert!(exported.contains("circuit_breaker_state"));
    }

    #[test]
    fn test_mvp3_specific_metrics() {
        let metrics = ComprehensiveMetrics::new().unwrap();
        
        // 测试MVP-3特定指标
        
        // Hot Acquire ≤1µs
        metrics.pool_acquire_duration.observe(0.8); // 0.8 microseconds
        
        // 80%/60% 背压阈值
        metrics.backpressure_queue_utilization.set(85.0);
        metrics.backpressure_pause_events.inc();
        
        // 5k fps flush <5ms
        metrics.wal_flush_duration.observe(3.2); // 3.2 milliseconds
        
        // 4GiB自动压缩
        metrics.wal_size_bytes.set(4 * 1024 * 1024 * 1024); // 4GiB
        metrics.wal_compactions.inc();
        
        // 100ms批处理
        metrics.wal_batch_size.observe(50.0); // 50 items in batch
        
        let exported = metrics.export().unwrap();
        assert!(exported.contains("pool_acquire_duration_microseconds"));
        assert!(exported.contains("backpressure_pause_events_total"));
        assert!(exported.contains("wal_flush_duration_milliseconds"));
    }

    #[test]
    fn test_all_mvp3_metrics_coverage() {
        let metrics = ComprehensiveMetrics::new().unwrap();
        let exported = metrics.export().unwrap();
        
        // 验证所有MVP-3核心指标都已注册
        let required_metrics = [
            // 系统级
            "system_uptime_seconds",
            "system_memory_usage_bytes", 
            "system_cpu_usage_percent",
            
            // 连接池（bb8默认4个连接）
            "pool_connections_active",
            "pool_connections_idle",
            "pool_acquire_duration_microseconds", // ≤1µs Hot Acquire
            
            // 三态熔断器
            "circuit_breaker_state", // 0=Closed, 1=Open, 2=Half-Open
            "circuit_breaker_failures_total",
            "circuit_breaker_trips_total",
            
            // 背压控制（80%/60%阈值）
            "backpressure_pause_events_total",
            "backpressure_resume_events_total",
            "backpressure_queue_utilization_percent",
            
            // FrameBus WAL（4GiB自动压缩，100ms批处理）
            "wal_size_bytes",
            "wal_flush_duration_milliseconds", // 5k fps flush <5ms
            "wal_batch_size",
            "wal_compactions_total",
            "wal_gc_runs_total",
            
            // Protocol-Bridge插件系统
            "bridge_connections",
            "bridge_requests_total",
            "bridge_data_points",
            
            // RBAC与REST API
            "api_requests_total",
            "api_auth_failures_total",
            
            // 业务指标
            "data_updates_per_second",
            "command_success_rate_percent",
        ];
        
        for metric in &required_metrics {
            assert!(exported.contains(metric), "Missing required MVP-3 metric: {}", metric);
        }
        
        println!("✓ All {} MVP-3 core metrics are registered", required_metrics.len());
    }
}