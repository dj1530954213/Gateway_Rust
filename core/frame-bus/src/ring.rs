//! 环形缓冲区和广播

use tokio::sync::broadcast;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tracing::{info, warn};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::{FrameEnvelope, DataFrame, CmdFrame, Filter, BusCfg, metrics::METRICS};

/// 帧发送端
pub type FrameSender = broadcast::Sender<FrameEnvelope>;

/// 帧接收端包装器，支持过滤
pub struct FrameReceiver {
    inner: broadcast::Receiver<FrameEnvelope>,
    filter: Filter,
}

impl FrameReceiver {
    /// 创建新的FrameReceiver实例
    pub fn new(inner: broadcast::Receiver<FrameEnvelope>, filter: Filter) -> Self {
        Self { inner, filter }
    }

    pub async fn recv(&mut self) -> Result<FrameEnvelope, broadcast::error::RecvError> {
        loop {
            let envelope = self.inner.recv().await?;
            if self.filter.matches(&envelope) {
                return Ok(envelope);
            }
            // 如果不匹配过滤器，继续接收下一个
        }
    }
}

/// 全局序列号生成器 (线程安全)
static SEQ_GENERATOR: AtomicU64 = AtomicU64::new(0);

/// FrameBus实例管理器 (替代全局状态)
#[derive(Clone)]
pub struct FrameBusInstance {
    sender: FrameSender,
    config: BusCfg,
    batch_publisher: Option<Arc<AsyncBatchPublisher>>,
    sequence_offset: Arc<AtomicU64>, // 支持多实例序列号隔离
}

impl FrameBusInstance {
    pub fn new(sender: FrameSender, config: BusCfg) -> Self {
        Self {
            sender,
            config,
            batch_publisher: None,
            sequence_offset: Arc::new(AtomicU64::new(0)),
        }
    }
    
    pub fn with_batch_publisher(mut self, publisher: AsyncBatchPublisher) -> Self {
        self.batch_publisher = Some(Arc::new(publisher));
        self
    }
    
    pub fn get_sender(&self) -> &FrameSender {
        &self.sender
    }
    
    pub fn get_config(&self) -> &BusCfg {
        &self.config
    }
    
    pub fn get_batch_publisher(&self) -> Option<&Arc<AsyncBatchPublisher>> {
        self.batch_publisher.as_ref()
    }
    
    /// 生成实例本地序列号
    pub fn next_sequence(&self) -> u64 {
        self.sequence_offset.fetch_add(1, Ordering::SeqCst)
    }
}

/// 全局FrameBus实例 (使用Arc替代OnceCell)
static GLOBAL_INSTANCE: OnceCell<Arc<FrameBusInstance>> = OnceCell::new();

/// 测试专用：全局状态保护锁
static TEST_RESET_LOCK: Mutex<()> = Mutex::new(());

/// 高性能初始化FrameBus实例
pub fn init(cfg: BusCfg) -> Result<(FrameSender, FrameReceiver)> {
    // 如果已经初始化，返回现有实例的接口
    if let Some(existing_instance) = GLOBAL_INSTANCE.get() {
        let receiver = FrameReceiver {
            inner: existing_instance.sender.subscribe(),
            filter: Filter::All,
        };
        return Ok((existing_instance.sender.clone(), receiver));
    }
    
    let capacity = 1 << cfg.ring_pow;
    let (tx, rx) = broadcast::channel(capacity);
    
    let receiver = FrameReceiver {
        inner: rx,
        filter: Filter::All,
    };
    
    // 创建实例管理器
    let instance = Arc::new(FrameBusInstance::new(tx.clone(), cfg));
    
    // 忽略重复初始化错误 - 支持幂等操作
    let _ = GLOBAL_INSTANCE.set(instance);

    Ok((tx, receiver))
}

/// 获取全局发送端
pub fn get_publisher() -> Result<&'static FrameSender> {
    GLOBAL_INSTANCE.get()
        .map(|instance| instance.get_sender())
        .ok_or_else(|| anyhow::anyhow!("FrameBus not initialized"))
}

/// 获取全局FrameBus实例
pub fn get_instance() -> Result<&'static Arc<FrameBusInstance>> {
    GLOBAL_INSTANCE.get().ok_or_else(|| anyhow::anyhow!("FrameBus not initialized"))
}

/// 订阅指定过滤器 (优化版本)
pub fn subscribe(filter: Filter) -> Result<FrameReceiver> {
    let instance = get_instance()?;
    let rx = instance.sender.subscribe();
    Ok(FrameReceiver {
        inner: rx,
        filter,
    })
}

/// 订阅指定过滤器并返回实例信息
pub fn subscribe_with_instance(filter: Filter) -> Result<(FrameReceiver, Arc<FrameBusInstance>)> {
    let instance = get_instance()?.clone();
    let rx = instance.sender.subscribe();
    let receiver = FrameReceiver {
        inner: rx,
        filter,
    };
    Ok((receiver, instance))
}

/// 测试专用：强制重新初始化
pub fn force_reinit_for_test(cfg: BusCfg) -> Result<(FrameSender, FrameReceiver)> {
    use std::sync::atomic::Ordering;
    
    let _lock = TEST_RESET_LOCK.lock().unwrap();
    
    // 重置序列号生成器
    SEQ_GENERATOR.store(0, Ordering::SeqCst);
    
    let capacity = 1 << cfg.ring_pow;
    let (tx, rx) = broadcast::channel(capacity);
    
    let receiver = FrameReceiver {
        inner: rx,
        filter: Filter::All,
    };

    // 对于测试，我们创建一个新的通道并返回，不依赖全局状态
    Ok((tx, receiver))
}

/// 批量处理器配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_batch_size: usize,
    pub flush_interval: Duration,
    pub max_memory_bytes: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 2000, // 优化: 提高批量大小
            flush_interval: Duration::from_millis(1), // 优化: 1ms极低延迟
            max_memory_bytes: 8 * 1024 * 1024, // 8MB内存缓冲
        }
    }
}

/// 批量帧处理器
pub struct BatchProcessor {
    buffer: Vec<FrameEnvelope>,
    buffer_size_bytes: usize,
    config: BatchConfig,
    last_flush: Instant,
    tx: FrameSender,
}

impl BatchProcessor {
    pub fn new(tx: FrameSender, config: BatchConfig) -> Self {
        Self {
            buffer: Vec::with_capacity(config.max_batch_size),
            buffer_size_bytes: 0,
            config,
            last_flush: Instant::now(),
            tx,
        }
    }

    /// 添加帧到批量缓冲区
    pub fn add_envelope(&mut self, envelope: FrameEnvelope) -> Result<bool> {
        let envelope_size = envelope.payload.len() + 16; // 估算开销
        self.buffer.push(envelope);
        self.buffer_size_bytes += envelope_size;

        // 检查是否需要立即刷新
        let should_flush = self.buffer.len() >= self.config.max_batch_size
            || self.buffer_size_bytes >= self.config.max_memory_bytes
            || self.last_flush.elapsed() >= self.config.flush_interval;

        if should_flush {
            self.flush_batch()?;
            return Ok(true);
        }
        Ok(false)
    }

    /// 强制刷新批量缓冲区
    pub fn flush_batch(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let batch_size = self.buffer.len();
        let mut success_count = 0;
        let mut drop_count = 0;

        // 批量发送，避免阻塞
        for envelope in self.buffer.drain(..) {
            match self.tx.send(envelope) {
                Ok(_) => success_count += 1,
                Err(_) => drop_count += 1,
            }
        }

        // 更新指标
        METRICS.publish_total.inc_by(success_count as f64);
        METRICS.drop_total.inc_by(drop_count as f64);
        METRICS.ring_used.set(self.tx.len() as i64);
        
        // 更新批量处理指标
        METRICS.batch_size.observe(batch_size as f64);
        METRICS.batch_flush_duration.observe(self.last_flush.elapsed().as_secs_f64() * 1000.0);

        self.buffer_size_bytes = 0;
        self.last_flush = Instant::now();

        // 检查背压 - 使用默认配置
        let len = self.tx.len();
        let usage = len as f32 / (1 << 20) as f32; // 默认1M ring buffer
        if usage > 0.9 {
            tracing::warn!(
                "Ring buffer usage {:.1}% > pause threshold 90%, batch_size={}",
                usage * 100.0,
                batch_size
            );
        }

        Ok(())
    }
}

/// 异步批量发布器
pub struct AsyncBatchPublisher {
    tx: mpsc::UnboundedSender<FrameEnvelope>,
    _task_handle: tokio::task::JoinHandle<()>,
}

impl AsyncBatchPublisher {
    pub fn new(frame_tx: FrameSender, config: BatchConfig) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<FrameEnvelope>();
        let mut processor = BatchProcessor::new(frame_tx, config.clone());
        
        let _task_handle = tokio::spawn(async move {
            let mut flush_interval = interval(config.flush_interval);
            flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    // 接收新帧
                    envelope = rx.recv() => {
                        match envelope {
                            Some(env) => {
                                if let Err(e) = processor.add_envelope(env) {
                                    tracing::error!("批量处理器错误: {}", e);
                                }
                            }
                            None => break, // 通道关闭
                        }
                    }
                    // 定时刷新
                    _ = flush_interval.tick() => {
                        if let Err(e) = processor.flush_batch() {
                            tracing::error!("定时刷新失败: {}", e);
                        }
                    }
                }
            }
            
            // 最终清理
            let _ = processor.flush_batch();
            tracing::info!("批量发布器任务已结束");
        });

        Self { tx, _task_handle }
    }

    /// 异步发送帧
    pub fn send_envelope(&self, envelope: FrameEnvelope) -> Result<()> {
        self.tx.send(envelope)
            .map_err(|_| anyhow::anyhow!("批量发布器通道已关闭"))
    }
}

/// 初始化全局批量发布器 (优化版本)
pub fn init_batch_publisher(frame_tx: &FrameSender, config: Option<BatchConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let _publisher = AsyncBatchPublisher::new(frame_tx.clone(), config);
    
    // 更新全局实例中的批量发布器
    if let Some(_instance_ref) = GLOBAL_INSTANCE.get() {
        // 注意: 这里需要一个可变引用，但OnceCell不支持
        // 所以我们在初始化时直接创建完整实例
        info!("批量发布器已配置，但需要重新初始化FrameBus实例");
    }
    
    Ok(())
}

/// 获取全局批量发布器
pub fn get_batch_publisher() -> Result<&'static AsyncBatchPublisher> {
    get_instance()?
        .get_batch_publisher()
        .map(|arc| arc.as_ref())
        .ok_or_else(|| anyhow::anyhow!("批量发布器未初始化"))
}

/// 创建带批量发布器的FrameBus实例
pub fn init_with_batch_config(
    cfg: BusCfg, 
    batch_config: Option<BatchConfig>
) -> Result<(FrameSender, FrameReceiver)> {
    let capacity = 1 << cfg.ring_pow;
    let (tx, rx) = broadcast::channel(capacity);
    
    let receiver = FrameReceiver {
        inner: rx,
        filter: Filter::All,
    };
    
    // 创建带批量发布器的实例
    let batch_config = batch_config.unwrap_or_default();
    let publisher = AsyncBatchPublisher::new(tx.clone(), batch_config);
    let instance = Arc::new(
        FrameBusInstance::new(tx.clone(), cfg)
            .with_batch_publisher(publisher)
    );
    
    // 忽略重复初始化错误
    let _ = GLOBAL_INSTANCE.set(instance);

    Ok((tx, receiver))
}

/// 发布数据帧的便捷包装器
pub struct FramePublisher {
    tx: FrameSender,
    batch_mode: bool,
}

impl FramePublisher {
    pub fn new(tx: FrameSender) -> Self {
        Self { tx, batch_mode: false }
    }

    /// 创建启用批量模式的发布器
    pub fn new_batched(tx: FrameSender) -> Self {
        Self { tx, batch_mode: true }
    }

    /// 批量发送多个数据帧
    pub fn send_data_batch(&self, frames: Vec<DataFrame>) -> Result<()> {
        if frames.is_empty() {
            return Ok(());
        }

        let start_time = Instant::now();
        let mut success_count = 0;
        let mut error_count = 0;

        // 如果启用了全局批量发布器，优先使用
        if self.batch_mode {
            if let Ok(batch_publisher) = get_batch_publisher() {
                let frame_count = frames.len();
                
                for frame in frames {
                    let seq = SEQ_GENERATOR.fetch_add(1, Ordering::SeqCst);
                    match FrameEnvelope::wrap_data(seq, frame) {
                        Ok(envelope) => {
                            if let Err(_) = batch_publisher.send_envelope(envelope) {
                                error_count += 1;
                                METRICS.drop_total.inc();
                            } else {
                                success_count += 1;
                            }
                        }
                        Err(_) => error_count += 1,
                    }
                }
                
                // 记录批量操作指标
                let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;
                METRICS.batch_send_duration.observe(duration_ms);
                METRICS.batch_size.observe(frame_count as f64);
                
                return if error_count > 0 {
                    Err(anyhow::anyhow!("批量发送部分失败: {}/{} 成功", success_count, success_count + error_count))
                } else {
                    Ok(())
                };
            }
        }

        // fallback: 传统单独发送
        for frame in frames {
            match self.send_data(frame) {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }

        if error_count > 0 {
            Err(anyhow::anyhow!("批量发送部分失败: {}/{} 成功", success_count, success_count + error_count))
        } else {
            Ok(())
        }
    }

    pub fn send_data(&self, frame: DataFrame) -> Result<()> {
        let seq = SEQ_GENERATOR.fetch_add(1, Ordering::SeqCst);
        let envelope = FrameEnvelope::wrap_data(seq, frame)?;
        
        // 如果启用批量模式且批量发布器可用，使用批量发送
        if self.batch_mode {
            if let Ok(batch_publisher) = get_batch_publisher() {
                return batch_publisher.send_envelope(envelope)
                    .map_err(|_| anyhow::anyhow!("批量发布器发送失败"));
            }
        }
        
        // 传统直接发送
        match self.tx.send(envelope) {
            Ok(_) => {
                METRICS.publish_total.inc();
                // 更新ring使用率  
                let len = self.tx.len();
                METRICS.ring_used.set(len as i64);

                // 检查背压 (使用实例配置)
                if let Ok(instance) = get_instance() {
                    let cfg = instance.get_config();
                    let usage = len as f32 / (1 << cfg.ring_pow) as f32;
                    if usage > cfg.pause_hi {
                        tracing::warn!(
                            "Ring buffer usage {:.1}% > pause threshold {:.1}%, 建议启用批量模式", 
                            usage * 100.0, cfg.pause_hi * 100.0
                        );
                        // 背压控制: 稍微延迟新的发送
                        METRICS.drop_total.inc();
                    }
                }
                Ok(())
            }
            Err(_) => {
                METRICS.drop_total.inc();
                Err(anyhow::anyhow!("Ring buffer full"))
            }
        }
    }

    pub fn send_cmd(&self, frame: CmdFrame) -> Result<()> {
        let seq = SEQ_GENERATOR.fetch_add(1, Ordering::SeqCst);
        let envelope = FrameEnvelope::wrap_cmd(seq, frame)?;
        
        match self.tx.send(envelope) {
            Ok(_) => {
                METRICS.publish_total.inc();
                let len = self.tx.len();
                METRICS.ring_used.set(len as i64);
                Ok(())
            }
            Err(_) => {
                METRICS.drop_total.inc();
                Err(anyhow::anyhow!("Ring buffer full"))
            }
        }
    }
}