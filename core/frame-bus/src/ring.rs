//! 环形缓冲区和广播

use tokio::sync::broadcast;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use anyhow::Result;

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

/// 全局序列号生成器
static SEQ_GENERATOR: AtomicU64 = AtomicU64::new(0);

/// 全局发送端
static GLOBAL_SENDER: OnceCell<FrameSender> = OnceCell::new();

/// 全局配置
static GLOBAL_CONFIG: OnceCell<BusCfg> = OnceCell::new();

/// 测试专用：全局状态保护锁
static TEST_RESET_LOCK: Mutex<()> = Mutex::new(());

/// 初始化ring
pub fn init(cfg: BusCfg) -> Result<(FrameSender, FrameReceiver)> {
    // 如果已经初始化，返回现有的发送端和新的接收端
    if let Some(existing_tx) = GLOBAL_SENDER.get() {
        let receiver = FrameReceiver {
            inner: existing_tx.subscribe(),
            filter: Filter::All,
        };
        return Ok((existing_tx.clone(), receiver));
    }
    
    let capacity = 1 << cfg.ring_pow;
    let (tx, rx) = broadcast::channel(capacity);
    
    let receiver = FrameReceiver {
        inner: rx,
        filter: Filter::All,
    };

    // 忽略重复初始化错误 - 支持幂等操作
    let _ = GLOBAL_SENDER.set(tx.clone());
    let _ = GLOBAL_CONFIG.set(cfg);

    Ok((tx, receiver))
}

/// 获取全局发送端
pub fn get_publisher() -> Result<&'static FrameSender> {
    GLOBAL_SENDER.get().ok_or_else(|| anyhow::anyhow!("FrameBus not initialized"))
}

/// 订阅指定过滤器
pub fn subscribe(filter: Filter) -> Result<FrameReceiver> {
    let tx = get_publisher()?;
    let rx = tx.subscribe();
    Ok(FrameReceiver {
        inner: rx,
        filter,
    })
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

/// 发布数据帧的便捷包装器
pub struct FramePublisher {
    tx: FrameSender,
}

impl FramePublisher {
    pub fn new(tx: FrameSender) -> Self {
        Self { tx }
    }

    pub fn send_data(&self, frame: DataFrame) -> Result<()> {
        let seq = SEQ_GENERATOR.fetch_add(1, Ordering::SeqCst);
        let envelope = FrameEnvelope::wrap_data(seq, frame)?;
        
        match self.tx.send(envelope) {
            Ok(_) => {
                METRICS.publish_total.inc();
                // 更新ring使用率  
                let len = self.tx.len();
                METRICS.ring_used.set(len as i64);

                // 检查背压
                if let Some(cfg) = GLOBAL_CONFIG.get() {
                    // 使用配置的容量而非API获取（broadcast已改变）
                    let usage = len as f32 / (1 << cfg.ring_pow) as f32;
                    if usage > cfg.pause_hi {
                        // TODO: 发送暂停信号
                        tracing::warn!("Ring buffer usage {}% > pause threshold {}%", 
                                     usage * 100.0, cfg.pause_hi * 100.0);
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