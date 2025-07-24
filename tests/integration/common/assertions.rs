//! 集成测试断言工具

use std::time::{Duration, Instant};
use anyhow::{Result, Context};
use serde_json::Value;

/// 数据帧验证结果
#[derive(Debug, PartialEq)]
pub struct FrameValidation {
    pub total_frames: usize,
    pub valid_frames: usize,
    pub invalid_frames: usize,
    pub accuracy_percentage: f64,
}

impl FrameValidation {
    /// 计算准确率
    pub fn accuracy(&self) -> f64 {
        if self.total_frames == 0 {
            return 0.0;
        }
        (self.valid_frames as f64 / self.total_frames as f64) * 100.0
    }

    /// 检查是否达到最小准确率要求
    pub fn meets_accuracy_requirement(&self, min_accuracy: f64) -> bool {
        self.accuracy() >= min_accuracy
    }
}

/// 性能测试结果
#[derive(Debug)]
pub struct PerformanceMetrics {
    pub fps: f64,
    pub latency_ms: f64,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub duration: Duration,
}

/// 集成测试断言工具
pub struct IntegrationAssertions;

impl IntegrationAssertions {
    /// 等待条件满足（带超时）
    pub async fn wait_for_condition<F, Fut>(
        mut condition: F,
        timeout: Duration,
        check_interval: Duration,
    ) -> Result<()>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<bool>>,
    {
        let start = Instant::now();
        
        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("Condition not met within timeout of {:?}", timeout);
            }
            
            match condition().await {
                Ok(true) => return Ok(()),
                Ok(false) => {
                    tokio::time::sleep(check_interval).await;
                    continue;
                }
                Err(e) => {
                    return Err(e).context("Condition check failed");
                }
            }
        }
    }

    /// 验证MQTT消息数量
    pub async fn assert_mqtt_message_count(
        mqtt_url: &str,
        topic: &str,
        expected_count: usize,
        timeout: Duration,
    ) -> Result<()> {
        use paho_mqtt as mqtt;
        
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(mqtt_url)
            .client_id("test_client")
            .finalize();
        
        let mut client = mqtt::Client::new(create_opts)?;
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .finalize();
        
        client.connect(conn_opts)?;
        client.subscribe(topic, 0)?;
        
        let mut message_count = 0;
        let start = Instant::now();
        
        while start.elapsed() < timeout {
            if let Ok(Some(_msg)) = client.try_receive() {
                message_count += 1;
                if message_count >= expected_count {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        client.disconnect(None)?;
        
        if message_count < expected_count {
            anyhow::bail!(
                "Expected {} messages, but received only {}",
                expected_count,
                message_count
            );
        }
        
        Ok(())
    }

    /// 验证数据格式转换准确性
    pub fn validate_data_conversion(
        original_data: &[Value],
        converted_data: &[Value],
        tolerance: f64,
    ) -> FrameValidation {
        let total_frames = original_data.len();
        let mut valid_frames = 0;
        
        for (orig, conv) in original_data.iter().zip(converted_data.iter()) {
            if Self::values_match(orig, conv, tolerance) {
                valid_frames += 1;
            }
        }
        
        let invalid_frames = total_frames - valid_frames;
        let accuracy_percentage = if total_frames > 0 {
            (valid_frames as f64 / total_frames as f64) * 100.0
        } else {
            0.0
        };
        
        FrameValidation {
            total_frames,
            valid_frames,
            invalid_frames,
            accuracy_percentage,
        }
    }

    /// 检查两个值是否在容差范围内匹配
    fn values_match(val1: &Value, val2: &Value, tolerance: f64) -> bool {
        match (val1, val2) {
            (Value::Number(n1), Value::Number(n2)) => {
                let f1 = n1.as_f64().unwrap_or(0.0);
                let f2 = n2.as_f64().unwrap_or(0.0);
                (f1 - f2).abs() <= tolerance
            }
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            _ => val1 == val2,
        }
    }

    /// 测试系统延迟
    pub async fn measure_end_to_end_latency(
        trigger_fn: impl std::future::Future<Output = Result<()>>,
        check_fn: impl std::future::Future<Output = Result<bool>>,
        timeout: Duration,
    ) -> Result<Duration> {
        let start = Instant::now();
        
        // 触发事件
        trigger_fn.await?;
        
        // 等待结果
        Self::wait_for_condition(
            || async { check_fn.await },
            timeout,
            Duration::from_millis(10),
        ).await?;
        
        Ok(start.elapsed())
    }

    /// 验证系统在故障后能正确恢复
    pub async fn assert_fault_recovery(
        fault_injection: impl std::future::Future<Output = Result<()>>,
        recovery_check: impl std::future::Future<Output = Result<bool>>,
        max_recovery_time: Duration,
    ) -> Result<Duration> {
        let start = Instant::now();
        
        // 注入故障
        fault_injection.await?;
        
        // 等待系统恢复
        Self::wait_for_condition(
            || async { recovery_check.await },
            max_recovery_time,
            Duration::from_millis(500),
        ).await?;
        
        Ok(start.elapsed())
    }

    /// 验证配置热重载不丢失数据
    pub async fn assert_config_hotreload_no_data_loss(
        config_update: impl std::future::Future<Output = Result<()>>,
        data_integrity_check: impl std::future::Future<Output = Result<bool>>,
        reload_timeout: Duration,
    ) -> Result<()> {
        // 执行配置更新
        config_update.await?;
        
        // 等待配置重载完成并验证数据完整性
        Self::wait_for_condition(
            || async { data_integrity_check.await },
            reload_timeout,
            Duration::from_millis(100),
        ).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_frame_validation_accuracy() {
        let validation = FrameValidation {
            total_frames: 100,
            valid_frames: 95,
            invalid_frames: 5,
            accuracy_percentage: 95.0,
        };
        
        assert_eq!(validation.accuracy(), 95.0);
        assert!(validation.meets_accuracy_requirement(90.0));
        assert!(!validation.meets_accuracy_requirement(98.0));
    }

    #[test]
    fn test_data_conversion_validation() {
        let original = vec![
            json!({"value": 42.0, "tag": "temp"}),
            json!({"value": 25.5, "tag": "humidity"}),
        ];
        
        let converted = vec![
            json!({"value": 42.1, "tag": "temp"}),
            json!({"value": 25.5, "tag": "humidity"}),
        ];
        
        let validation = IntegrationAssertions::validate_data_conversion(
            &original,
            &converted,
            0.2 // 0.2的容差
        );
        
        assert_eq!(validation.total_frames, 2);
        assert_eq!(validation.valid_frames, 2);
        assert_eq!(validation.accuracy(), 100.0);
    }
}