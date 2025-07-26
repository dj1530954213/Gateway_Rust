/*!
# Backpressure Control System

背压控制系统，基于80%/60%阈值实现Pause/Resume广播机制
*/

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::{info, warn, debug};

/// 背压控制信号类型
#[derive(Debug, Clone)]
pub enum BackpressureSignal {
    /// 暂停信号：队列达到高水位
    Pause {
        component: String,
        queue_depth: usize,
        threshold: usize,
        timestamp: u64,
    },
    /// 恢复信号：队列降到低水位
    Resume {
        component: String,
        queue_depth: usize,
        threshold: usize,
        timestamp: u64,
    },
}

/// 背压控制配置
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// 高水位阈值（触发暂停）
    pub high_watermark: f64,
    /// 低水位阈值（触发恢复）
    pub low_watermark: f64,
    /// 队列最大容量
    pub max_capacity: usize,
    /// 检查间隔
    pub check_interval: Duration,
    /// 广播通道容量
    pub broadcast_capacity: usize,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            high_watermark: 0.8,  // 80%
            low_watermark: 0.6,   // 60%
            max_capacity: 10000,
            check_interval: Duration::from_millis(100),
            broadcast_capacity: 1000,
        }
    }
}

/// 队列状态
#[derive(Debug, Clone)]
pub struct QueueStatus {
    pub current_size: usize,
    pub max_capacity: usize,
    pub utilization: f64,
    pub is_paused: bool,
    pub last_pause_time: Option<Instant>,
    pub last_resume_time: Option<Instant>,
}

/// 背压管理器
#[derive(Debug)]
pub struct BackpressureManager {
    config: BackpressureConfig,
    sender: broadcast::Sender<BackpressureSignal>,
    receiver: broadcast::Receiver<BackpressureSignal>,
    queues: Arc<Mutex<HashMap<String, QueueMetrics>>>,
    is_paused: AtomicBool,
    stats: BackpressureStats,
}

/// 队列指标
#[derive(Debug)]
struct QueueMetrics {
    current_size: AtomicUsize,
    max_capacity: usize,
    is_paused: AtomicBool,
    last_pause_time: Mutex<Option<Instant>>,
    last_resume_time: Mutex<Option<Instant>>,
    pause_count: AtomicU64,
    resume_count: AtomicU64,
}

/// 背压统计信息
#[derive(Debug, Default)]
struct BackpressureStats {
    total_pause_events: AtomicU64,
    total_resume_events: AtomicU64,
    total_queue_updates: AtomicU64,
    last_pause_time: Mutex<Option<Instant>>,
    last_resume_time: Mutex<Option<Instant>>,
}

impl BackpressureManager {
    /// 创建新的背压管理器
    pub fn new(config: BackpressureConfig) -> Self {
        let (sender, receiver) = broadcast::channel(config.broadcast_capacity);
        
        Self {
            config,
            sender,
            receiver,
            queues: Arc::new(Mutex::new(HashMap::new())),
            is_paused: AtomicBool::new(false),
            stats: BackpressureStats::default(),
        }
    }

    /// 使用默认配置创建背压管理器
    pub fn with_defaults() -> Self {
        Self::new(BackpressureConfig::default())
    }

    /// 注册队列
    pub fn register_queue(&self, name: String, max_capacity: usize) {
        let mut queues = self.queues.lock().unwrap();
        queues.insert(name.clone(), QueueMetrics {
            current_size: AtomicUsize::new(0),
            max_capacity,
            is_paused: AtomicBool::new(false),
            last_pause_time: Mutex::new(None),
            last_resume_time: Mutex::new(None),
            pause_count: AtomicU64::new(0),
            resume_count: AtomicU64::new(0),
        });
        
        debug!("Registered queue '{}' with capacity {}", name, max_capacity);
    }

    /// 更新队列深度
    pub fn update_queue_depth(&self, queue_name: &str, current_size: usize) {
        self.stats.total_queue_updates.fetch_add(1, Ordering::Relaxed);
        
        let queues = self.queues.lock().unwrap();
        if let Some(metrics) = queues.get(queue_name) {
            let old_size = metrics.current_size.swap(current_size, Ordering::Relaxed);
            let max_capacity = metrics.max_capacity;
            
            let utilization = current_size as f64 / max_capacity as f64;
            
            // 检查是否需要暂停
            if !metrics.is_paused.load(Ordering::Relaxed) && 
               utilization >= self.config.high_watermark {
                self.trigger_pause(queue_name, current_size, max_capacity, metrics);
            }
            // 检查是否需要恢复
            else if metrics.is_paused.load(Ordering::Relaxed) && 
                    utilization <= self.config.low_watermark {
                self.trigger_resume(queue_name, current_size, max_capacity, metrics);
            }
            
            debug!("Queue '{}' depth: {} -> {} ({:.1}% utilization)", 
                   queue_name, old_size, current_size, utilization * 100.0);
        }
    }

    /// 触发暂停信号
    fn trigger_pause(&self, queue_name: &str, current_size: usize, max_capacity: usize, metrics: &QueueMetrics) {
        let high_threshold = (max_capacity as f64 * self.config.high_watermark) as usize;
        
        if metrics.is_paused.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            let now = Instant::now();
            *metrics.last_pause_time.lock().unwrap() = Some(now);
            metrics.pause_count.fetch_add(1, Ordering::Relaxed);
            
            self.stats.total_pause_events.fetch_add(1, Ordering::Relaxed);
            *self.stats.last_pause_time.lock().unwrap() = Some(now);
            
            let signal = BackpressureSignal::Pause {
                component: queue_name.to_string(),
                queue_depth: current_size,
                threshold: high_threshold,
                timestamp: now.elapsed().as_nanos() as u64,
            };
            
            if let Err(e) = self.sender.send(signal) {
                warn!("Failed to send pause signal for queue '{}': {}", queue_name, e);
            } else {
                warn!("Queue '{}' paused: {}/{} ({}%)", 
                      queue_name, current_size, max_capacity, 
                      (current_size as f64 / max_capacity as f64 * 100.0) as u32);
            }
        }
    }

    /// 触发恢复信号
    fn trigger_resume(&self, queue_name: &str, current_size: usize, max_capacity: usize, metrics: &QueueMetrics) {
        let low_threshold = (max_capacity as f64 * self.config.low_watermark) as usize;
        
        if metrics.is_paused.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            let now = Instant::now();
            *metrics.last_resume_time.lock().unwrap() = Some(now);
            metrics.resume_count.fetch_add(1, Ordering::Relaxed);
            
            self.stats.total_resume_events.fetch_add(1, Ordering::Relaxed);
            *self.stats.last_resume_time.lock().unwrap() = Some(now);
            
            let signal = BackpressureSignal::Resume {
                component: queue_name.to_string(),
                queue_depth: current_size,
                threshold: low_threshold,
                timestamp: now.elapsed().as_nanos() as u64,
            };
            
            if let Err(e) = self.sender.send(signal) {
                warn!("Failed to send resume signal for queue '{}': {}", queue_name, e);
            } else {
                info!("Queue '{}' resumed: {}/{} ({}%)", 
                      queue_name, current_size, max_capacity,
                      (current_size as f64 / max_capacity as f64 * 100.0) as u32);
            }
        }
    }

    /// 获取队列状态
    pub fn get_queue_status(&self, queue_name: &str) -> Option<QueueStatus> {
        let queues = self.queues.lock().unwrap();
        queues.get(queue_name).map(|metrics| {
            QueueStatus {
                current_size: metrics.current_size.load(Ordering::Relaxed),
                max_capacity: metrics.max_capacity,
                utilization: {
                    let size = metrics.current_size.load(Ordering::Relaxed);
                    size as f64 / metrics.max_capacity as f64
                },
                is_paused: metrics.is_paused.load(Ordering::Relaxed),
                last_pause_time: *metrics.last_pause_time.lock().unwrap(),
                last_resume_time: *metrics.last_resume_time.lock().unwrap(),
            }
        })
    }

    /// 获取所有队列状态
    pub fn get_all_queue_status(&self) -> HashMap<String, QueueStatus> {
        let queues = self.queues.lock().unwrap();
        queues.iter().map(|(name, metrics)| {
            let status = QueueStatus {
                current_size: metrics.current_size.load(Ordering::Relaxed),
                max_capacity: metrics.max_capacity,
                utilization: {
                    let size = metrics.current_size.load(Ordering::Relaxed);
                    size as f64 / metrics.max_capacity as f64
                },
                is_paused: metrics.is_paused.load(Ordering::Relaxed),
                last_pause_time: *metrics.last_pause_time.lock().unwrap(),
                last_resume_time: *metrics.last_resume_time.lock().unwrap(),
            };
            (name.clone(), status)
        }).collect()
    }

    /// 订阅背压信号
    pub fn subscribe(&self) -> broadcast::Receiver<BackpressureSignal> {
        self.sender.subscribe()
    }

    /// 检查系统是否处于暂停状态
    pub fn is_system_paused(&self) -> bool {
        let queues = self.queues.lock().unwrap();
        queues.values().any(|metrics| metrics.is_paused.load(Ordering::Relaxed))
    }

    /// 手动暂停队列
    pub fn manual_pause(&self, queue_name: &str) -> Result<(), String> {
        let queues = self.queues.lock().unwrap();
        if let Some(metrics) = queues.get(queue_name) {
            let current_size = metrics.current_size.load(Ordering::Relaxed);
            self.trigger_pause(queue_name, current_size, metrics.max_capacity, metrics);
            Ok(())
        } else {
            Err(format!("Queue '{}' not found", queue_name))
        }
    }

    /// 手动恢复队列
    pub fn manual_resume(&self, queue_name: &str) -> Result<(), String> {
        let queues = self.queues.lock().unwrap();
        if let Some(metrics) = queues.get(queue_name) {
            let current_size = metrics.current_size.load(Ordering::Relaxed);
            self.trigger_resume(queue_name, current_size, metrics.max_capacity, metrics);
            Ok(())
        } else {
            Err(format!("Queue '{}' not found", queue_name))
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> BackpressureManagerStats {
        BackpressureManagerStats {
            total_pause_events: self.stats.total_pause_events.load(Ordering::Relaxed),
            total_resume_events: self.stats.total_resume_events.load(Ordering::Relaxed),
            total_queue_updates: self.stats.total_queue_updates.load(Ordering::Relaxed),
            last_pause_time: *self.stats.last_pause_time.lock().unwrap(),
            last_resume_time: *self.stats.last_resume_time.lock().unwrap(),
            queues_count: self.queues.lock().unwrap().len(),
            system_paused: self.is_system_paused(),
        }
    }

    /// 启动后台监控任务
    pub async fn start_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let manager = Arc::new(std::sync::Mutex::new(self));
        let interval = self.config.check_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // 这里可以添加定期检查逻辑
                // 例如：检查队列是否长时间暂停，发送告警等
                debug!("Backpressure monitoring tick");
            }
        })
    }
}

/// 背压管理器统计信息
#[derive(Debug)]
pub struct BackpressureManagerStats {
    pub total_pause_events: u64,
    pub total_resume_events: u64,
    pub total_queue_updates: u64,
    pub last_pause_time: Option<Instant>,
    pub last_resume_time: Option<Instant>,
    pub queues_count: usize,
    pub system_paused: bool,
}

/// 背压感知的队列包装器
#[derive(Debug)]
pub struct BackpressureAwareQueue<T> {
    queue: tokio::sync::mpsc::UnboundedSender<T>,
    manager: Arc<BackpressureManager>,
    name: String,
    capacity: usize,
    current_size: AtomicUsize,
}

impl<T> BackpressureAwareQueue<T> {
    /// 创建新的背压感知队列
    pub fn new(
        name: String,
        capacity: usize,
        manager: Arc<BackpressureManager>,
    ) -> (Self, tokio::sync::mpsc::UnboundedReceiver<T>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        manager.register_queue(name.clone(), capacity);
        
        let queue = Self {
            queue: tx,
            manager,
            name,
            capacity,
            current_size: AtomicUsize::new(0),
        };
        
        (queue, rx)
    }

    /// 发送消息
    pub fn send(&self, item: T) -> Result<(), tokio::sync::mpsc::error::SendError<T>> {
        let result = self.queue.send(item);
        
        if result.is_ok() {
            let new_size = self.current_size.fetch_add(1, Ordering::Relaxed) + 1;
            self.manager.update_queue_depth(&self.name, new_size);
        }
        
        result
    }

    /// 标记消息已处理
    pub fn mark_processed(&self) {
        let new_size = self.current_size.fetch_sub(1, Ordering::Relaxed).saturating_sub(1);
        self.manager.update_queue_depth(&self.name, new_size);
    }

    /// 获取当前队列大小
    pub fn len(&self) -> usize {
        self.current_size.load(Ordering::Relaxed)
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 获取队列状态
    pub fn status(&self) -> Option<QueueStatus> {
        self.manager.get_queue_status(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_backpressure_manager() {
        let config = BackpressureConfig {
            high_watermark: 0.8,
            low_watermark: 0.6,
            max_capacity: 100,
            check_interval: Duration::from_millis(10),
            broadcast_capacity: 10,
        };
        
        let manager = BackpressureManager::new(config);
        manager.register_queue("test".to_string(), 100);
        
        // 测试正常状态
        manager.update_queue_depth("test", 50);
        let status = manager.get_queue_status("test").unwrap();
        assert!(!status.is_paused);
        assert_eq!(status.utilization, 0.5);
        
        // 测试触发暂停
        manager.update_queue_depth("test", 85);
        let status = manager.get_queue_status("test").unwrap();
        assert!(status.is_paused);
        
        // 测试恢复
        manager.update_queue_depth("test", 50);
        let status = manager.get_queue_status("test").unwrap();
        assert!(!status.is_paused);
    }

    #[tokio::test]
    async fn test_backpressure_aware_queue() {
        let manager = Arc::new(BackpressureManager::with_defaults());
        let (queue, mut rx) = BackpressureAwareQueue::new(
            "test_queue".to_string(),
            10,
            manager.clone(),
        );
        
        // 发送消息
        queue.send(1).unwrap();
        queue.send(2).unwrap();
        
        assert_eq!(queue.len(), 2);
        
        // 接收消息
        let item = rx.recv().await.unwrap();
        assert_eq!(item, 1);
        queue.mark_processed();
        
        assert_eq!(queue.len(), 1);
    }

    #[tokio::test]
    async fn test_backpressure_signals() {
        let manager = BackpressureManager::with_defaults();
        let mut receiver = manager.subscribe();
        
        manager.register_queue("signal_test".to_string(), 10);
        
        // 触发暂停信号
        manager.update_queue_depth("signal_test", 9); // 90% > 80%
        
        let signal = receiver.recv().await.unwrap();
        match signal {
            BackpressureSignal::Pause { component, .. } => {
                assert_eq!(component, "signal_test");
            }
            _ => panic!("Expected pause signal"),
        }
        
        // 触发恢复信号
        manager.update_queue_depth("signal_test", 5); // 50% < 60%
        
        let signal = receiver.recv().await.unwrap();
        match signal {
            BackpressureSignal::Resume { component, .. } => {
                assert_eq!(component, "signal_test");
            }
            _ => panic!("Expected resume signal"),
        }
    }
}