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
pub use ring::{FrameSender, FrameReceiver, FramePublisher};
pub use filter::Filter;
pub use config::BusCfg;
pub use wal::WAL;
pub use command::{CommandProcessor, CommandPriority, CommandStatus, CommandResult, PendingCommand};
pub use permissions::{PermissionManager, Permission, AccessControlEntry};

use anyhow::Result;
use std::path::Path;

/// 初始化FrameBus全局实例
pub fn init<P: AsRef<Path>>(ring_size: usize, wal_dir: P) -> Result<(FrameSender, FrameReceiver)> {
    let cfg = BusCfg {
        ring_pow: (ring_size as f64).log2() as u8,
        wal_dir: wal_dir.as_ref().to_path_buf(),
        ..Default::default()
    };
    
    metrics::init_metrics();
    wal::init(&cfg.wal_dir)?;
    ring::init(cfg)
}

/// 发布DataFrame到总线
pub fn publish_data(frame: DataFrame) -> Result<()> {
    let tx = ring::get_publisher()?;
    let publisher = ring::FramePublisher::new(tx.clone());
    publisher.send_data(frame)
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