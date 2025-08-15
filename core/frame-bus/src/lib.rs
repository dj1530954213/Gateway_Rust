//! FrameBus - L2 事件总线核心
//! 
//! 统一数据流转的中央总线，支持持久化和背压控制

mod generated;
pub mod envelope;
pub mod ring;
pub mod wal;
pub mod control;
pub mod config;
pub mod metrics;
pub mod filter;
pub mod command;
pub mod permissions;

pub use envelope::{DataFrame, CmdFrame, CmdAckFrame, FrameEnvelope, FrameKind, Value};
pub use ring::{FrameSender, FrameReceiver, FramePublisher, BatchConfig, AsyncBatchPublisher};
pub use filter::Filter;
pub use config::{BusCfg, PerformancePresets};
pub use wal::WAL;
pub use command::{CommandProcessor, CommandPriority, CommandStatus, CommandResult, PendingCommand};
pub use permissions::{PermissionManager, Permission, AccessControlEntry};

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{info, warn};

/// 初始化FrameBus全局实例
pub fn init<P: AsRef<Path>>(ring_size: usize, wal_dir: P) -> Result<(FrameSender, FrameReceiver)> {
    let cfg = BusCfg {
        ring_pow: (ring_size as f64).log2() as u8,
        wal_dir: wal_dir.as_ref().to_path_buf(),
        ..Default::default()
    };
    
    metrics::init_metrics();
    wal::init(&cfg.wal_dir)?;
    let (tx, rx) = ring::init(cfg)?;
    
    // 初始化批量发布器（使用默认配置）
    ring::init_batch_publisher(&tx, None)?;
    
    Ok((tx, rx))
}

/// 初始化FrameBus全局实例（带自定义批量配置）
pub fn init_with_batch_config<P: AsRef<Path>>(
    ring_size: usize, 
    wal_dir: P, 
    batch_config: BatchConfig
) -> Result<(FrameSender, FrameReceiver)> {
    let cfg = BusCfg {
        ring_pow: (ring_size as f64).log2() as u8,
        wal_dir: wal_dir.as_ref().to_path_buf(),
        ..Default::default()
    };
    
    metrics::init_metrics();
    wal::init(&cfg.wal_dir)?;
    let (tx, rx) = ring::init(cfg)?;
    
    // 初始化批量发布器（使用自定义配置）
    ring::init_batch_publisher(&tx, Some(batch_config))?;
    
    Ok((tx, rx))
}

/// 发布DataFrame到总线（传统方式）
pub fn publish_data(frame: DataFrame) -> Result<()> {
    let tx = ring::get_publisher()?;
    let publisher = ring::FramePublisher::new(tx.clone());
    publisher.send_data(frame)
}

/// 批量发布DataFrame到总线（高性能方式）
pub fn publish_data_batch(frames: Vec<DataFrame>) -> Result<()> {
    let tx = ring::get_publisher()?;
    let publisher = ring::FramePublisher::new_batched(tx.clone());
    publisher.send_data_batch(frames)
}

/// 获取批量发布器实例（用于频繁发布）
pub fn get_batch_publisher() -> Result<&'static ring::AsyncBatchPublisher> {
    ring::get_batch_publisher()
}

/// 初始化高性能版本FrameBus (新推荐接口)
pub fn init_high_performance<P: AsRef<Path>>(
    ring_size: usize, 
    wal_dir: P, 
    config_preset: Option<BusCfg>
) -> Result<(FrameSender, FrameReceiver)> {
    let mut cfg = config_preset.unwrap_or_else(|| PerformancePresets::high_throughput());
    
    // 更新ring_pow以匹配ring_size
    if ring_size > 0 {
        cfg.ring_pow = (ring_size as f64).log2() as u8;
    }
    cfg.wal_dir = wal_dir.as_ref().to_path_buf();
    
    // 验证配置有效性
    cfg.validate().map_err(|e| anyhow::anyhow!("Configuration validation failed: {}", e))?;
    
    metrics::init_metrics();
    
    // 使用新的WAL配置
    let wal_config = wal::WalConfig {
        batch_timeout: Duration::from_millis(1), // 1ms极低延迟
        batch_size_limit: 5000,
        write_queue_capacity: cfg.async_write_queue_size,
        write_workers: 2,
        async_sync: true,
        sync_interval: Duration::from_millis(cfg.wal_flush_ms),
        gc_interval: Duration::from_secs(30),
        auto_compact_threshold: cfg.wal_max_bytes,
        retain_frames: 10000,
        backpressure_threshold: (cfg.async_write_queue_size as f32 * cfg.backpressure_threshold) as usize,
    };
    
    wal::init_with_config(&cfg.wal_dir, wal_config)?;
    
    // 使用新的批量配置初始化
    let batch_config = BatchConfig {
        max_batch_size: 2000, // 提高批量大小
        flush_interval: Duration::from_millis(1), // 1ms极低延迟
        max_memory_bytes: 16 * 1024 * 1024, // 16MB缓冲
    };
    
    let (tx, rx) = ring::init_with_batch_config(cfg.clone(), Some(batch_config))?;
    
    info!("高性能FrameBus初始化完成: Ring={}, WAL={}, 批量={}", 
          cfg.ring_capacity(), wal_dir.as_ref().display(), 2000);
    
    Ok((tx, rx))
}

/// 初始化内存模式FrameBus (WSL2兼容性备选方案)
pub fn init_memory_mode<P: AsRef<Path>>(ring_size: usize, _wal_dir: P) -> Result<(FrameSender, FrameReceiver)> {
    info!("Initializing FrameBus in memory-only mode for WSL2 compatibility");
    
    let cfg = BusCfg {
        ring_pow: (ring_size as f64).log2() as u8,
        wal_dir: PathBuf::from("/tmp/memory_mode"), // 不实际使用，仅为满足配置要求
        ..PerformancePresets::memory_optimized()
    };
    
    // 验证配置有效性
    cfg.validate().map_err(|e| anyhow::anyhow!("Configuration validation failed: {}", e))?;
    
    metrics::init_metrics();
    
    // 初始化内存WAL
    wal::init_memory_wal()?;
    
    // 使用内存优化配置初始化Ring
    let batch_config = BatchConfig {
        max_batch_size: 500, // 内存模式下较小的批量大小
        flush_interval: Duration::from_millis(5), // 5ms刷新间隔
        max_memory_bytes: 4 * 1024 * 1024, // 4MB内存缓冲
    };
    
    let (tx, rx) = ring::init_with_batch_config(cfg.clone(), Some(batch_config))?;
    
    info!("内存模式FrameBus初始化完成: Ring={}, 批量大小={}", 
          cfg.ring_capacity(), 500);
    
    Ok((tx, rx))
}

/// 智能初始化FrameBus (自动降级到内存模式)
pub fn init_with_fallback<P: AsRef<Path>>(ring_size: usize, wal_dir: P) -> Result<(FrameSender, FrameReceiver)> {
    // 首先尝试高性能持久化模式
    match init_high_performance(ring_size, &wal_dir, None) {
        Ok((tx, rx)) => {
            info!("成功初始化高性能持久化FrameBus");
            Ok((tx, rx))
        },
        Err(e) => {
            warn!("持久化FrameBus初始化失败: {}", e);
            info!("降级到内存模式FrameBus...");
            
            // 降级到内存模式
            init_memory_mode(ring_size, wal_dir)
                .context("内存模式FrameBus初始化也失败")
        }
    }
}

/// 获取性能统计信息
pub fn get_performance_stats() -> Result<PerformanceStats> {
    let wal_stats = wal::get_performance_stats()?;
    let backpressure_active = wal::is_backpressure_active()?;
    
    Ok(PerformanceStats {
        wal_stats,
        backpressure_active,
        ring_capacity: ring::get_instance()?.get_config().ring_capacity(),
    })
}

/// 性能统计信息聚合
#[derive(Debug)]
pub struct PerformanceStats {
    pub wal_stats: wal::WalStats,
    pub backpressure_active: bool,
    pub ring_capacity: usize,
}

/// 发布CmdFrame到总线  
pub fn publish_cmd(frame: CmdFrame) -> Result<()> {
    let tx = ring::get_publisher()?;
    let publisher = ring::FramePublisher::new(tx.clone());
    publisher.send_cmd(frame)
}

/// 订阅指定过滤器的Frame
pub fn subscribe(filter: Filter) -> Result<FrameReceiver> {
    ring::subscribe(filter)
}

/// ACK指定序列号
pub fn ack(consumer_id: &str, seq: u64) -> Result<()> {
    wal::ack(consumer_id, seq)
}

/// 测试专用：创建独立的FrameBus实例，不使用全局状态
pub fn init_test_instance<P: AsRef<Path>>(ring_size: usize, wal_dir: P) -> Result<(FrameSender, FrameReceiver)> {
    let cfg = BusCfg {
        ring_pow: (ring_size as f64).log2() as u8,
        wal_dir: wal_dir.as_ref().to_path_buf(),
        ..Default::default()
    };
    
    // 对于测试，我们不初始化全局状态，而是返回独立的实例
    ring::force_reinit_for_test(cfg)
}

/// 性能测试辅助函数：创建测试数据帧
pub fn create_test_frame(tag: &str, value: f64) -> DataFrame {
    DataFrame::new(tag, Value::float(value))
}

/// 性能测试辅助函数：创建批量测试数据帧
pub fn create_test_frames(tag_prefix: &str, count: usize, base_value: f64) -> Vec<DataFrame> {
    (0..count)
        .map(|i| {
            let tag = format!("{}.{:04}", tag_prefix, i);
            let value = base_value + i as f64;
            DataFrame::new(tag, Value::float(value))
        })
        .collect()
}