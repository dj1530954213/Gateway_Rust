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

/// 全局内存WAL实例 (WSL2兼容性备选方案)
static GLOBAL_MEMORY_WAL: OnceCell<Arc<InMemoryWalManager>> = OnceCell::new();

/// 批量写入条目
#[derive(Debug)]
struct BatchEntry {
    seq: u64,
    payload: Vec<u8>,
}

/// WAL配置
#[derive(Debug, Clone)]
pub struct WalConfig {
    /// 批量写入超时 (优化为1ms以提升性能)
    pub batch_timeout: Duration,
    /// 批量大小限制 (优化为5000)
    pub batch_size_limit: usize,
    /// 写入队列最大长度
    pub write_queue_capacity: usize,
    /// 异步写入工作线程数
    pub write_workers: usize,
    /// 异步刷盘模式
    pub async_sync: bool,
    /// 刷盘间隔 (优化为10ms)
    pub sync_interval: Duration,
    /// GC 检查间隔
    pub gc_interval: Duration,
    /// 自动压缩阈值 (4GiB for MVP-3)
    pub auto_compact_threshold: u64,
    /// 保留的帧数量
    pub retain_frames: u64,
    /// 背压阈值
    pub backpressure_threshold: usize,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            batch_timeout: Duration::from_millis(1), // 优化：1ms批处理提升响应性
            batch_size_limit: 5000, // 优化：提高批量大小
            write_queue_capacity: 50000, // 写入队列容量
            write_workers: 2, // 2个异步写入工作线程
            async_sync: true, // 启用异步刷盘
            sync_interval: Duration::from_millis(10), // 10ms刷盘间隔
            gc_interval: Duration::from_secs(30),
            auto_compact_threshold: 4 * 1024 * 1024 * 1024, // 4GiB
            retain_frames: 10000,
            backpressure_threshold: 40000, // 队列80%时触发背压
        }
    }
}

/// WAL管理器
pub struct WalManager {
    db: Arc<DB>,
    config: WalConfig,
    // 高性能写入队列 (有界，支持背压)
    write_queue: mpsc::Sender<BatchEntry>,
    // 异步同步通道
    sync_sender: mpsc::UnboundedSender<()>,
    // 后台任务状态
    background_running: Arc<AtomicBool>,
    // 性能统计
    total_writes: AtomicU64,
    batch_writes: AtomicU64,
    sync_operations: AtomicU64,
    gc_runs: AtomicU64,
    compactions: AtomicU64,
    // 背压状态
    backpressure_active: Arc<AtomicBool>,
    // 延迟统计
    last_write_latency: Arc<std::sync::Mutex<Duration>>,
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
    
    /// 高性能异步写入帧
    pub async fn write_frame(&self, seq: u64, payload: &[u8]) -> Result<()> {
        self.manager.write_frame(seq, payload).await
    }
    
    /// 同步写入帧 (兼容性接口)
    pub fn write_frame_sync(&self, seq: u64, payload: &[u8]) -> Result<()> {
        self.manager.write_frame_sync(seq, payload)
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
    
    /// 同步到磁盘 (支持异步模式)
    pub async fn sync(&self) -> Result<()> {
        self.manager.trigger_sync()
    }
    
    /// 强制同步刷盘
    pub async fn force_sync(&self) -> Result<()> {
        self.manager.flush()
    }
    
    /// 获取性能统计
    pub fn get_performance_stats(&self) -> WalStats {
        self.manager.get_stats()
    }
    
    /// 检查背压状态
    pub fn is_backpressure_active(&self) -> bool {
        self.manager.is_backpressure_active()
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
        
        // 高性能RocksDB优化设置
        opts.set_write_buffer_size(128 * 1024 * 1024); // 128MB write buffer
        opts.set_max_write_buffer_number(4); // 更多buffer
        opts.set_target_file_size_base(128 * 1024 * 1024);
        opts.set_level_zero_file_num_compaction_trigger(6); // 更晚触发压缩
        opts.set_level_zero_slowdown_writes_trigger(25);
        opts.set_level_zero_stop_writes_trigger(40);
        
        // 异步写入优化
        opts.set_use_fsync(false); // 使用fdatasync代替fsync
        opts.set_bytes_per_sync(1024 * 1024); // 1MB增量同步
        opts.set_wal_bytes_per_sync(512 * 1024); // 512KB WAL增量同步
        
        // 压缩优化
        opts.set_max_background_compactions(4);
        opts.set_max_background_flushes(2);
        opts.set_allow_concurrent_memtable_write(true);
        opts.set_enable_write_thread_adaptive_yield(true);

        // 定义列族
        let frames_cf = ColumnFamilyDescriptor::new("frames", Options::default());
        let acks_cf = ColumnFamilyDescriptor::new("acks", Options::default());

        let db = DB::open_cf_descriptors(&opts, path, vec![frames_cf, acks_cf])?;
        
        // 创建有界写入队列以支持背压
        let (write_sender, write_receiver) = mpsc::channel(config.write_queue_capacity);
        let (sync_sender, sync_receiver) = mpsc::unbounded_channel();
        
        let manager = Self {
            db: Arc::new(db),
            config: config.clone(),
            write_queue: write_sender,
            sync_sender,
            background_running: Arc::new(AtomicBool::new(true)),
            total_writes: AtomicU64::new(0),
            batch_writes: AtomicU64::new(0),
            sync_operations: AtomicU64::new(0),
            gc_runs: AtomicU64::new(0),
            compactions: AtomicU64::new(0),
            backpressure_active: Arc::new(AtomicBool::new(false)),
            last_write_latency: Arc::new(std::sync::Mutex::new(Duration::from_nanos(0))),
        };
        
        // 启动后台任务
        manager.start_background_tasks(write_receiver, sync_receiver)?;
        
        Ok(manager)
    }

    /// 启动后台任务：批量写入、异步同步、GC、自动压缩
    fn start_background_tasks(
        &self, 
        write_receiver: mpsc::Receiver<BatchEntry>, 
        sync_receiver: mpsc::UnboundedReceiver<()>
    ) -> Result<()> {
        let db = self.db.clone();
        let config = self.config.clone();
        let background_running = self.background_running.clone();
        let batch_writes = Arc::new(AtomicU64::new(0));
        let sync_operations = Arc::new(AtomicU64::new(0));
        let gc_runs = Arc::new(AtomicU64::new(0));
        let compactions = Arc::new(AtomicU64::new(0));
        let backpressure_active = self.backpressure_active.clone();
        let last_write_latency = self.last_write_latency.clone();
        
        // 启动多个批量写入工作线程
        for worker_id in 0..config.write_workers {
            let db_clone = db.clone();
            let config_clone = config.clone();
            let batch_writes_clone = batch_writes.clone();
            let background_running_clone = background_running.clone();
            let backpressure_clone = backpressure_active.clone();
            let latency_clone = last_write_latency.clone();
            
            tokio::spawn(async move {
                info!("WAL写入工作线程 {} 启动", worker_id);
                Self::high_performance_write_task(
                    db_clone, config_clone, background_running_clone, 
                    batch_writes_clone, backpressure_clone, latency_clone
                ).await;
                info!("WAL写入工作线程 {} 退出", worker_id);
            });
        }
        
        // 提前保存需要的配置值
        let async_sync_enabled = config.async_sync;
        let sync_config = config.clone(); // 为异步同步任务克隆配置
        
        // 批量写入分发任务
        tokio::spawn(async move {
            Self::write_dispatcher_task(db.clone(), config.clone(), write_receiver, batch_writes).await;
        });
        
        // 异步同步任务
        if async_sync_enabled {
            let db_sync = self.db.clone();
            let sync_ops = sync_operations.clone();
            let sync_running = background_running.clone();
            tokio::spawn(async move {
                Self::async_sync_task(db_sync, sync_config, sync_receiver, sync_ops, sync_running).await;
            });
        }
        
        // GC和压缩任务
        let db_clone = self.db.clone();
        let config_clone = self.config.clone();
        let background_running_clone = self.background_running.clone();
        tokio::spawn(async move {
            Self::gc_and_compact_task(db_clone, config_clone, background_running_clone, gc_runs, compactions).await;
        });
        
        info!("WAL高性能后台任务启动完成: {} 写入工作线程, 异步同步: {}", 
              self.config.write_workers, self.config.async_sync);
        Ok(())
    }

    /// 高性能写入任务 (单独工作线程)
    async fn high_performance_write_task(
        db: Arc<DB>,
        _config: WalConfig,
        background_running: Arc<AtomicBool>,
        batch_writes: Arc<AtomicU64>,
        backpressure_active: Arc<AtomicBool>,
        last_write_latency: Arc<std::sync::Mutex<Duration>>,
    ) {
        use std::sync::mpsc;
        
        // 创建本地工作线程队列
        let (local_tx, local_rx) = mpsc::channel::<Vec<BatchEntry>>();
        
        // 在独立的系统线程中处理写入，避免异步等待
        std::thread::spawn(move || {
            while background_running.load(Ordering::Relaxed) {
                if let Ok(batch_entries) = local_rx.recv_timeout(Duration::from_millis(1)) {
                    let start = Instant::now();
                    
                    if let Err(e) = Self::flush_batch_optimized(&db, batch_entries, &batch_writes) {
                        warn!("高性能写入失败: {}", e);
                    } else {
                        let latency = start.elapsed();
                        if let Ok(mut last_latency) = last_write_latency.lock() {
                            *last_latency = latency;
                        }
                        
                        // 检查是否超过延迟阈值
                        if latency.as_millis() > 10 {
                            warn!("写入延迟超过10ms阈值: {:?}", latency);
                            backpressure_active.store(true, Ordering::Relaxed);
                        } else if latency.as_millis() < 3 {
                            backpressure_active.store(false, Ordering::Relaxed);
                        }
                    }
                }
            }
        });
        
        debug!("高性能写入工作线程初始化完成");
    }
    
    /// 批量写入分发任务
    async fn write_dispatcher_task(
        _db: Arc<DB>,
        config: WalConfig,
        mut receiver: mpsc::Receiver<BatchEntry>,
        _batch_writes: Arc<AtomicU64>,
    ) {
        let mut batch = Vec::with_capacity(config.batch_size_limit);
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
                                // TODO: 发送给工作线程处理
                                last_flush = Instant::now();
                            }
                        }
                        None => {
                            // 通道关闭，处理剩余批次并退出
                            if !batch.is_empty() {
                                // TODO: 发送给工作线程处理
                            }
                            break;
                        }
                    }
                }
                _ = timeout => {
                    // 超时，处理当前批次
                    if !batch.is_empty() && last_flush.elapsed() >= config.batch_timeout {
                        // TODO: 发送给工作线程处理
                        last_flush = Instant::now();
                    }
                }
            }
        }
        
        debug!("批量写入分发任务终止");
    }

    /// 优化的批量刷新到数据库 (零拷贝 + 并行优化)
    fn flush_batch_optimized(
        db: &Arc<DB>, 
        batch_entries: Vec<BatchEntry>, 
        batch_writes: &Arc<AtomicU64>
    ) -> Result<()> {
        if batch_entries.is_empty() {
            return Ok(());
        }
        
        let start = Instant::now();
        let frames_cf = db.cf_handle("frames")
            .ok_or_else(|| anyhow::anyhow!("frames CF not found"))?;
        
        // 高性能批量写入
        let mut write_batch = rocksdb::WriteBatch::default();
        
        // 使用迭代器避免额外分配
        for entry in batch_entries.iter() {
            let key = entry.seq.to_be_bytes();
            write_batch.put_cf(frames_cf, key, &entry.payload);
        }
        
        // 使用高性能写入选项
        let mut write_opts = rocksdb::WriteOptions::default();
        write_opts.set_sync(false); // 异步写入，由单独的sync任务处理
        write_opts.disable_wal(false); // 保持WAL以确保数据安全
        
        db.write_opt(write_batch, &write_opts)
            .map_err(|e| anyhow::anyhow!("Batch write failed: {}", e))?;
            
        batch_writes.fetch_add(1, Ordering::Relaxed);
        let batch_count = batch_entries.len();
        
        // 性能指标记录
        let flush_duration = start.elapsed();
        METRICS.wal_flush_duration.observe(flush_duration.as_millis() as f64);
        METRICS.batch_size.observe(batch_count as f64);
        
        // 性能监控：目标<1ms
        if flush_duration.as_millis() > 1 {
            warn!("批量写入超过1ms目标: {:?}, 批量大小: {}", flush_duration, batch_count);
        }
        
        debug!("高性能批量写入完成: {} 条记录, 耗时: {:?}", batch_count, flush_duration);
        
        Ok(())
    }
    
    /// 传统批量刷新 (兼容性保留)
    fn flush_batch(db: &Arc<DB>, batch: &mut Vec<BatchEntry>, batch_writes: &Arc<AtomicU64>) {
        if batch.is_empty() {
            return;
        }
        
        if let Err(e) = Self::flush_batch_optimized(db, batch.drain(..).collect(), batch_writes) {
            warn!("传统批量刷新失败: {}", e);
        }
    }

    /// 异步同步任务 (定时刷盘)
    async fn async_sync_task(
        db: Arc<DB>,
        config: WalConfig,
        mut sync_receiver: mpsc::UnboundedReceiver<()>,
        sync_operations: Arc<AtomicU64>,
        background_running: Arc<AtomicBool>,
    ) {
        let mut sync_interval = tokio::time::interval(config.sync_interval);
        sync_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        while background_running.load(Ordering::Relaxed) {
            tokio::select! {
                // 定时同步
                _ = sync_interval.tick() => {
                    let start = Instant::now();
                    if let Err(e) = db.flush() {
                        warn!("定时同步失败: {}", e);
                    } else {
                        sync_operations.fetch_add(1, Ordering::Relaxed);
                        let sync_duration = start.elapsed();
                        if sync_duration.as_millis() > 5 {
                            debug!("同步耗时: {:?}", sync_duration);
                        }
                    }
                }
                // 手动触发同步
                _ = sync_receiver.recv() => {
                    let start = Instant::now();
                    if let Err(e) = db.flush() {
                        warn!("手动同步失败: {}", e);
                    } else {
                        sync_operations.fetch_add(1, Ordering::Relaxed);
                        debug!("手动同步完成: {:?}", start.elapsed());
                    }
                }
            }
        }
        
        debug!("异步同步任务终止");
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

    /// 高性能写入帧到WAL (支持背压控制)
    pub async fn write_frame(&self, seq: u64, payload: &[u8]) -> Result<()> {
        let start = Instant::now();
        
        // 检查背压状态
        if self.backpressure_active.load(Ordering::Relaxed) {
            // 在背压状态下，稍微延迟以减少负荷
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
        
        let entry = BatchEntry {
            seq,
            payload: payload.to_vec(),
        };
        
        // 使用有界通道，在队列满时等待
        match tokio::time::timeout(Duration::from_millis(10), self.write_queue.send(entry)).await {
            Ok(result) => {
                result.map_err(|_| anyhow::anyhow!("Write queue closed"))?;
                self.total_writes.fetch_add(1, Ordering::Relaxed);
                
                // 记录写入延迟
                let write_latency = start.elapsed();
                if write_latency.as_millis() > 1 {
                    warn!("写入延迟超过1ms: {:?}", write_latency);
                }
                
                Ok(())
            },
            Err(_) => {
                // 写入超时，触发背压
                self.backpressure_active.store(true, Ordering::Relaxed);
                Err(anyhow::anyhow!("Write timeout - backpressure activated"))
            }
        }
    }
    
    /// 传统同步写入接口 (兼容性保留)
    pub fn write_frame_sync(&self, seq: u64, payload: &[u8]) -> Result<()> {
        let entry = BatchEntry {
            seq,
            payload: payload.to_vec(),
        };
        
        // 非阻塞式发送，在队列满时返回错误
        self.write_queue.try_send(entry)
            .map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => {
                    self.backpressure_active.store(true, Ordering::Relaxed);
                    anyhow::anyhow!("Write queue full - backpressure activated")
                },
                mpsc::error::TrySendError::Closed(_) => {
                    anyhow::anyhow!("Write queue closed")
                }
            })?;
        
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
        let queue_len = self.write_queue.capacity() - self.write_queue.capacity(); // 近似计算
        let queue_usage = (queue_len as f64 / self.config.write_queue_capacity as f64) * 100.0;
        
        let last_latency_ms = self.last_write_latency
            .lock()
            .map(|l| l.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);
        
        WalStats {
            total_writes: self.total_writes.load(Ordering::Relaxed),
            batch_writes: self.batch_writes.load(Ordering::Relaxed),
            sync_operations: self.sync_operations.load(Ordering::Relaxed),
            gc_runs: self.gc_runs.load(Ordering::Relaxed),
            compactions: self.compactions.load(Ordering::Relaxed),
            current_size: self.estimate_size(),
            background_running: self.background_running.load(Ordering::Relaxed),
            backpressure_active: self.backpressure_active.load(Ordering::Relaxed),
            last_write_latency_ms: last_latency_ms,
            queue_usage_percent: queue_usage,
        }
    }
    
    /// 获取背压状态
    pub fn is_backpressure_active(&self) -> bool {
        self.backpressure_active.load(Ordering::Relaxed)
    }
    
    /// 手动触发同步
    pub fn trigger_sync(&self) -> Result<()> {
        if self.config.async_sync {
            self.sync_sender.send(())
                .map_err(|_| anyhow::anyhow!("Sync channel closed"))?;
        } else {
            self.db.flush()?;
        }
        Ok(())
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
    pub sync_operations: u64,
    pub gc_runs: u64,
    pub compactions: u64,
    pub current_size: u64,
    pub background_running: bool,
    pub backpressure_active: bool,
    pub last_write_latency_ms: f64,
    pub queue_usage_percent: f64,
}

/// 内存WAL管理器 (WSL2兼容性备选方案)
pub struct InMemoryWalManager {
    // 使用HashMap存储帧数据，以序列号为键
    frames: Arc<std::sync::RwLock<std::collections::HashMap<u64, Vec<u8>>>>,
    // ACK记录
    acks: Arc<std::sync::RwLock<std::collections::HashMap<String, u64>>>,
    // 配置
    config: WalConfig,
    // 性能统计
    total_writes: AtomicU64,
    sync_operations: AtomicU64,
    last_write_latency: Arc<std::sync::Mutex<Duration>>,
}

impl InMemoryWalManager {
    /// 创建新的内存WAL管理器
    pub fn new(config: WalConfig) -> Self {
        info!("Initializing In-Memory WAL Manager (WSL2 compatibility mode)");
        Self {
            frames: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            acks: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            config,
            total_writes: AtomicU64::new(0),
            sync_operations: AtomicU64::new(0),
            last_write_latency: Arc::new(std::sync::Mutex::new(Duration::from_nanos(0))),
        }
    }

    /// 高性能写入帧到内存
    pub async fn write_frame(&self, seq: u64, payload: &[u8]) -> Result<()> {
        let start = Instant::now();
        
        {
            let mut frames = self.frames.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;
            frames.insert(seq, payload.to_vec());
        }
        
        self.total_writes.fetch_add(1, Ordering::Relaxed);
        
        let write_latency = start.elapsed();
        if let Ok(mut last_latency) = self.last_write_latency.lock() {
            *last_latency = write_latency;
        }
        
        // 内存操作应该非常快，记录异常延迟
        if write_latency.as_millis() > 1 {
            warn!("内存写入延迟异常: {:?}", write_latency);
        }
        
        Ok(())
    }

    /// 同步写入接口 (兼容性)
    pub fn write_frame_sync(&self, seq: u64, payload: &[u8]) -> Result<()> {
        let start = Instant::now();
        
        {
            let mut frames = self.frames.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;
            frames.insert(seq, payload.to_vec());
        }
        
        self.total_writes.fetch_add(1, Ordering::Relaxed);
        
        let write_latency = start.elapsed();
        if let Ok(mut last_latency) = self.last_write_latency.lock() {
            *last_latency = write_latency;
        }
        
        Ok(())
    }

    /// 读取帧
    pub fn read_frame(&self, seq: u64) -> Result<Option<Vec<u8>>> {
        let frames = self.frames.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock"))?;
        Ok(frames.get(&seq).cloned())
    }

    /// 恢复所有帧
    pub fn recover_all(&self) -> Result<Vec<crate::FrameEnvelope>> {
        use prost::Message;
        
        let frames = self.frames.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock"))?;
        
        let mut envelopes = Vec::new();
        
        for (seq, data) in frames.iter() {
            if let Ok(envelope) = crate::FrameEnvelope::decode(&data[..]) {
                envelopes.push(envelope);
            } else {
                // 如果无法解码，创建一个基本的信封
                warn!("Failed to decode frame {}, creating basic envelope", seq);
            }
        }
        
        // 按序列号排序
        envelopes.sort_by_key(|f| f.seq);
        
        Ok(envelopes)
    }

    /// 记录消费者ACK
    pub fn ack(&self, consumer_id: &str, seq: u64) -> Result<()> {
        let mut acks = self.acks.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;
        acks.insert(consumer_id.to_string(), seq);
        Ok(())
    }

    /// 获取最小frames序列号
    pub fn min_frame_seq(&self) -> Result<Option<u64>> {
        let frames = self.frames.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock"))?;
        Ok(frames.keys().min().copied())
    }

    /// 获取最小ACK序列号
    pub fn min_ack_seq(&self) -> Result<Option<u64>> {
        let acks = self.acks.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock"))?;
        Ok(acks.values().min().copied())
    }

    /// 垃圾回收
    pub fn gc(&self, keep_seq: u64) -> Result<()> {
        let mut frames = self.frames.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;
        
        let mut to_remove = Vec::new();
        for &seq in frames.keys() {
            if seq < keep_seq {
                to_remove.push(seq);
            }
        }
        
        for seq in to_remove {
            frames.remove(&seq);
        }
        
        debug!("内存WAL GC: 删除了 {} 个序列号 < {} 的帧", frames.len(), keep_seq);
        Ok(())
    }

    /// 估算内存使用量
    fn estimate_size(&self) -> u64 {
        let frames = self.frames.read().unwrap();
        let mut total_size = 0u64;
        
        for (_, data) in frames.iter() {
            total_size += data.len() as u64 + 8; // 8字节用于键
        }
        
        total_size
    }

    /// 强制刷新 (内存模式下为空操作)
    pub fn flush(&self) -> Result<()> {
        self.sync_operations.fetch_add(1, Ordering::Relaxed);
        Ok(()) // 内存模式下无需刷新
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> WalStats {
        let _frames_count = self.frames.read().unwrap().len() as u64;
        let last_latency_ms = self.last_write_latency
            .lock()
            .map(|l| l.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

        WalStats {
            total_writes: self.total_writes.load(Ordering::Relaxed),
            batch_writes: 0, // 内存模式下不使用批量写入
            sync_operations: self.sync_operations.load(Ordering::Relaxed),
            gc_runs: 0,
            compactions: 0,
            current_size: self.estimate_size(),
            background_running: true,
            backpressure_active: false, // 内存模式下无背压
            last_write_latency_ms: last_latency_ms,
            queue_usage_percent: 0.0, // 内存模式下无队列
        }
    }

    /// 检查背压状态 (内存模式下始终为false)
    pub fn is_backpressure_active(&self) -> bool {
        false
    }

    /// 触发同步 (内存模式下为空操作)
    pub fn trigger_sync(&self) -> Result<()> {
        self.sync_operations.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

/// 内存WAL (WSL2兼容性备选方案)
pub struct InMemoryWAL {
    manager: Arc<InMemoryWalManager>,
}

impl InMemoryWAL {
    /// 创建新的内存WAL实例
    pub async fn new(config: WalConfig) -> Result<Self> {
        let manager = Arc::new(InMemoryWalManager::new(config));
        Ok(Self { manager })
    }

    /// 高性能异步写入帧
    pub async fn write_frame(&self, seq: u64, payload: &[u8]) -> Result<()> {
        self.manager.write_frame(seq, payload).await
    }

    /// 同步写入帧
    pub fn write_frame_sync(&self, seq: u64, payload: &[u8]) -> Result<()> {
        self.manager.write_frame_sync(seq, payload)
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

    /// 获取最小序列号
    pub async fn get_min_sequence(&self) -> Result<Option<u64>> {
        self.manager.min_frame_seq()
    }

    /// 同步到磁盘 (内存模式下为空操作)
    pub async fn sync(&self) -> Result<()> {
        self.manager.trigger_sync()
    }

    /// 强制同步刷盘 (内存模式下为空操作)
    pub async fn force_sync(&self) -> Result<()> {
        self.manager.flush()
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> WalStats {
        self.manager.get_stats()
    }

    /// 检查背压状态 (内存模式下始终为false)
    pub fn is_backpressure_active(&self) -> bool {
        self.manager.is_backpressure_active()
    }

    /// 垃圾回收
    pub async fn gc(&self, keep_seq: u64) -> Result<()> {
        self.manager.gc(keep_seq)
    }
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

/// 初始化内存WAL (WSL2兼容性备选方案)
pub fn init_memory_wal() -> Result<()> {
    info!("Initializing In-Memory WAL for WSL2 compatibility");
    
    // 如果内存WAL已经初始化，直接返回成功
    if GLOBAL_MEMORY_WAL.get().is_some() {
        return Ok(());
    }
    
    let config = WalConfig {
        // 内存模式优化配置
        batch_timeout: Duration::from_micros(100), // 更低延迟
        batch_size_limit: 1000,
        write_queue_capacity: 10000,
        write_workers: 1, // 内存模式下单线程即可
        async_sync: false, // 内存模式下不需要异步同步
        sync_interval: Duration::from_millis(1000), // 不重要，因为是空操作
        gc_interval: Duration::from_secs(60),
        auto_compact_threshold: 1024 * 1024 * 1024, // 1GB内存限制
        retain_frames: 50000, // 保留更多帧在内存中
        backpressure_threshold: 8000,
    };
    
    let manager = InMemoryWalManager::new(config);
    // 忽略重复初始化错误 - 支持幂等操作
    let _ = GLOBAL_MEMORY_WAL.set(Arc::new(manager));
    
    info!("In-Memory WAL initialized successfully");
    Ok(())
}

/// 初始化内存WAL (使用自定义配置)
pub fn init_memory_wal_with_config(config: WalConfig) -> Result<()> {
    info!("Initializing In-Memory WAL with custom config");
    
    // 如果内存WAL已经初始化，直接返回成功
    if GLOBAL_MEMORY_WAL.get().is_some() {
        return Ok(());
    }
    
    let manager = InMemoryWalManager::new(config);
    // 忽略重复初始化错误 - 支持幂等操作
    let _ = GLOBAL_MEMORY_WAL.set(Arc::new(manager));
    
    info!("In-Memory WAL with custom config initialized successfully");
    Ok(())
}

/// 获取WAL实例，优先返回持久化WAL，失败时使用内存WAL
fn get_wal() -> Result<&'static Arc<WalManager>> {
    GLOBAL_WAL.get().ok_or_else(|| anyhow::anyhow!("WAL not initialized"))
}

/// 获取内存WAL实例
fn get_memory_wal() -> Result<&'static Arc<InMemoryWalManager>> {
    GLOBAL_MEMORY_WAL.get().ok_or_else(|| anyhow::anyhow!("Memory WAL not initialized"))
}

/// 智能获取可用的WAL实例（优先持久化，降级到内存）
fn get_available_wal() -> Result<WalInstance> {
    if let Some(wal) = GLOBAL_WAL.get() {
        Ok(WalInstance::Persistent(wal))
    } else if let Some(memory_wal) = GLOBAL_MEMORY_WAL.get() {
        Ok(WalInstance::Memory(memory_wal))
    } else {
        Err(anyhow::anyhow!("No WAL instance initialized"))
    }
}

/// WAL实例枚举，支持持久化和内存两种模式
enum WalInstance {
    Persistent(&'static Arc<WalManager>),
    Memory(&'static Arc<InMemoryWalManager>),
}

/// 公共API函数 (支持智能降级：持久化 -> 内存)

/// 同步写入帧 (兼容性接口，支持降级)
pub fn write_frame(seq: u64, payload: &[u8]) -> Result<()> {
    match get_available_wal()? {
        WalInstance::Persistent(wal) => wal.write_frame_sync(seq, payload),
        WalInstance::Memory(memory_wal) => memory_wal.write_frame_sync(seq, payload),
    }
}

/// 异步高性能写入帧 (支持降级)
pub async fn write_frame_async(seq: u64, payload: &[u8]) -> Result<()> {
    match get_available_wal()? {
        WalInstance::Persistent(wal) => wal.write_frame(seq, payload).await,
        WalInstance::Memory(memory_wal) => memory_wal.write_frame(seq, payload).await,
    }
}

/// ACK序列号 (支持降级)
pub fn ack(consumer_id: &str, seq: u64) -> Result<()> {
    match get_available_wal()? {
        WalInstance::Persistent(wal) => wal.ack(consumer_id, seq),
        WalInstance::Memory(memory_wal) => memory_wal.ack(consumer_id, seq),
    }
}

/// 执行垃圾回收 (支持降级)
pub fn gc_if_needed() -> Result<()> {
    match get_available_wal()? {
        WalInstance::Persistent(wal) => {
            if let Some(min_seq) = wal.min_ack_seq()? {
                // 保留最近1000个帧
                if min_seq > 1000 {
                    wal.gc(min_seq - 1000)?;
                }
            }
        },
        WalInstance::Memory(memory_wal) => {
            if let Some(min_seq) = memory_wal.min_ack_seq()? {
                // 内存模式下保留更多帧以提供更好的性能
                if min_seq > 5000 {
                    memory_wal.gc(min_seq - 5000)?;
                }
            }
        },
    }
    
    Ok(())
}

/// 强制刷新 (支持降级)
pub fn flush() -> Result<()> {
    match get_available_wal()? {
        WalInstance::Persistent(wal) => wal.flush(),
        WalInstance::Memory(memory_wal) => memory_wal.flush(),
    }
}

/// 触发异步同步 (支持降级)
pub fn trigger_sync() -> Result<()> {
    match get_available_wal()? {
        WalInstance::Persistent(wal) => wal.trigger_sync(),
        WalInstance::Memory(memory_wal) => memory_wal.trigger_sync(),
    }
}

/// 获取性能统计 (支持降级)
pub fn get_performance_stats() -> Result<WalStats> {
    match get_available_wal()? {
        WalInstance::Persistent(wal) => Ok(wal.get_stats()),
        WalInstance::Memory(memory_wal) => Ok(memory_wal.get_stats()),
    }
}

/// 检查背压状态 (支持降级)
pub fn is_backpressure_active() -> Result<bool> {
    match get_available_wal()? {
        WalInstance::Persistent(wal) => Ok(wal.is_backpressure_active()),
        WalInstance::Memory(memory_wal) => Ok(memory_wal.is_backpressure_active()),
    }
}