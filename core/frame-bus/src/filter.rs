//! 订阅过滤器

use crate::{FrameEnvelope, FrameKind};
use regex::Regex;
use prost::Message;

/// 帧过滤器
#[derive(Debug, Clone)]
pub enum Filter {
    /// 匹配所有帧
    All,
    /// 按帧类型过滤
    Kind(FrameKind),
    /// 按tag前缀过滤
    TagPrefix(String),
    /// 按tag正则表达式过滤
    TagRegex(Regex),
    /// 组合过滤器（AND逻辑）
    And(Vec<Filter>),
    /// 组合过滤器（OR逻辑）
    Or(Vec<Filter>),
}

impl Filter {
    /// 检查帧是否匹配过滤器
    pub fn matches(&self, envelope: &FrameEnvelope) -> bool {
        match self {
            Filter::All => true,
            Filter::Kind(kind) => envelope.kind() == *kind,
            Filter::TagPrefix(prefix) => {
                // 需要解包获取tag
                match envelope.kind() {
                    FrameKind::Data => {
                        if let Ok(frame) = crate::DataFrame::decode(&envelope.payload[..]) {
                            frame.tag.starts_with(prefix)
                        } else {
                            false
                        }
                    }
                    FrameKind::Cmd => {
                        if let Ok(frame) = crate::CmdFrame::decode(&envelope.payload[..]) {
                            frame.tag.starts_with(prefix)
                        } else {
                            false
                        }
                    }
                    FrameKind::CmdAck => {
                        if let Ok(frame) = crate::CmdAckFrame::decode(&envelope.payload[..]) {
                            frame.tag.starts_with(prefix)
                        } else {
                            false
                        }
                    }
                }
            }
            Filter::TagRegex(regex) => {
                // 需要解包获取tag
                match envelope.kind() {
                    FrameKind::Data => {
                        if let Ok(frame) = crate::DataFrame::decode(&envelope.payload[..]) {
                            regex.is_match(&frame.tag)
                        } else {
                            false
                        }
                    }
                    FrameKind::Cmd => {
                        if let Ok(frame) = crate::CmdFrame::decode(&envelope.payload[..]) {
                            regex.is_match(&frame.tag)
                        } else {
                            false
                        }
                    }
                    FrameKind::CmdAck => {
                        if let Ok(frame) = crate::CmdAckFrame::decode(&envelope.payload[..]) {
                            regex.is_match(&frame.tag)
                        } else {
                            false
                        }
                    }
                }
            }
            Filter::And(filters) => {
                filters.iter().all(|f| f.matches(envelope))
            }
            Filter::Or(filters) => {
                filters.iter().any(|f| f.matches(envelope))
            }
        }
    }

    /// 便捷构造方法
    pub fn data_only() -> Self {
        Filter::Kind(FrameKind::Data)
    }

    pub fn cmd_only() -> Self {
        Filter::Kind(FrameKind::Cmd)
    }

    pub fn cmd_ack_only() -> Self {
        Filter::Kind(FrameKind::CmdAck)
    }

    pub fn tag_starts_with<S: Into<String>>(prefix: S) -> Self {
        Filter::TagPrefix(prefix.into())
    }

    pub fn tag_matches(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Filter::TagRegex(Regex::new(pattern)?))
    }
}

impl Default for Filter {
    fn default() -> Self {
        Filter::All
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataFrame, Value, FrameEnvelope};

    #[test]
    fn test_kind_filter() {
        let frame = DataFrame::new("test.tag", Value::int(42));
        let envelope = FrameEnvelope::wrap_data(1, frame).unwrap();

        assert!(Filter::data_only().matches(&envelope));
        assert!(!Filter::cmd_only().matches(&envelope));
    }

    #[test]
    fn test_prefix_filter() {
        let frame = DataFrame::new("plant.temp.sensor1", Value::float(25.5));
        let envelope = FrameEnvelope::wrap_data(1, frame).unwrap();

        assert!(Filter::tag_starts_with("plant.").matches(&envelope));
        assert!(Filter::tag_starts_with("plant.temp").matches(&envelope));
        assert!(!Filter::tag_starts_with("device.").matches(&envelope));
    }

    #[test]
    fn test_regex_filter() {
        let frame = DataFrame::new("sensor_temp_01", Value::float(25.5));
        let envelope = FrameEnvelope::wrap_data(1, frame).unwrap();

        let filter = Filter::tag_matches(r"sensor_.*_\d+").unwrap();
        assert!(filter.matches(&envelope));

        let filter2 = Filter::tag_matches(r"device_.*").unwrap();
        assert!(!filter2.matches(&envelope));
    }

    #[test]
    fn test_and_filter() {
        let frame = DataFrame::new("plant.temp.sensor1", Value::float(25.5));
        let envelope = FrameEnvelope::wrap_data(1, frame).unwrap();

        let filter = Filter::And(vec![
            Filter::data_only(),
            Filter::tag_starts_with("plant."),
        ]);

        assert!(filter.matches(&envelope));
    }
}