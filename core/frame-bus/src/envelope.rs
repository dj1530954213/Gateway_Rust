//! Frame数据结构和封装

use prost::Message;
use std::time::{SystemTime, UNIX_EPOCH};

// 引入手工编写的protobuf结构
pub use crate::generated::*;

/// 帧类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    Data = 0,
    Cmd = 1,
    CmdAck = 2,
}

impl From<i32> for FrameKind {
    fn from(value: i32) -> Self {
        match value {
            0 => FrameKind::Data,
            1 => FrameKind::Cmd,
            2 => FrameKind::CmdAck,
            _ => FrameKind::Data, // 默认值
        }
    }
}

impl DataFrame {
    /// 创建新的数据帧
    pub fn new<S: Into<String>>(tag: S, value: Value) -> Self {
        Self {
            tag: tag.into(),
            value: Some(value),
            timestamp: current_timestamp_ns(),
            qos: 2, // 默认good quality
            meta: std::collections::HashMap::new(),
        }
    }

    /// 设置QoS
    pub fn with_qos(mut self, qos: u32) -> Self {
        self.qos = qos;
        self
    }

    /// 添加元数据
    pub fn with_meta<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.meta.insert(key.into(), value.into());
        self
    }
}

impl CmdFrame {
    /// 创建新的命令帧
    pub fn new<S: Into<String>>(tag: S, value: Value, origin: S) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let cmd_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
            
        Self {
            tag: tag.into(),
            value: Some(value),
            origin: origin.into(),
            cmd_id,
            timestamp: current_timestamp_ns(),
            priority: 1, // 默认正常优先级
            timeout_ms: 5000, // 默认5秒超时
            meta: std::collections::HashMap::new(),
        }
    }

    /// 设置命令ID
    pub fn with_cmd_id(mut self, cmd_id: u64) -> Self {
        self.cmd_id = cmd_id;
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// 设置超时时间
    pub fn with_timeout_ms(mut self, timeout_ms: u32) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// 添加元数据
    pub fn with_meta<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.meta.insert(key.into(), value.into());
        self
    }
}

impl CmdAckFrame {
    /// 创建成功确认帧
    pub fn success<S: Into<String>>(
        cmd_id: u64,
        tag: S,
        driver_id: S,
        actual_value: Option<Value>,
    ) -> Self {
        Self {
            cmd_id,
            tag: tag.into(),
            success: true,
            error_msg: String::new(),
            timestamp: current_timestamp_ns(),
            actual_value,
            driver_id: driver_id.into(),
        }
    }

    /// 创建失败确认帧
    pub fn failure<S: Into<String>>(
        cmd_id: u64,
        tag: S,
        driver_id: S,
        error_msg: S,
    ) -> Self {
        Self {
            cmd_id,
            tag: tag.into(),
            success: false,
            error_msg: error_msg.into(),
            timestamp: current_timestamp_ns(),
            actual_value: None,
            driver_id: driver_id.into(),
        }
    }
}

impl FrameEnvelope {
    /// 包装DataFrame
    pub fn wrap_data(seq: u64, frame: DataFrame) -> Result<Self, prost::EncodeError> {
        let payload = frame.encode_to_vec();
        Ok(Self {
            seq,
            kind: FrameKind::Data as i32,
            payload,
        })
    }

    /// 包装CmdFrame
    pub fn wrap_cmd(seq: u64, frame: CmdFrame) -> Result<Self, prost::EncodeError> {
        let payload = frame.encode_to_vec();
        Ok(Self {
            seq,
            kind: FrameKind::Cmd as i32,
            payload,
        })
    }

    /// 包装CmdAckFrame
    pub fn wrap_cmd_ack(seq: u64, frame: CmdAckFrame) -> Result<Self, prost::EncodeError> {
        let payload = frame.encode_to_vec();
        Ok(Self {
            seq,
            kind: FrameKind::CmdAck as i32,
            payload,
        })
    }

    /// 解包为DataFrame
    pub fn into_data(self) -> Result<DataFrame, prost::DecodeError> {
        if self.kind != FrameKind::Data as i32 {
            return Err(prost::DecodeError::new("not a data frame"));
        }
        DataFrame::decode(&self.payload[..])
    }

    /// 解包为CmdFrame
    pub fn into_cmd(self) -> Result<CmdFrame, prost::DecodeError> {
        if self.kind != FrameKind::Cmd as i32 {
            return Err(prost::DecodeError::new("not a cmd frame"));
        }
        CmdFrame::decode(&self.payload[..])
    }

    /// 解包为CmdAckFrame
    pub fn into_cmd_ack(self) -> Result<CmdAckFrame, prost::DecodeError> {
        if self.kind != FrameKind::CmdAck as i32 {
            return Err(prost::DecodeError::new("not a cmd ack frame"));
        }
        CmdAckFrame::decode(&self.payload[..])
    }

    /// 获取帧类型
    pub fn kind(&self) -> FrameKind {
        FrameKind::from(self.kind)
    }
}

/// Value类型转换方法（构造方法在generated.rs中定义）
impl Value {

    /// 尝试转换为f64
    pub fn to_f64(&self) -> Option<f64> {
        match &self.value {
            Some(value::Value::FloatV(v)) => Some(*v),
            Some(value::Value::IntV(v)) => Some(*v as f64),
            Some(value::Value::BoolV(v)) => Some(if *v { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// 尝试转换为i64
    pub fn to_i64(&self) -> Option<i64> {
        match &self.value {
            Some(value::Value::IntV(v)) => Some(*v),
            Some(value::Value::FloatV(v)) => Some(*v as i64),
            Some(value::Value::BoolV(v)) => Some(if *v { 1 } else { 0 }),
            _ => None,
        }
    }

    /// 尝试转换为bool
    pub fn to_bool(&self) -> Option<bool> {
        match &self.value {
            Some(value::Value::BoolV(v)) => Some(*v),
            Some(value::Value::IntV(v)) => Some(*v != 0),
            Some(value::Value::FloatV(v)) => Some(*v != 0.0),
            _ => None,
        }
    }

    /// 尝试转换为字符串
    pub fn to_string(&self) -> Option<String> {
        match &self.value {
            Some(value::Value::StrV(v)) => Some(v.clone()),
            Some(value::Value::IntV(v)) => Some(v.to_string()),
            Some(value::Value::FloatV(v)) => Some(v.to_string()),
            Some(value::Value::BoolV(v)) => Some(v.to_string()),
            _ => None,
        }
    }
}

/// 获取当前时间戳（纳秒）
fn current_timestamp_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_frame_creation() {
        let frame = DataFrame::new("test.tag", Value::int(42))
            .with_qos(2)
            .with_meta("unit", "celsius");

        assert_eq!(frame.tag, "test.tag");
        assert_eq!(frame.value.unwrap().to_i64(), Some(42));
        assert_eq!(frame.qos, 2);
        assert_eq!(frame.meta.get("unit"), Some(&"celsius".to_string()));
    }

    #[test]
    fn test_envelope_roundtrip() {
        let original = DataFrame::new("test.tag", Value::float(3.14));
        let envelope = FrameEnvelope::wrap_data(123, original.clone()).unwrap();
        
        assert_eq!(envelope.seq, 123);
        assert_eq!(envelope.kind(), FrameKind::Data);
        
        let decoded = envelope.into_data().unwrap();
        assert_eq!(decoded.tag, original.tag);
        assert_eq!(decoded.value.unwrap().to_f64(), Some(3.14));
    }
}