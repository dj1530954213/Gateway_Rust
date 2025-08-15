//! FrameBus配置 - 高性能优化版本
//! 
//! 支持异步写入、背压控制和批量处理优化

use std::path::PathBuf;

/// 性能优化配置预设
pub struct PerformancePresets;

impl PerformancePresets {
    /// 高吞吐量配置 (适用于5k+ fps)
    pub fn high_throughput() -> BusCfg {
        BusCfg {
            ring_pow: 21, // 2M ring buffer
            pause_hi: 0.90,
            resume_lo: 0.75,
            wal_dir: PathBuf::from(std::env::var("WAL_DIR").unwrap_or_else(|_| "/tmp/gateway_wal".to_string())),
            wal_flush_ms: 5, // 5ms极低延迟
            wal_max_bytes: 16 * 1024 * 1024 * 1024, // 16GB
            high_performance_mode: true,
            async_write_queue_size: 100000, // 100K队列
            backpressure_threshold: 0.95,
        }
    }
    
    /// 低延迟配置 (适用于<1ms延迟要求)
    pub fn low_latency() -> BusCfg {
        BusCfg {
            ring_pow: 19, // 512K ring buffer
            pause_hi: 0.80,
            resume_lo: 0.60,
            wal_dir: PathBuf::from(std::env::var("WAL_DIR").unwrap_or_else(|_| "/tmp/gateway_wal".to_string())),
            wal_flush_ms: 1, // 1ms极低延迟
            wal_max_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            high_performance_mode: true,
            async_write_queue_size: 20000,
            backpressure_threshold: 0.85,
        }
    }
    
    /// 内存优先配置 (适用于内存限制环境)
    pub fn memory_optimized() -> BusCfg {
        BusCfg {
            ring_pow: 17, // 128K ring buffer
            pause_hi: 0.75,
            resume_lo: 0.50,
            wal_dir: PathBuf::from(std::env::var("WAL_DIR").unwrap_or_else(|_| "/tmp/gateway_wal".to_string())),
            wal_flush_ms: 20,
            wal_max_bytes: 2 * 1024 * 1024 * 1024, // 2GB
            high_performance_mode: false,
            async_write_queue_size: 5000,
            backpressure_threshold: 0.80,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BusCfg {
    /// Ring缓冲区大小 (2的指数)
    pub ring_pow: u8,
    /// 暂停阈值 (百分比)
    pub pause_hi: f32,
    /// 恢复阈值 (百分比)  
    pub resume_lo: f32,
    /// WAL目录
    pub wal_dir: PathBuf,
    /// WAL刷新间隔 (毫秒)
    pub wal_flush_ms: u64,
    /// WAL最大字节数
    pub wal_max_bytes: u64,
    /// 高性能模式
    pub high_performance_mode: bool,
    /// 异步写入队列大小
    pub async_write_queue_size: usize,
    /// 背压控制阈值
    pub backpressure_threshold: f32,
}

impl BusCfg {
    /// 获取Ring缓冲区实际容量
    pub fn ring_capacity(&self) -> usize {
        1 << self.ring_pow
    }
    
    /// 获取暂停阈值对应的绝对数量
    pub fn pause_threshold(&self) -> usize {
        (self.ring_capacity() as f32 * self.pause_hi) as usize
    }
    
    /// 获取恢复阈值对应的绝对数量
    pub fn resume_threshold(&self) -> usize {
        (self.ring_capacity() as f32 * self.resume_lo) as usize
    }
    
    /// 验证配置合理性
    pub fn validate(&self) -> Result<(), String> {
        if self.ring_pow < 10 || self.ring_pow > 25 {
            return Err(format!("ring_pow {} out of range [10, 25]", self.ring_pow));
        }
        
        if self.pause_hi <= self.resume_lo {
            return Err("pause_hi must be greater than resume_lo".to_string());
        }
        
        if self.wal_flush_ms == 0 {
            return Err("wal_flush_ms must be greater than 0".to_string());
        }
        
        Ok(())
    }
}

impl Default for BusCfg {
    fn default() -> Self {
        Self {
            ring_pow: 20,  // 优化: 1M容量提升吞吐量
            pause_hi: 0.85, // 优化: 提高暂停阈值减少频繁背压
            resume_lo: 0.70, // 优化: 提高恢复阈值
            wal_dir: PathBuf::from(std::env::var("WAL_DIR").unwrap_or_else(|_| "/tmp/gateway_wal".to_string())),
            wal_flush_ms: 10, // 优化: 10ms刷新间隔
            wal_max_bytes: 8 * 1024 * 1024 * 1024, // 优化: 8GB支持更大数据量
            high_performance_mode: true, // 启用高性能模式
            async_write_queue_size: 50000, // 50K异步写入队列
            backpressure_threshold: 0.90, // 90%触发背压
        }
    }
}