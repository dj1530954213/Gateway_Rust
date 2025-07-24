//! MQTT重连和消息缓冲管理
//!
//! 支持断网续传和指数退避重连

use crate::config::MqttMessage;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

/// 重连管理器
pub struct ReconnectManager {
    retry_count: usize,
    max_retries: usize,
    base_delay: Duration,
    max_delay: Duration,
    last_attempt: Option<Instant>,
}

impl ReconnectManager {
    /// 创建新的重连管理器
    pub fn new(max_retries: usize, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            retry_count: 0,
            max_retries,
            base_delay,
            max_delay,
            last_attempt: None,
        }
    }

    /// 获取当前重试延迟（指数退避）
    pub fn get_current_delay(&self) -> Duration {
        if self.retry_count == 0 {
            self.base_delay
        } else {
            let delay = self.base_delay * (2u32.pow(self.retry_count as u32));
            std::cmp::min(delay, self.max_delay)
        }
    }

    /// 记录重连尝试
    pub fn record_attempt(&mut self) {
        self.retry_count += 1;
        self.last_attempt = Some(Instant::now());
    }

    /// 重置重试计数（连接成功时调用）
    pub fn reset(&mut self) {
        self.retry_count = 0;
        self.last_attempt = None;
    }

    /// 检查是否可以尝试重连
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// 获取当前重试次数
    pub fn get_retry_count(&self) -> usize {
        self.retry_count
    }

    /// 检查是否已达到最大重试次数
    pub fn is_exhausted(&self) -> bool {
        self.retry_count >= self.max_retries
    }

    /// 异步等待重连延迟
    pub async fn wait_for_retry(&self) {
        if self.retry_count > 0 {
            let delay = self.get_current_delay();
            sleep(delay).await;
        }
    }
}

/// 消息缓冲器，用于断网时缓存消息
pub struct MessageBuffer {
    buffer: VecDeque<MqttMessage>,
    max_size: usize,
    dropped_count: usize,
}

impl MessageBuffer {
    /// 创建新的消息缓冲器
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: VecDeque::new(),
            max_size,
            dropped_count: 0,
        }
    }

    /// 添加消息到缓冲区
    pub fn push(&mut self, message: MqttMessage) {
        if self.buffer.len() >= self.max_size {
            // 缓冲区已满，丢弃最旧的消息
            self.buffer.pop_front();
            self.dropped_count += 1;
        }
        self.buffer.push_back(message);
    }

    /// 从缓冲区获取下一个消息
    pub fn pop(&mut self) -> Option<MqttMessage> {
        self.buffer.pop_front()
    }

    /// 获取缓冲区中的消息数量
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// 获取被丢弃的消息数量
    pub fn dropped_count(&self) -> usize {
        self.dropped_count
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// 获取缓冲区剩余容量
    pub fn remaining_capacity(&self) -> usize {
        self.max_size.saturating_sub(self.buffer.len())
    }

    /// 检查缓冲区是否已满
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.max_size
    }

    /// 预览下一个消息（不移除）
    pub fn peek(&self) -> Option<&MqttMessage> {
        self.buffer.front()
    }
}

/// 飞行中消息跟踪器（QoS > 0时使用）
pub struct InflightTracker {
    inflight_messages: std::collections::HashMap<u16, (MqttMessage, Instant)>,
    max_inflight: usize,
    timeout: Duration,
    next_packet_id: u16,
}

impl InflightTracker {
    /// 创建新的飞行中消息跟踪器
    pub fn new(max_inflight: usize, timeout: Duration) -> Self {
        Self {
            inflight_messages: std::collections::HashMap::new(),
            max_inflight,
            timeout,
            next_packet_id: 1,
        }
    }

    /// 获取下一个可用的包ID
    pub fn next_packet_id(&mut self) -> Option<u16> {
        if self.inflight_messages.len() >= self.max_inflight {
            return None; // 已达到最大飞行中消息数
        }

        let start_id = self.next_packet_id;
        loop {
            let id = self.next_packet_id;
            self.next_packet_id = if self.next_packet_id == u16::MAX { 1 } else { self.next_packet_id + 1 };
            
            if !self.inflight_messages.contains_key(&id) {
                return Some(id);
            }
            
            // 防止无限循环
            if self.next_packet_id == start_id {
                return None;
            }
        }
    }

    /// 记录发送的消息
    pub fn track_message(&mut self, packet_id: u16, message: MqttMessage) {
        self.inflight_messages.insert(packet_id, (message, Instant::now()));
    }

    /// 确认消息已收到
    pub fn acknowledge(&mut self, packet_id: u16) -> Option<MqttMessage> {
        self.inflight_messages.remove(&packet_id).map(|(msg, _)| msg)
    }

    /// 获取超时的消息
    pub fn get_timeout_messages(&mut self) -> Vec<(u16, MqttMessage)> {
        let now = Instant::now();
        let mut timeouts = Vec::new();
        
        self.inflight_messages.retain(|&packet_id, (message, sent_time)| {
            if now.duration_since(*sent_time) > self.timeout {
                timeouts.push((packet_id, message.clone()));
                false // 从映射中移除
            } else {
                true // 保留在映射中
            }
        });
        
        timeouts
    }

    /// 获取飞行中消息数量
    pub fn inflight_count(&self) -> usize {
        self.inflight_messages.len()
    }

    /// 检查是否可以发送更多消息
    pub fn can_send(&self) -> bool {
        self.inflight_messages.len() < self.max_inflight
    }

    /// 清空所有飞行中消息
    pub fn clear(&mut self) -> Vec<MqttMessage> {
        let messages = self.inflight_messages.drain()
            .map(|(_, (msg, _))| msg)
            .collect();
        messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DataPoint;
    use std::collections::HashMap;
    use serde_json::Value;

    fn create_test_message(device_id: &str, point_count: usize) -> MqttMessage {
        let points = (0..point_count).map(|i| {
            DataPoint {
                tag: format!("sensor_{}", i),
                value: Value::Number(serde_json::Number::from_f64(i as f64 * 0.5).unwrap()),
                quality: 2,
                meta: HashMap::new(),
            }
        }).collect();

        MqttMessage {
            device_id: device_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            points,
        }
    }

    #[test]
    fn test_reconnect_manager_exponential_backoff() {
        let mut manager = ReconnectManager::new(
            5,
            Duration::from_secs(1),
            Duration::from_secs(30),
        );

        // 第一次尝试
        assert_eq!(manager.get_current_delay(), Duration::from_secs(1));
        assert!(manager.can_retry());
        
        manager.record_attempt();
        assert_eq!(manager.get_retry_count(), 1);
        
        // 第二次尝试 - 延迟应该翻倍
        assert_eq!(manager.get_current_delay(), Duration::from_secs(2));
        manager.record_attempt();
        
        // 第三次尝试
        assert_eq!(manager.get_current_delay(), Duration::from_secs(4));
        manager.record_attempt();
        
        // 重置后应该回到初始状态
        manager.reset();
        assert_eq!(manager.get_retry_count(), 0);
        assert_eq!(manager.get_current_delay(), Duration::from_secs(1));
    }

    #[test]
    fn test_message_buffer() {
        let mut buffer = MessageBuffer::new(3);
        
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.remaining_capacity(), 3);
        
        // 添加消息
        buffer.push(create_test_message("device1", 1));
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_full());
        
        buffer.push(create_test_message("device2", 1));
        buffer.push(create_test_message("device3", 1));
        assert!(buffer.is_full());
        assert_eq!(buffer.len(), 3);
        
        // 缓冲区满时再添加应该丢弃最旧的
        buffer.push(create_test_message("device4", 1));
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.dropped_count(), 1);
        
        // 取出消息
        let msg = buffer.pop().unwrap();
        assert_eq!(msg.device_id, "device2"); // device1被丢弃了
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn test_inflight_tracker() {
        let mut tracker = InflightTracker::new(2, Duration::from_secs(5));
        
        assert!(tracker.can_send());
        assert_eq!(tracker.inflight_count(), 0);
        
        // 获取包ID并跟踪消息
        let id1 = tracker.next_packet_id().unwrap();
        tracker.track_message(id1, create_test_message("device1", 1));
        assert_eq!(tracker.inflight_count(), 1);
        
        let id2 = tracker.next_packet_id().unwrap();
        tracker.track_message(id2, create_test_message("device2", 1));
        assert_eq!(tracker.inflight_count(), 2);
        assert!(!tracker.can_send()); // 已达到最大值
        
        // 应该无法获取更多包ID
        assert!(tracker.next_packet_id().is_none());
        
        // 确认消息
        let msg = tracker.acknowledge(id1).unwrap();
        assert_eq!(msg.device_id, "device1");
        assert_eq!(tracker.inflight_count(), 1);
        assert!(tracker.can_send());
    }

    #[test]
    fn test_inflight_timeout() {
        let mut tracker = InflightTracker::new(10, Duration::from_millis(100));
        
        let id = tracker.next_packet_id().unwrap();
        tracker.track_message(id, create_test_message("timeout-device", 1));
        
        // 立即检查不应该有超时
        let timeouts = tracker.get_timeout_messages();
        assert!(timeouts.is_empty());
        
        // 等待超时
        std::thread::sleep(Duration::from_millis(150));
        let timeouts = tracker.get_timeout_messages();
        assert_eq!(timeouts.len(), 1);
        assert_eq!(timeouts[0].0, id);
        assert_eq!(timeouts[0].1.device_id, "timeout-device");
        
        // 消息应该从跟踪器中移除
        assert_eq!(tracker.inflight_count(), 0);
    }
}