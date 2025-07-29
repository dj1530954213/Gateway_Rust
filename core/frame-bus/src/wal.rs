//! WAL (Write-Ahead Log) 实现
//! MVP-3增强：后台GC线程、自动压缩、批量写入优化

use rocksdb::{DB, Options, ColumnFamilyDescriptor};
use std::path::Path;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};

use crate::metrics::METRICS;

/// 全局WAL实例
static GLOBAL_WAL: OnceCell<Arc<WalManager>> = OnceCell::new();

/// 批量写入条目
#[derive(Debug)]
struct BatchEntry {
    seq: u64,
    payload: Vec<u8>,
}

/// WAL配置
#[derive(Debug, Clone)]
pub struct WalConfig {
    /// 批量写入超时 (100ms for MVP-3)
    pub batch_timeout: Duration,
    /// 批量大小限制
    pub batch_size_limit: usize,
    /// GC 检查间隔
    pub gc_interval: Duration,
    /// 自动压缩阈值 (4GiB for MVP-3)
    pub auto_compact_threshold: u64,
    /// 保留的帧数量
    pub retain_frames: u64,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            batch_timeout: Duration::from_millis(100), // MVP-3要求：100ms批处理
            batch_size_limit: 1000,
            gc_interval: Duration::from_secs(30),
            auto_compact_threshold: 4 * 1024 * 1024 * 1024, // 4GiB
            retain_frames: 10000,
        }
    }
}

/// WAL管理器
pub struct WalManager {
    db: Arc<DB>,
    config: WalConfig,
    // 批量写入通道
    batch_sender: mpsc::UnboundedSender<BatchEntry>,
    // 后台任务状态
    background_running: Arc<AtomicBool>,
    // 统计信息
    total_writes: AtomicU64,
    batch_writes: AtomicU64,
    gc_runs: AtomicU64,
    compactions: AtomicU64,
}

/// WAL 对外接口
pub struct WAL {
    manager: Arc<WalManager>,
}

impl WAL {
    /// 创建新的 WAL 实例
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let manager = Arc::new(WalManager::open(path, WalConfig::default())?);
        Ok(Self { manager })
    }

    /// 使用自定义配置创建 WAL 实例
    pub async fn with_config<P: AsRef<Path>>(path: P, config: WalConfig) -> Result<Self> {
        let manager = Arc::new(WalManager::open(path, config)?);
        Ok(Self { manager })
    }
    
    /// 写入帧
    pub async fn write_frame(&self, seq: u64, payload: &[u8]) -> Result<()> {
        self.manager.write_frame(seq, payload)
    }
    
    /// 读取帧
    pub async fn read_frame(&self, seq: u64) -> Result<Option<Vec<u8>>> {
        self.manager.read_frame(seq)
    }
    
    /// 恢复所有帧
    pub async fn recover_all(&self) -> Result<Vec<crate::FrameEnvelope>> {
        self.manager.recover_all()
    }
    
    /// 恢复所有帧（别名方法）
    pub async fn recover(&self) -> Result<Vec<crate::FrameEnvelope>> {
        self.recover_all().await
    }
    
    /// 添加单个帧
    pub async fn append(&self, envelope: &crate::FrameEnvelope) -> Result<()> {
        use prost::Message;
        let payload = envelope.encode_to_vec();
        self.write_frame(envelope.seq, &payload).await
    }
    
    /// 批量添加帧
    pub async fn append_batch(&self, envelopes: &[crate::FrameEnvelope]) -> Result<()> {
        for envelope in envelopes {
            self.append(envelope).await?;
        }
        Ok(())
    }
    
    /// 获取最小序列号（从frames中）
    pub async fn get_min_sequence(&self) -> Result<Option<u64>> {
        self.manager.min_frame_seq()
    }
    
    /// 同步到磁盘
    pub async fn sync(&self) -> Result<()> {
        self.manager.flush()
    }
    
    /// 垃圾回收
    pub async fn gc(&self, keep_seq: u64) -> Result<()> {
        self.manager.gc(keep_seq)
    }
}

impl WalManager {
    /// 打开WAL数据库
    fn open<P: AsRef<Path>>(path: P, config: WalConfig) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        // 优化RocksDB设置以支持高性能写入
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB write buffer
        opts.set_max_write_buffer_number(3);
        opts.set_target_file_size_base(64 * 1024 * 1024);
        opts.set_level_zero_file_num_compaction_trigger(4);
        opts.set_level_zero_slowdown_writes_trigger(20);
        opts.set_level_zero_stop_writes_trigger(36);

        // 定义列族
        let frames_cf = ColumnFamilyDescriptor::new("frames", Options::default());
        let acks_cf = ColumnFamilyDescriptor::new("acks", Options::default());

        let db = DB::open_cf_descriptors(&opts, path, vec![frames_cf, acks_cf])?;
        
        // 创建批量写入通道
        let (batch_sender, batch_receiver) = mpsc::unbounded_channel();
        
        let manager = Self {
            db: Arc::new(db),
            config,
            batch_sender,
            background_running: Arc::new(AtomicBool::new(true)),
            total_writes: AtomicU64::new(0),
            batch_writes: AtomicU64::new(0),
            gc_runs: AtomicU64::new(0),
            compactions: AtomicU64::new(0),
        };
        
        // 启动后台任务
        manager.start_background_tasks(batch_receiver)?;
        
        Ok(manager)
    }

    /// 启动后台任务：批量写入、GC、自动压缩
    fn start_background_tasks(&self, mut batch_receiver: mpsc::UnboundedReceiver<BatchEntry>) -> Result<()> {
        let db = self.db.clone();
        let config = self.config.clone();
        let background_running = self.background_running.clone();
        let batch_writes = Arc::new(AtomicU64::new(0));
        let gc_runs = Arc::new(AtomicU64::new(0));
        let compactions = Arc::new(AtomicU64::new(0));
        
        // 批量写入任务
        tokio::spawn(async move {
            Self::batch_write_task(db.clone(), config.clone(), batch_receiver, batch_writes).await;
        });
        
        // GC和压缩任务
        let db_clone = self.db.clone();
        let config_clone = self.config.clone();
        let background_running_clone = self.background_running.clone();
        tokio::spawn(async move {
            Self::gc_and_compact_task(db_clone, config_clone, background_running_clone, gc_runs, compactions).await;
        });
        
        info!("WAL background tasks started");
        Ok(())
    }

    /// 批量写入任务
    async fn batch_write_task(
        db: Arc<DB>,
        config: WalConfig,
        mut receiver: mpsc::UnboundedReceiver<BatchEntry>,
        batch_writes: Arc<AtomicU64>,
    ) {
        let mut batch = Vec::new();
        let mut last_flush = Instant::now();
        
        loop {
            let timeout = tokio::time::sleep(config.batch_timeout);
            tokio::pin!(timeout);
            
            tokio::select! {
                entry = receiver.recv() => {
                    match entry {
                        Some(entry) => {
                            batch.push(entry);
                            
                            // 检查是否需要立即刷新
                            if batch.len() >= config.batch_size_limit {
                                Self::flush_batch(&db, &mut batch, &batch_writes);
                                last_flush = Instant::now();
                            }
                        }
                        None => {
                            // 通道关闭，刷新剩余批次并退出
                            if !batch.is_empty() {
                                Self::flush_batch(&db, &mut batch, &batch_writes);
                            }
                            break;
                        }
                    }
                }
                _ = timeout => {
                    // 超时，刷新当前批次
                    if !batch.is_empty() && last_flush.elapsed() >= config.batch_timeout {
                        Self::flush_batch(&db, &mut batch, &batch_writes);
                        last_flush = Instant::now();
                    }
                }
            }
        }
        
        debug!("Batch write task terminated");
    }

    /// 刷新批次到数据库
    fn flush_batch(db: &Arc<DB>, batch: &mut Vec<BatchEntry>, batch_writes: &Arc<AtomicU64>) {
        if batch.is_empty() {
            return;
        }
        
        let start = Instant::now();
        let frames_cf = match db.cf_handle("frames") {
            Some(cf) => cf,
            None => {
                warn!("frames CF not found, dropping batch");
                batch.clear();
                return;
            }
        };
        
        // 使用RocksDB批量写入
        let mut write_batch = rocksdb::WriteBatch::default();
        for entry in batch.iter() {
            let key = entry.seq.to_be_bytes();
            write_batch.put_cf(frames_cf, key, &entry.payload);
        }
        
        if let Err(e) = db.write(write_batch) {
            warn!("Failed to write batch: {}", e);
        } else {
            batch_writes.fetch_add(1, Ordering::Relaxed);
            debug!("Flushed batch of {} entries in {:?}", batch.len(), start.elapsed());
            
            // 更新指标 - 5k fps flush <5ms 性能要求
            let flush_duration = start.elapsed();
            METRICS.wal_flush_duration.observe(flush_duration.as_millis() as f64);
            if flush_duration.as_millis() > 5 {
                warn!("Batch flush exceeded 5ms requirement: {:?}", flush_duration);
            }
        }
        
        batch.clear();
    }

    /// GC和自动压缩任务
    async fn gc_and_compact_task(
        db: Arc<DB>,
        config: WalConfig,
        background_running: Arc<AtomicBool>,
        gc_runs: Arc<AtomicU64>,
        compactions: Arc<AtomicU64>,
    ) {
        let mut interval = tokio::time::interval(config.gc_interval);
        
        while background_running.load(Ordering::Relaxed) {
            interval.tick().await;
            
            // 执行GC
            if let Err(e) = Self::perform_gc(&db, &config) {
                warn!("GC failed: {}", e);
            } else {
                gc_runs.fetch_add(1, Ordering::Relaxed);
            }
            
            // 检查是否需要自动压缩
            let current_size = db.property_int_value("rocksdb.estimate-live-data-size")
                .unwrap_or(Some(0))
                .unwrap_or(0);
                
            if current_size > config.auto_compact_threshold {
                info!("WAL size ({} bytes) exceeds threshold, triggering compaction", current_size);
                db.compact_range::<&[u8], &[u8]>(None, None);
                compactions.fetch_add(1, Ordering::Relaxed);
                info!("Auto compaction completed");
            }
        }
        
        debug!("GC and compact task terminated");
    }

    /// 执行垃圾回收
    fn perform_gc(db: &Arc<DB>, config: &WalConfig) -> Result<()> {
        let acks_cf = db.cf_handle("acks")
            .ok_or_else(|| anyhow::anyhow!("acks CF not found"))?;
        let frames_cf = db.cf_handle("frames")
            .ok_or_else(|| anyhow::anyhow!("frames CF not found"))?;
        
        // 找到所有消费者的最小ACK序列号
        let iter = db.iterator_cf(acks_cf, rocksdb::IteratorMode::Start);
        let mut min_ack_seq = None;
        
        for item in iter {
            let (_, value) = item?;
            if value.len() >= 8 {
                let seq = u64::from_le_bytes(value[0..8].try_into().unwrap());
                min_ack_seq = Some(min_ack_seq.map_or(seq, |min: u64| min.min(seq)));
            }
        }
        
        if let Some(min_seq) = min_ack_seq {
            // 保留指定数量的帧
            if min_seq > config.retain_frames {
                let gc_seq = min_seq - config.retain_frames;
                let start_key = 0u64.to_be_bytes();
                let end_key = gc_seq.to_be_bytes();
                
                db.delete_range_cf(frames_cf, start_key, end_key)?;
                debug!("WAL GC: deleted frames with seq < {}", gc_seq);
            }
        }
        
        Ok(())
    }

    /// 写入帧到WAL (MVP-3: 使用批量写入)
    pub fn write_frame(&self, seq: u64, payload: &[u8]) -> Result<()> {
        // 使用批量写入通道
        let entry = BatchEntry {
            seq,
            payload: payload.to_vec(),
        };
        
        self.batch_sender.send(entry)
            .map_err(|_| anyhow::anyhow!("Failed to send to batch writer, channel closed"))?;
        
        self.total_writes.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }

    /// 直接写入帧到WAL (绕过批量系统，用于紧急情况)
    pub fn write_frame_direct(&self, seq: u64, payload: &[u8]) -> Result<()> {
        let frames_cf = self.db.cf_handle("frames")
            .ok_or_else(|| anyhow::anyhow!("frames CF not found"))?;
        
        let key = seq.to_be_bytes();
        self.db.put_cf(frames_cf, key, payload)?;
        
        // 更新指标
        let size = self.estimate_size();
        METRICS.wal_bytes.set(size as i64);
        
        Ok(())
    }

    /// 读取帧
    pub fn read_frame(&self, seq: u64) -> Result<Option<Vec<u8>>> {
        let frames_cf = self.db.cf_handle("frames")
            .ok_or_else(|| anyhow::anyhow!("frames CF not found"))?;
        
        let key = seq.to_be_bytes();
        match self.db.get_cf(frames_cf, key)? {
            Some(data) => Ok(Some(data)),
            None => Ok(None),
        }
    }

    /// 恢复所有帧
    pub fn recover_all(&self) -> Result<Vec<crate::FrameEnvelope>> {
        use prost::Message;
        
        let frames_cf = self.db.cf_handle("frames")
            .ok_or_else(|| anyhow::anyhow!("frames CF not found"))?;
        
        let mut frames = Vec::new();
        let iter = self.db.iterator_cf(frames_cf, rocksdb::IteratorMode::Start);
        
        for item in iter {
            let (_, value) = item?;
            if let Ok(envelope) = crate::FrameEnvelope::decode(&value[..]) {
                frames.push(envelope);
            }
        }
        
        // 按序列号排序
        frames.sort_by_key(|f| f.seq);
        
        Ok(frames)
    }

    /// 记录消费者ACK
    pub fn ack(&self, consumer_id: &str, seq: u64) -> Result<()> {
        let acks_cf = self.db.cf_handle("acks")
            .ok_or_else(|| anyhow::anyhow!("acks CF not found"))?;
        
        let value = seq.to_le_bytes();
        self.db.put_cf(acks_cf, consumer_id.as_bytes(), value)?;
        
        Ok(())
    }

    /// 获取最小frames序列号
    pub fn min_frame_seq(&self) -> Result<Option<u64>> {
        let frames_cf = self.db.cf_handle("frames")
            .ok_or_else(|| anyhow::anyhow!("frames CF not found"))?;
        
        let iter = self.db.iterator_cf(frames_cf, rocksdb::IteratorMode::Start);
        
        if let Some(item) = iter.into_iter().next() {
            let (key, _) = item?;
            if key.len() >= 8 {
                let seq = u64::from_be_bytes(key[0..8].try_into().unwrap());
                return Ok(Some(seq));
            }
        }
        
        Ok(None)
    }

    /// 获取最小ACK序列号（用于GC）
    pub fn min_ack_seq(&self) -> Result<Option<u64>> {
        let acks_cf = self.db.cf_handle("acks")
            .ok_or_else(|| anyhow::anyhow!("acks CF not found"))?;
        
        let iter = self.db.iterator_cf(acks_cf, rocksdb::IteratorMode::Start);
        let mut min_seq = None;
        
        for item in iter {
            let (_, value) = item?;
            if value.len() >= 8 {
                let seq = u64::from_le_bytes(value[0..8].try_into().unwrap());
                min_seq = Some(min_seq.map_or(seq, |min: u64| min.min(seq)));
            }
        }
        
        Ok(min_seq)
    }

    /// 垃圾回收旧的帧
    pub fn gc(&self, keep_seq: u64) -> Result<()> {
        let frames_cf = self.db.cf_handle("frames")
            .ok_or_else(|| anyhow::anyhow!("frames CF not found"))?;
        
        // 删除序列号小于keep_seq的所有帧
        let start_key = 0u64.to_be_bytes();
        let end_key = keep_seq.to_be_bytes();
        
        self.db.delete_range_cf(frames_cf, start_key, end_key)?;
        
        tracing::info!("WAL GC: deleted frames with seq < {}", keep_seq);
        
        // 更新大小指标
        let size = self.estimate_size();
        METRICS.wal_bytes.set(size as i64);
        
        Ok(())
    }

    /// 估算数据库大小
    fn estimate_size(&self) -> u64 {
        // 简单实现：基于近似大小
        self.db.property_int_value("rocksdb.estimate-live-data-size")
            .unwrap_or(Some(0))
            .unwrap_or(0)
    }

    /// 强制刷新到磁盘
    pub fn flush(&self) -> Result<()> {
        let start = std::time::Instant::now();
        self.db.flush()?;
        let duration = start.elapsed().as_millis() as f64;
        METRICS.wal_flush_duration.observe(duration);
        Ok(())
    }

    /// 获取WAL统计信息
    pub fn get_stats(&self) -> WalStats {
        WalStats {
            total_writes: self.total_writes.load(Ordering::Relaxed),
            batch_writes: self.batch_writes.load(Ordering::Relaxed),
            gc_runs: self.gc_runs.load(Ordering::Relaxed),
            compactions: self.compactions.load(Ordering::Relaxed),
            current_size: self.estimate_size(),
            background_running: self.background_running.load(Ordering::Relaxed),
        }
    }

    /// 停止后台任务
    pub fn shutdown(&self) {
        self.background_running.store(false, Ordering::Relaxed);
        info!("WAL background tasks shutdown requested");
    }
}

/// WAL统计信息
#[derive(Debug)]
pub struct WalStats {
    pub total_writes: u64,
    pub batch_writes: u64,
    pub gc_runs: u64,
    pub compactions: u64,
    pub current_size: u64,
    pub background_running: bool,
}

impl Drop for WalManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// 初始化WAL
pub fn init<P: AsRef<Path>>(wal_dir: P) -> Result<()> {
    // 如果WAL已经初始化，直接返回成功
    if GLOBAL_WAL.get().is_some() {
        return Ok(());
    }
    
    let manager = WalManager::open(wal_dir, WalConfig::default())?;
    // 忽略重复初始化错误 - 支持幂等操作
    let _ = GLOBAL_WAL.set(Arc::new(manager));
    Ok(())
}

/// 使用自定义配置初始化WAL
pub fn init_with_config<P: AsRef<Path>>(wal_dir: P, config: WalConfig) -> Result<()> {
    // 如果WAL已经初始化，直接返回成功
    if GLOBAL_WAL.get().is_some() {
        return Ok(());
    }
    
    let manager = WalManager::open(wal_dir, config)?;
    // 忽略重复初始化错误 - 支持幂等操作
    let _ = GLOBAL_WAL.set(Arc::new(manager));
    Ok(())
}

/// 获取WAL实例
fn get_wal() -> Result<&'static Arc<WalManager>> {
    GLOBAL_WAL.get().ok_or_else(|| anyhow::anyhow!("WAL not initialized"))
}

/// 公共API函数

/// 写入帧
pub fn write_frame(seq: u64, payload: &[u8]) -> Result<()> {
    get_wal()?.write_frame(seq, payload)
}

/// ACK序列号
pub fn ack(consumer_id: &str, seq: u64) -> Result<()> {
    get_wal()?.ack(consumer_id, seq)
}

/// 执行垃圾回收
pub fn gc_if_needed() -> Result<()> {
    let wal = get_wal()?;
    
    if let Some(min_seq) = wal.min_ack_seq()? {
        // 保留最近1000个帧
        if min_seq > 1000 {
            wal.gc(min_seq - 1000)?;
        }
    }
    
    Ok(())
}

/// 强制刷新
pub fn flush() -> Result<()> {
    get_wal()?.flush()
}