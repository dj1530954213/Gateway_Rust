//! 数据验证工具
//! 用于验证数据在整个处理链路中的准确性

use std::collections::HashMap;
use serde_json::Value;
use anyhow::Result;

/// 数据验证器
pub struct DataValidator {
    tolerance: f64,
    strict_mode: bool,
}

impl DataValidator {
    /// 创建新的数据验证器
    pub fn new(tolerance: f64, strict_mode: bool) -> Self {
        Self {
            tolerance,
            strict_mode,
        }
    }

    /// 验证数值是否在容差范围内
    pub fn validate_numeric_value(&self, expected: f64, actual: f64) -> ValidationResult {
        if expected.is_nan() || actual.is_nan() {
            return ValidationResult {
                is_valid: false,
                error: Some("NaN value detected".to_string()),
                relative_error: f64::INFINITY,
            };
        }

        if expected.is_infinite() || actual.is_infinite() {
            return ValidationResult {
                is_valid: false,
                error: Some("Infinite value detected".to_string()),
                relative_error: f64::INFINITY,
            };
        }

        let absolute_error = (actual - expected).abs();
        let relative_error = if expected.abs() > f64::EPSILON {
            absolute_error / expected.abs()
        } else {
            absolute_error
        };

        let is_valid = if self.strict_mode {
            relative_error <= self.tolerance
        } else {
            relative_error <= self.tolerance || absolute_error <= self.tolerance
        };

        ValidationResult {
            is_valid,
            error: if is_valid {
                None
            } else {
                Some(format!(
                    "Value mismatch: expected={:.6}, actual={:.6}, error={:.6}",
                    expected, actual, relative_error
                ))
            },
            relative_error,
        }
    }

    /// 验证JSON值
    pub fn validate_json_value(&self, expected: &Value, actual: &Value) -> ValidationResult {
        match (expected, actual) {
            (Value::Number(exp), Value::Number(act)) => {
                let exp_f64 = exp.as_f64().unwrap_or(0.0);
                let act_f64 = act.as_f64().unwrap_or(0.0);
                self.validate_numeric_value(exp_f64, act_f64)
            }
            (Value::String(exp), Value::String(act)) => ValidationResult {
                is_valid: exp == act,
                error: if exp == act {
                    None
                } else {
                    Some(format!("String mismatch: expected='{}', actual='{}'", exp, act))
                },
                relative_error: if exp == act { 0.0 } else { 1.0 },
            },
            (Value::Bool(exp), Value::Bool(act)) => ValidationResult {
                is_valid: exp == act,
                error: if exp == act {
                    None
                } else {
                    Some(format!("Boolean mismatch: expected={}, actual={}", exp, act))
                },
                relative_error: if exp == act { 0.0 } else { 1.0 },
            },
            _ => ValidationResult {
                is_valid: false,
                error: Some(format!(
                    "Type mismatch: expected={:?}, actual={:?}",
                    expected, actual
                )),
                relative_error: 1.0,
            },
        }
    }

    /// 验证数据帧格式
    pub fn validate_frame_format(&self, frame: &Value) -> FrameFormatValidation {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 检查必需字段
        let required_fields = vec!["tag", "value", "timestamp"];
        for field in required_fields {
            if !frame.get(field).is_some() {
                errors.push(format!("Missing required field: {}", field));
            }
        }

        // 检查数据类型
        if let Some(value) = frame.get("value") {
            if !value.is_number() && !value.is_string() && !value.is_boolean() {
                errors.push("Value field must be number, string, or boolean".to_string());
            }
        }

        if let Some(timestamp) = frame.get("timestamp") {
            if !timestamp.is_number() {
                errors.push("Timestamp field must be a number".to_string());
            }
        }

        // 检查可选字段
        if let Some(tag) = frame.get("tag") {
            if let Some(tag_str) = tag.as_str() {
                if tag_str.is_empty() {
                    warnings.push("Tag field is empty".to_string());
                }
                if tag_str.len() > 255 {
                    warnings.push("Tag field is very long (>255 chars)".to_string());
                }
            }
        }

        // 检查单位字段
        if let Some(unit) = frame.get("unit") {
            if !unit.is_string() {
                warnings.push("Unit field should be a string".to_string());
            }
        }

        // 检查质量字段
        if let Some(quality) = frame.get("quality") {
            if !quality.is_number() {
                warnings.push("Quality field should be a number".to_string());
            }
        }

        FrameFormatValidation {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// 批量验证数据转换
    pub fn validate_data_conversion_batch(
        &self,
        expected_data: &HashMap<String, f64>,
        actual_data: &HashMap<String, Value>,
    ) -> BatchValidationResult {
        let mut valid_count = 0;
        let mut total_count = 0;
        let mut validation_errors = Vec::new();
        let mut missing_tags = Vec::new();
        let mut extra_tags = Vec::new();

        // 检查期望的数据
        for (tag, expected_value) in expected_data {
            total_count += 1;

            if let Some(actual_frame) = actual_data.get(tag) {
                if let Some(actual_value) = actual_frame.get("value") {
                    let validation = self.validate_json_value(
                        &Value::Number(serde_json::Number::from_f64(*expected_value).unwrap()),
                        actual_value,
                    );

                    if validation.is_valid {
                        valid_count += 1;
                    } else {
                        validation_errors.push(TagValidationError {
                            tag: tag.clone(),
                            error: validation.error.unwrap_or_default(),
                            relative_error: validation.relative_error,
                        });
                    }
                } else {
                    validation_errors.push(TagValidationError {
                        tag: tag.clone(),
                        error: "Missing value field in frame".to_string(),
                        relative_error: 1.0,
                    });
                }
            } else {
                missing_tags.push(tag.clone());
            }
        }

        // 检查额外的数据
        for tag in actual_data.keys() {
            if !expected_data.contains_key(tag) {
                extra_tags.push(tag.clone());
            }
        }

        let accuracy = if total_count > 0 {
            (valid_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };

        BatchValidationResult {
            total_count,
            valid_count,
            accuracy,
            validation_errors,
            missing_tags,
            extra_tags,
        }
    }

    /// 验证数据时序性
    pub fn validate_data_timing(
        &self,
        data_frames: &[Value],
        expected_interval_ms: u64,
        tolerance_ms: u64,
    ) -> TimingValidationResult {
        if data_frames.len() < 2 {
            return TimingValidationResult {
                is_valid: true,
                total_intervals: 0,
                valid_intervals: 0,
                average_interval_ms: 0.0,
                max_jitter_ms: 0.0,
                timing_errors: Vec::new(),
            };
        }

        let mut timestamps = Vec::new();
        for frame in data_frames {
            if let Some(ts) = frame.get("timestamp").and_then(|v| v.as_u64()) {
                timestamps.push(ts);
            }
        }

        timestamps.sort();

        let mut intervals = Vec::new();
        let mut timing_errors = Vec::new();

        for i in 1..timestamps.len() {
            let interval = timestamps[i] - timestamps[i - 1];
            intervals.push(interval);

            let expected = expected_interval_ms;
            let tolerance = tolerance_ms;

            if interval < expected.saturating_sub(tolerance) || interval > expected + tolerance {
                timing_errors.push(TimingError {
                    index: i,
                    expected_interval_ms: expected,
                    actual_interval_ms: interval,
                    jitter_ms: if interval > expected {
                        interval - expected
                    } else {
                        expected - interval
                    },
                });
            }
        }

        let valid_intervals = intervals.len() - timing_errors.len();
        let average_interval = intervals.iter().sum::<u64>() as f64 / intervals.len() as f64;
        let max_jitter = timing_errors
            .iter()
            .map(|e| e.jitter_ms)
            .max()
            .unwrap_or(0);

        TimingValidationResult {
            is_valid: timing_errors.is_empty(),
            total_intervals: intervals.len(),
            valid_intervals,
            average_interval_ms: average_interval,
            max_jitter_ms: max_jitter as f64,
            timing_errors,
        }
    }
}

impl Default for DataValidator {
    fn default() -> Self {
        Self::new(0.01, false) // 1%容差，非严格模式
    }
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub error: Option<String>,
    pub relative_error: f64,
}

/// 帧格式验证结果
#[derive(Debug)]
pub struct FrameFormatValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// 批量验证结果
#[derive(Debug)]
pub struct BatchValidationResult {
    pub total_count: usize,
    pub valid_count: usize,
    pub accuracy: f64,
    pub validation_errors: Vec<TagValidationError>,
    pub missing_tags: Vec<String>,
    pub extra_tags: Vec<String>,
}

/// 标签验证错误
#[derive(Debug, Clone)]
pub struct TagValidationError {
    pub tag: String,
    pub error: String,
    pub relative_error: f64,
}

/// 时序验证结果
#[derive(Debug)]
pub struct TimingValidationResult {
    pub is_valid: bool,
    pub total_intervals: usize,
    pub valid_intervals: usize,
    pub average_interval_ms: f64,
    pub max_jitter_ms: f64,
    pub timing_errors: Vec<TimingError>,
}

/// 时序错误
#[derive(Debug, Clone)]
pub struct TimingError {
    pub index: usize,
    pub expected_interval_ms: u64,
    pub actual_interval_ms: u64,
    pub jitter_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_numeric_validation() {
        let validator = DataValidator::new(0.01, false);

        // 精确匹配
        let result = validator.validate_numeric_value(100.0, 100.0);
        assert!(result.is_valid);

        // 容差范围内
        let result = validator.validate_numeric_value(100.0, 100.5);
        assert!(result.is_valid);

        // 超出容差范围
        let result = validator.validate_numeric_value(100.0, 105.0);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_frame_format_validation() {
        let validator = DataValidator::default();

        // 有效帧
        let valid_frame = json!({
            "tag": "test.temperature",
            "value": 25.5,
            "timestamp": 1642694400,
            "unit": "°C"
        });

        let result = validator.validate_frame_format(&valid_frame);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());

        // 无效帧 - 缺少必需字段
        let invalid_frame = json!({
            "tag": "test.temperature",
            "unit": "°C"
        });

        let result = validator.validate_frame_format(&invalid_frame);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_batch_validation() {
        let validator = DataValidator::default();

        let expected = [("tag1".to_string(), 100.0), ("tag2".to_string(), 200.0)]
            .iter()
            .cloned()
            .collect();

        let actual = [
            (
                "tag1".to_string(),
                json!({
                    "tag": "tag1",
                    "value": 100.1,
                    "timestamp": 1642694400
                }),
            ),
            (
                "tag2".to_string(),
                json!({
                    "tag": "tag2",
                    "value": 199.8,
                    "timestamp": 1642694401
                }),
            ),
        ]
        .iter()
        .cloned()
        .collect();

        let result = validator.validate_data_conversion_batch(&expected, &actual);
        assert_eq!(result.total_count, 2);
        assert!(result.accuracy > 95.0);
    }
}