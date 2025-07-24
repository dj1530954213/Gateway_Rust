//! MQTT批量处理和缓冲测试

use mqtt5::config::{MqttMessage, DataPoint, BatchCfg};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde_json::Value;

/// 模拟的批处理器
struct MockBatcher {
    batch_cfg: BatchCfg,
    current_batch: Vec<DataPoint>,
    last_send: Instant,
    total_sent: usize,
}

impl MockBatcher {
    fn new(batch_cfg: BatchCfg) -> Self {
        Self {
            batch_cfg,
            current_batch: Vec::new(),
            last_send: Instant::now(),
            total_sent: 0,
        }
    }

    fn add_point(&mut self, point: DataPoint) -> bool {
        self.current_batch.push(point);
        self.should_send_batch()
    }

    fn should_send_batch(&self) -> bool {
        !self.current_batch.is_empty() && (
            self.current_batch.len() >= self.batch_cfg.size ||
            self.last_send.elapsed() >= self.batch_cfg.timeout
        )
    }

    fn create_and_reset_batch(&mut self, device_id: String) -> Option<MqttMessage> {
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

    fn force_send(&mut self, device_id: String) -> Option<MqttMessage> {
        if !self.current_batch.is_empty() {
            self.create_and_reset_batch(device_id)
        } else {
            None
        }
    }

    fn get_total_sent(&self) -> usize {
        self.total_sent
    }

    fn get_current_batch_size(&self) -> usize {
        self.current_batch.len()
    }
}

#[test]
fn test_batch_size_trigger() {
    let batch_cfg = BatchCfg {
        size: 3,
        timeout: Duration::from_secs(10), // 长超时，确保只由大小触发
    };

    let mut batcher = MockBatcher::new(batch_cfg);

    // 添加数据点，不应该触发发送
    assert!(!batcher.add_point(create_test_point("point1", 1.0)));
    assert_eq!(batcher.get_current_batch_size(), 1);

    assert!(!batcher.add_point(create_test_point("point2", 2.0)));
    assert_eq!(batcher.get_current_batch_size(), 2);

    // 第三个数据点应该触发批量发送
    assert!(batcher.add_point(create_test_point("point3", 3.0)));
    assert_eq!(batcher.get_current_batch_size(), 3);

    // 创建批次
    let batch = batcher.create_and_reset_batch("test-device".to_string()).unwrap();
    assert_eq!(batch.points.len(), 3);
    assert_eq!(batch.device_id, "test-device");
    assert_eq!(batcher.get_current_batch_size(), 0);
    assert_eq!(batcher.get_total_sent(), 1);
}

#[test]
fn test_batch_timeout_trigger() {
    let batch_cfg = BatchCfg {
        size: 10, // 大批次大小，确保只由超时触发
        timeout: Duration::from_millis(50),
    };

    let mut batcher = MockBatcher::new(batch_cfg);

    // 添加少量数据点
    assert!(!batcher.add_point(create_test_point("point1", 1.0)));
    assert!(!batcher.add_point(create_test_point("point2", 2.0)));
    assert_eq!(batcher.get_current_batch_size(), 2);

    // 等待超时
    std::thread::sleep(Duration::from_millis(60));

    // 现在应该触发发送
    assert!(batcher.should_send_batch());

    let batch = batcher.create_and_reset_batch("timeout-device".to_string()).unwrap();
    assert_eq!(batch.points.len(), 2);
    assert_eq!(batcher.get_current_batch_size(), 0);
}

#[test]
fn test_mixed_batch_triggers() {
    let batch_cfg = BatchCfg {
        size: 3,
        timeout: Duration::from_millis(100),
    };

    let mut batcher = MockBatcher::new(batch_cfg);

    // 快速添加足够的点触发大小限制
    assert!(!batcher.add_point(create_test_point("fast1", 1.0)));
    assert!(!batcher.add_point(create_test_point("fast2", 2.0)));
    assert!(batcher.add_point(create_test_point("fast3", 3.0))); // 应该触发

    let batch1 = batcher.create_and_reset_batch("device1".to_string()).unwrap();
    assert_eq!(batch1.points.len(), 3);

    // 添加少量点然后等待超时
    assert!(!batcher.add_point(create_test_point("slow1", 4.0)));
    std::thread::sleep(Duration::from_millis(110));

    assert!(batcher.should_send_batch());
    let batch2 = batcher.create_and_reset_batch("device2".to_string()).unwrap();
    assert_eq!(batch2.points.len(), 1);

    assert_eq!(batcher.get_total_sent(), 2);
}

#[test]
fn test_empty_batch_handling() {
    let batch_cfg = BatchCfg {
        size: 5,
        timeout: Duration::from_millis(50),
    };

    let mut batcher = MockBatcher::new(batch_cfg);

    // 空批次不应该触发发送
    assert!(!batcher.should_send_batch());
    assert!(batcher.create_and_reset_batch("empty-device".to_string()).is_none());

    // 强制发送空批次也应该返回None
    assert!(batcher.force_send("empty-device".to_string()).is_none());
}

#[test]
fn test_force_send_partial_batch() {
    let batch_cfg = BatchCfg {
        size: 10,
        timeout: Duration::from_secs(3600), // 长超时
    };

    let mut batcher = MockBatcher::new(batch_cfg);

    // 添加少量数据点
    assert!(!batcher.add_point(create_test_point("point1", 1.0)));
    assert!(!batcher.add_point(create_test_point("point2", 2.0)));
    assert_eq!(batcher.get_current_batch_size(), 2);

    // 强制发送部分批次
    let batch = batcher.force_send("force-device".to_string()).unwrap();
    assert_eq!(batch.points.len(), 2);
    assert_eq!(batcher.get_current_batch_size(), 0);
    assert_eq!(batcher.get_total_sent(), 1);
}

#[test]
fn test_batch_message_structure() {
    let batch_cfg = BatchCfg {
        size: 2,
        timeout: Duration::from_secs(1),
    };

    let mut batcher = MockBatcher::new(batch_cfg);

    // 创建带有元数据的测试点
    let mut meta1 = HashMap::new();
    meta1.insert("source".to_string(), "driver1".to_string());
    meta1.insert("location".to_string(), "zone_a".to_string());

    let mut meta2 = HashMap::new();
    meta2.insert("source".to_string(), "driver2".to_string());
    meta2.insert("location".to_string(), "zone_b".to_string());

    let point1 = DataPoint {
        tag: "temperature.sensor1".to_string(),
        value: Value::Number(serde_json::Number::from_f64(23.5).unwrap()),
        quality: 2,
        meta: meta1,
    };

    let point2 = DataPoint {
        tag: "humidity.sensor2".to_string(),
        value: Value::Number(serde_json::Number::from_f64(65.2).unwrap()),
        quality: 2,
        meta: meta2,
    };

    batcher.add_point(point1);
    assert!(batcher.add_point(point2)); // 应该触发发送

    let batch = batcher.create_and_reset_batch("structured-device".to_string()).unwrap();

    // 验证批次结构
    assert_eq!(batch.device_id, "structured-device");
    assert!(batch.timestamp > 0);
    assert_eq!(batch.points.len(), 2);

    // 验证数据点
    assert_eq!(batch.points[0].tag, "temperature.sensor1");
    assert_eq!(batch.points[1].tag, "humidity.sensor2");

    // 验证元数据
    assert_eq!(batch.points[0].meta.get("source"), Some(&"driver1".to_string()));
    assert_eq!(batch.points[1].meta.get("location"), Some(&"zone_b".to_string()));
}

#[test]
fn test_batch_serialization_size() {
    let batch_cfg = BatchCfg {
        size: 100,
        timeout: Duration::from_secs(1),
    };

    let mut batcher = MockBatcher::new(batch_cfg);

    // 创建大批次
    for i in 0..100 {
        let point = DataPoint {
            tag: format!("sensor.{:03}", i),
            value: Value::Number(serde_json::Number::from_f64(i as f64 * 0.1).unwrap()),
            quality: 2,
            meta: {
                let mut meta = HashMap::new();
                meta.insert("id".to_string(), i.to_string());
                meta.insert("type".to_string(), "temperature".to_string());
                meta
            },
        };
        
        if i == 99 {
            assert!(batcher.add_point(point)); // 最后一个应该触发
        } else {
            assert!(!batcher.add_point(point));
        }
    }

    let batch = batcher.create_and_reset_batch("large-batch-device".to_string()).unwrap();
    assert_eq!(batch.points.len(), 100);

    // 序列化批次并检查大小
    let json_size = serde_json::to_vec(&batch).unwrap().len();
    assert!(json_size > 1000); // 大批次应该有相当的大小

    // 测试压缩前后的大小差异
    let json_data = serde_json::to_vec(&batch).unwrap();
    let compressed_data = zstd::encode_all(&json_data[..], 3).unwrap();
    let compression_ratio = compressed_data.len() as f64 / json_data.len() as f64;
    
    assert!(compression_ratio < 1.0); // 压缩应该减小大小
    assert!(compression_ratio > 0.05); // 允许更好的压缩效果（调整为5%）
}

#[test]
fn test_batch_timing_precision() {
    let batch_cfg = BatchCfg {
        size: 100, // 大批次，确保由时间触发
        timeout: Duration::from_millis(100),
    };

    let mut batcher = MockBatcher::new(batch_cfg);

    let start_time = Instant::now();
    batcher.add_point(create_test_point("timing_test", 1.0));

    // 在超时之前检查
    std::thread::sleep(Duration::from_millis(50));
    assert!(!batcher.should_send_batch());

    // 在超时之后检查
    std::thread::sleep(Duration::from_millis(60)); // 总共110ms
    assert!(batcher.should_send_batch());

    let elapsed = start_time.elapsed();
    assert!(elapsed >= Duration::from_millis(100));
    assert!(elapsed < Duration::from_millis(200)); // 应该相对及时
}

#[test]
fn test_concurrent_batch_access_simulation() {
    let batch_cfg = BatchCfg {
        size: 5,
        timeout: Duration::from_millis(100),
    };

    let mut batcher = MockBatcher::new(batch_cfg);

    // 模拟快速连续添加（模拟并发场景）
    let mut should_send_count = 0;
    
    for i in 0..20 {
        let point = create_test_point(&format!("concurrent_{}", i), i as f64);
        if batcher.add_point(point) {
            should_send_count += 1;
            // 模拟发送批次
            let _batch = batcher.create_and_reset_batch(format!("device_{}", should_send_count));
        }
    }

    // 20个点，批次大小为5，应该产生4个完整批次
    assert_eq!(should_send_count, 4);
    assert_eq!(batcher.get_total_sent(), 4);
    assert_eq!(batcher.get_current_batch_size(), 0); // 应该没有剩余
}

#[test]
fn test_batch_memory_efficiency() {
    let batch_cfg = BatchCfg {
        size: 1000,
        timeout: Duration::from_secs(1),
    };

    let mut batcher = MockBatcher::new(batch_cfg);

    // 添加大量小数据点
    for i in 0..500 {
        let point = DataPoint {
            tag: format!("mem_test_{}", i),
            value: Value::Number(i.into()),
            quality: 2,
            meta: HashMap::new(), // 最小元数据
        };
        batcher.add_point(point);
    }

    assert_eq!(batcher.get_current_batch_size(), 500);

    // 测试大数据点
    let large_meta = (0..100).map(|i| {
        (format!("meta_key_{}", i), format!("meta_value_{}", i))
    }).collect();

    let large_point = DataPoint {
        tag: "large_metadata_point".to_string(),
        value: Value::String("x".repeat(1000)), // 大字符串值
        quality: 2,
        meta: large_meta,
    };

    batcher.add_point(large_point);
    assert_eq!(batcher.get_current_batch_size(), 501);

    let batch = batcher.force_send("memory-test-device".to_string()).unwrap();
    assert_eq!(batch.points.len(), 501);

    // 验证内存被正确释放（批次重置后）
    assert_eq!(batcher.get_current_batch_size(), 0);
}

/// 创建测试用的数据点
fn create_test_point(tag: &str, value: f64) -> DataPoint {
    DataPoint {
        tag: tag.to_string(),
        value: Value::Number(serde_json::Number::from_f64(value).unwrap()),
        quality: 2,
        meta: HashMap::new(),
    }
}