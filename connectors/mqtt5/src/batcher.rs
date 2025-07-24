//! MQTT消息批量处理器
//!
//! 支持按大小和时间触发的批量发送机制

use crate::config::{BatchCfg, MqttMessage, DataPoint};
use std::time::{Duration, Instant};

/// MQTT消息批处理器
pub struct Batcher {
    config: BatchCfg,
    current_batch: Vec<DataPoint>,
    last_send: Instant,
    total_sent: usize,
}

impl Batcher {
    /// 创建新的批处理器
    pub fn new(config: BatchCfg) -> Self {
        Self {
            config,
            current_batch: Vec::new(),
            last_send: Instant::now(),
            total_sent: 0,
        }
    }

    /// 添加数据点到当前批次
    /// 返回是否应该发送批次
    pub fn add_point(&mut self, point: DataPoint) -> bool {
        self.current_batch.push(point);
        self.should_send_batch()
    }

    /// 检查是否应该发送当前批次
    pub fn should_send_batch(&self) -> bool {
        !self.current_batch.is_empty() && (
            self.current_batch.len() >= self.config.size ||
            self.last_send.elapsed() >= self.config.timeout
        )
    }

    /// 创建批次消息并重置当前批次
    pub fn create_and_reset_batch(&mut self, device_id: String) -> Option<MqttMessage> {
        if self.current_batch.is_empty() {
            return None;
        }

        let message = MqttMessage {
            device_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            points: self.current_batch.drain(..).collect(),
        };

        self.last_send = Instant::now();
        self.total_sent += 1;
        Some(message)
    }

    /// 强制发送当前批次（即使未达到触发条件）
    pub fn force_send(&mut self, device_id: String) -> Option<MqttMessage> {
        if !self.current_batch.is_empty() {
            self.create_and_reset_batch(device_id)
        } else {
            None
        }
    }

    /// 获取已发送批次总数
    pub fn get_total_sent(&self) -> usize {
        self.total_sent
    }

    /// 获取当前批次大小
    pub fn get_current_batch_size(&self) -> usize {
        self.current_batch.len()
    }

    /// 获取自上次发送以来的时间
    pub fn time_since_last_send(&self) -> Duration {
        self.last_send.elapsed()
    }

    /// 检查批次是否为空
    pub fn is_empty(&self) -> bool {
        self.current_batch.is_empty()
    }

    /// 清空当前批次（不发送）
    pub fn clear(&mut self) {
        self.current_batch.clear();
    }

    /// 获取剩余容量（到达批次大小限制还需多少个点）
    pub fn remaining_capacity(&self) -> usize {
        self.config.size.saturating_sub(self.current_batch.len())
    }

    /// 获取到超时还剩多少时间
    pub fn time_until_timeout(&self) -> Option<Duration> {
        let elapsed = self.last_send.elapsed();
        if elapsed >= self.config.timeout {
            None // 已超时
        } else {
            Some(self.config.timeout - elapsed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use serde_json::Value;

    fn create_test_point(tag: &str, value: f64) -> DataPoint {
        DataPoint {
            tag: tag.to_string(),
            value: Value::Number(serde_json::Number::from_f64(value).unwrap()),
            quality: 2,
            meta: HashMap::new(),
        }
    }

    #[test]
    fn test_batcher_creation() {
        let config = BatchCfg {
            size: 10,
            timeout: Duration::from_secs(5),
        };
        let batcher = Batcher::new(config);
        
        assert_eq!(batcher.get_current_batch_size(), 0);
        assert_eq!(batcher.get_total_sent(), 0);
        assert!(batcher.is_empty());
        assert_eq!(batcher.remaining_capacity(), 10);
    }

    #[test]
    fn test_add_point_and_size_trigger() {
        let config = BatchCfg {
            size: 2,
            timeout: Duration::from_secs(10),
        };
        let mut batcher = Batcher::new(config);

        // 第一个点不应触发
        assert!(!batcher.add_point(create_test_point("point1", 1.0)));
        assert_eq!(batcher.get_current_batch_size(), 1);
        assert_eq!(batcher.remaining_capacity(), 1);

        // 第二个点应该触发
        assert!(batcher.add_point(create_test_point("point2", 2.0)));
        assert_eq!(batcher.get_current_batch_size(), 2);
        assert_eq!(batcher.remaining_capacity(), 0);
    }

    #[test]
    fn test_timeout_detection() {
        let config = BatchCfg {
            size: 10,
            timeout: Duration::from_millis(50),
        };
        let mut batcher = Batcher::new(config);

        // 添加一个点
        batcher.add_point(create_test_point("point1", 1.0));
        assert!(!batcher.should_send_batch());

        // 等待超时
        std::thread::sleep(Duration::from_millis(60));
        assert!(batcher.should_send_batch());
    }

    #[test]
    fn test_create_and_reset_batch() {
        let config = BatchCfg {
            size: 2,
            timeout: Duration::from_secs(10),
        };
        let mut batcher = Batcher::new(config);

        batcher.add_point(create_test_point("point1", 1.0));
        batcher.add_point(create_test_point("point2", 2.0));

        let batch = batcher.create_and_reset_batch("test-device".to_string()).unwrap();
        
        assert_eq!(batch.device_id, "test-device");
        assert_eq!(batch.points.len(), 2);
        assert_eq!(batcher.get_current_batch_size(), 0);
        assert_eq!(batcher.get_total_sent(), 1);
        assert!(batcher.is_empty());
    }

    #[test]
    fn test_force_send() {
        let config = BatchCfg {
            size: 10,
            timeout: Duration::from_secs(10),
        };
        let mut batcher = Batcher::new(config);

        batcher.add_point(create_test_point("point1", 1.0));
        assert!(!batcher.should_send_batch());

        let batch = batcher.force_send("force-device".to_string()).unwrap();
        assert_eq!(batch.points.len(), 1);
        assert_eq!(batcher.get_current_batch_size(), 0);
    }

    #[test]
    fn test_empty_batch_handling() {
        let config = BatchCfg {
            size: 5,
            timeout: Duration::from_secs(5),
        };
        let mut batcher = Batcher::new(config);

        assert!(batcher.create_and_reset_batch("empty".to_string()).is_none());
        assert!(batcher.force_send("empty".to_string()).is_none());
    }
}