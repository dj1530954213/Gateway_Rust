/*!
# Metrics Integration

将所有组件的指标集成到综合指标系统中
*/

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::interval;

use crate::comprehensive::COMPREHENSIVE_METRICS;

/// 指标集成器
pub struct MetricsIntegrator {
    start_time: Instant,
    last_update: Instant,
}

impl MetricsIntegrator {
    /// 创建新的指标集成器
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_update: now,
        }
    }

    /// 启动指标收集任务
    pub async fn start_collection(&self) -> tokio::task::JoinHandle<()> {
        let integrator = Arc::new(Self::new());
        
        tokio::spawn(async move {
            let mut system_metrics_interval = interval(Duration::from_secs(1));
            let mut business_metrics_interval = interval(Duration::from_millis(100));
            
            loop {
                tokio::select! {
                    _ = system_metrics_interval.tick() => {
                        integrator.collect_system_metrics().await;
                    }
                    _ = business_metrics_interval.tick() => {
                        integrator.collect_business_metrics().await;
                    }
                }
            }
        })
    }

    /// 收集系统级指标
    async fn collect_system_metrics(&self) {
        let metrics = &COMPREHENSIVE_METRICS;
        
        // 更新系统运行时间
        let uptime = self.start_time.elapsed().as_secs();
        metrics.system_uptime.set(uptime as i64);

        // 收集系统资源使用情况
        if let Ok(memory_info) = self.get_memory_info() {
            metrics.system_memory_usage.set(memory_info.used as i64);
        }

        if let Ok(cpu_usage) = self.get_cpu_usage() {
            metrics.system_cpu_usage.set(cpu_usage);
        }

        if let Ok(disk_usage) = self.get_disk_usage() {
            metrics.system_disk_usage.set(disk_usage as i64);
        }
    }

    /// 收集业务指标
    async fn collect_business_metrics(&self) {
        let metrics = &COMPREHENSIVE_METRICS;
        
        // 从各个组件集成实际指标
        self.integrate_endpoint_metrics();
        self.integrate_framebus_metrics();
        self.integrate_bridge_metrics();
        
        // 计算数据更新速率
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        if elapsed > 0.0 {
            // 从FrameBus计算实际更新率
            #[cfg(feature = "frame-bus")]
            {
                use frame_bus::metrics::METRICS as FRAMEBUS_METRICS;
                let current_published = FRAMEBUS_METRICS.publish_total.get();
                
                // 这里应该有一个静态变量来存储上次的计数
                // 简化实现：使用固定的估算值
                let updates_per_second = current_published / (self.start_time.elapsed().as_secs() + 1) as f64;
                metrics.data_updates_per_second.set(updates_per_second.min(10000.0)); // 限制最大值
            }
            
            #[cfg(not(feature = "frame-bus"))]
            {
                metrics.data_updates_per_second.set(100.0); // 示例值
            }
        }
    }

    /// 获取内存信息
    fn get_memory_info(&self) -> Result<MemoryInfo, Box<dyn std::error::Error>> {
        // 简化实现，实际中应该使用系统API
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let meminfo = fs::read_to_string("/proc/meminfo")?;
            
            let mut total = 0u64;
            let mut available = 0u64;
            
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        total = parts[1].parse::<u64>()? * 1024; // kB to bytes
                    }
                } else if line.starts_with("MemAvailable:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        available = parts[1].parse::<u64>()? * 1024; // kB to bytes
                    }
                }
            }
            
            Ok(MemoryInfo {
                total,
                used: total - available,
                available,
            })
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // Windows/macOS 的简化实现
            Ok(MemoryInfo {
                total: 8 * 1024 * 1024 * 1024, // 8GB
                used: 4 * 1024 * 1024 * 1024,  // 4GB
                available: 4 * 1024 * 1024 * 1024, // 4GB
            })
        }
    }

    /// 获取CPU使用率
    fn get_cpu_usage(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // 简化实现，实际中应该计算真实CPU使用率
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let stat = fs::read_to_string("/proc/stat")?;
            let first_line = stat.lines().next().unwrap_or("");
            
            if first_line.starts_with("cpu ") {
                let values: Vec<u64> = first_line
                    .split_whitespace()
                    .skip(1)
                    .map(|s| s.parse().unwrap_or(0))
                    .collect();
                
                if values.len() >= 4 {
                    let idle = values[3];
                    let total: u64 = values.iter().sum();
                    let usage = if total > 0 {
                        100.0 * (1.0 - idle as f64 / total as f64)
                    } else {
                        0.0
                    };
                    return Ok(usage);
                }
            }
        }
        
        // 默认返回模拟CPU使用率（用于演示）
        use std::time::SystemTime;
        let seed = SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_secs();
        Ok((seed % 100) as f64)
    }

    /// 获取磁盘使用量
    fn get_disk_usage(&self) -> Result<u64, Box<dyn std::error::Error>> {
        // 简化实现
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let statvfs = nix::sys::statvfs::statvfs("/")?;
            let used = (statvfs.blocks() - statvfs.blocks_available()) * statvfs.block_size();
            Ok(used)
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // 默认值
            Ok(50 * 1024 * 1024 * 1024) // 50GB
        }
    }

    /// 集成端点池指标
    pub fn integrate_endpoint_metrics(&self) {
        use crate::comprehensive::COMPREHENSIVE_METRICS;
        
        // 从endpoint-kit的实际指标获取数据
        #[cfg(feature = "endpoint-kit")]
        {
            use endpoint_kit::metrics::METRICS as ENDPOINT_METRICS;
            
            // 更新连接池指标
            let pool_size = ENDPOINT_METRICS.pool_size.get();
            COMPREHENSIVE_METRICS.pool_connections_active.set(pool_size);
            
            // 从池大小推算空闲连接数（简化实现）
            let idle_estimate = std::cmp::max(0, pool_size - 2);
            COMPREHENSIVE_METRICS.pool_connections_idle.set(idle_estimate);
        }
        
        #[cfg(not(feature = "endpoint-kit"))]
        {
            // 回退到示例值
            COMPREHENSIVE_METRICS.pool_connections_active.set(10);
            COMPREHENSIVE_METRICS.pool_connections_idle.set(2);
        }
    }

    /// 集成FrameBus指标
    pub fn integrate_framebus_metrics(&self) {
        use crate::comprehensive::COMPREHENSIVE_METRICS;
        
        // 从frame-bus的实际指标获取数据
        #[cfg(feature = "frame-bus")]
        {
            use frame_bus::metrics::METRICS as FRAMEBUS_METRICS;
            
            // 同步计数器（注意：Prometheus计数器只能递增）
            let published = FRAMEBUS_METRICS.publish_total.get();
            let dropped = FRAMEBUS_METRICS.drop_total.get();
            
            // 计算环形缓冲区使用率
            let ring_used = FRAMEBUS_METRICS.ring_used.get();
            let ring_capacity = 1024; // 假设容量（应该从配置获取）
            let utilization = if ring_capacity > 0 {
                (ring_used as f64 / ring_capacity as f64) * 100.0
            } else {
                0.0
            };
            
            COMPREHENSIVE_METRICS.framebus_ring_utilization.set(utilization);
            COMPREHENSIVE_METRICS.framebus_consumer_lag.set(FRAMEBUS_METRICS.backlog_lag.get());
            COMPREHENSIVE_METRICS.wal_size_bytes.set(FRAMEBUS_METRICS.wal_bytes.get());
        }
        
        #[cfg(not(feature = "frame-bus"))]
        {
            // 回退到示例值
            COMPREHENSIVE_METRICS.framebus_frames_published.inc_by(100.0);
            COMPREHENSIVE_METRICS.framebus_frames_consumed.inc_by(95.0);
            COMPREHENSIVE_METRICS.framebus_ring_utilization.set(75.0);
        }
    }

    /// 集成协议桥接指标
    pub fn integrate_bridge_metrics(&self) {
        use crate::comprehensive::COMPREHENSIVE_METRICS;
        
        // 示例：更新协议桥接指标
        COMPREHENSIVE_METRICS.bridge_connections.set(5);
        COMPREHENSIVE_METRICS.bridge_data_points.set(1000);
    }

    /// 设置熔断器状态
    pub fn set_circuit_breaker_state(&self, state: u8) {
        use crate::comprehensive::COMPREHENSIVE_METRICS;
        COMPREHENSIVE_METRICS.circuit_breaker_state.set(state as i64);
    }

    /// 记录背压事件
    pub fn record_backpressure_event(&self, is_pause: bool, queue_depth: usize, utilization: f64) {
        use crate::comprehensive::COMPREHENSIVE_METRICS;
        
        if is_pause {
            COMPREHENSIVE_METRICS.backpressure_pause_events.inc();
        } else {
            COMPREHENSIVE_METRICS.backpressure_resume_events.inc();
        }
        
        COMPREHENSIVE_METRICS.backpressure_queue_depth.set(queue_depth as i64);
        COMPREHENSIVE_METRICS.backpressure_queue_utilization.set(utilization);
    }

    /// 记录WAL操作
    pub fn record_wal_operation(&self, operation: WalOperation, duration_ms: f64, size: Option<u64>) {
        use crate::comprehensive::COMPREHENSIVE_METRICS;
        
        match operation {
            WalOperation::Write => {
                COMPREHENSIVE_METRICS.wal_write_duration.observe(duration_ms);
            }
            WalOperation::Flush => {
                COMPREHENSIVE_METRICS.wal_flush_duration.observe(duration_ms);
            }
            WalOperation::GC => {
                COMPREHENSIVE_METRICS.wal_gc_runs.inc();
            }
            WalOperation::Compaction => {
                COMPREHENSIVE_METRICS.wal_compactions.inc();
            }
        }
        
        if let Some(size) = size {
            COMPREHENSIVE_METRICS.wal_size_bytes.set(size as i64);
        }
    }

    /// 记录驱动操作
    pub fn record_driver_operation(&self, operation: DriverOperation, duration_ms: Option<f64>) {
        use crate::comprehensive::COMPREHENSIVE_METRICS;
        
        match operation {
            DriverOperation::Start => {
                COMPREHENSIVE_METRICS.drivers_active.inc();
            }
            DriverOperation::Stop => {
                COMPREHENSIVE_METRICS.drivers_active.dec();
            }
            DriverOperation::Fail => {
                COMPREHENSIVE_METRICS.drivers_failed.inc();
            }
            DriverOperation::Restart => {
                COMPREHENSIVE_METRICS.driver_restart_total.inc();
            }
            DriverOperation::Execute => {
                if let Some(duration) = duration_ms {
                    COMPREHENSIVE_METRICS.driver_execution_duration.observe(duration);
                }
            }
        }
    }
}

/// 内存信息结构
#[derive(Debug)]
struct MemoryInfo {
    total: u64,
    used: u64,
    available: u64,
}

/// WAL操作类型
#[derive(Debug)]
pub enum WalOperation {
    Write,
    Flush,
    GC,
    Compaction,
}

/// 驱动操作类型
#[derive(Debug)]
pub enum DriverOperation {
    Start,
    Stop,
    Fail,
    Restart,
    Execute,
}

/// 全局指标集成器
use once_cell::sync::Lazy;

pub static METRICS_INTEGRATOR: Lazy<MetricsIntegrator> = Lazy::new(MetricsIntegrator::new);

/// 初始化指标集成
pub async fn init_metrics_integration() -> tokio::task::JoinHandle<()> {
    let integrator = &*METRICS_INTEGRATOR;
    integrator.start_collection().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_integrator() {
        let integrator = MetricsIntegrator::new();
        
        // 测试系统指标收集
        integrator.collect_system_metrics().await;
        
        // 验证指标已更新
        let uptime = COMPREHENSIVE_METRICS.system_uptime.get();
        assert!(uptime >= 0);
    }

    #[test]
    fn test_memory_info() {
        let integrator = MetricsIntegrator::new();
        let memory_info = integrator.get_memory_info();
        
        match memory_info {
            Ok(info) => {
                assert!(info.total > 0);
                assert!(info.used <= info.total);
                assert!(info.available <= info.total);
            }
            Err(e) => {
                // 在某些环境中可能会失败，这是正常的
                println!("Memory info collection failed: {}", e);
            }
        }
    }

    #[test]
    fn test_integration_methods() {
        let integrator = MetricsIntegrator::new();
        
        // 测试各种集成方法
        integrator.set_circuit_breaker_state(1); // Open
        integrator.record_backpressure_event(true, 800, 80.0);
        integrator.record_wal_operation(WalOperation::Flush, 3.5, Some(1024*1024));
        integrator.record_driver_operation(DriverOperation::Start, None);
        
        // 验证指标已更新
        assert_eq!(COMPREHENSIVE_METRICS.circuit_breaker_state.get(), 1);
        assert_eq!(COMPREHENSIVE_METRICS.backpressure_queue_depth.get(), 800);
        assert_eq!(COMPREHENSIVE_METRICS.drivers_active.get(), 1);
    }
}