//! FrameBus配置

use std::path::PathBuf;

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
}

impl Default for BusCfg {
    fn default() -> Self {
        Self {
            ring_pow: 18,  // 256k
            pause_hi: 0.80,
            resume_lo: 0.60,
            wal_dir: PathBuf::from("./wal"),
            wal_flush_ms: 100,
            wal_max_bytes: 4 * 1024 * 1024 * 1024, // 4GB
        }
    }
}