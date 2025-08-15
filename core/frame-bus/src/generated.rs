// 手工编写的protobuf结构，用于替代prost生成的代码
// 这避免了对外部protoc编译器的依赖

use prost::Message;

/// 统一数据值类型
#[derive(Clone, PartialEq, Message)]
pub struct Value {
    #[prost(oneof = "value::Value", tags = "1, 2, 3, 4, 5")]
    pub value: ::core::option::Option<value::Value>,
}

/// Nested message and enum types in `Value`.
pub mod value {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Value {
        #[prost(bool, tag = "1")]
        BoolV(bool),
        #[prost(int64, tag = "2")]
        IntV(i64),
        #[prost(double, tag = "3")]
        FloatV(f64),
        #[prost(bytes, tag = "4")]
        BinV(::prost::alloc::vec::Vec<u8>),
        #[prost(string, tag = "5")]
        StrV(::prost::alloc::string::String),
    }
}

/// 现场数据帧
#[derive(Clone, PartialEq, Message)]
pub struct DataFrame {
    /// 全局唯一标识符 "driver_id.point"
    #[prost(string, tag = "1")]
    pub tag: ::prost::alloc::string::String,
    /// 数据值
    #[prost(message, optional, tag = "2")]
    pub value: ::core::option::Option<Value>,
    /// ns since epoch (monotonic)
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
    /// 0=bad, 1=uncertain, 2=good
    #[prost(uint32, tag = "4")]
    pub qos: u32,
    /// 元数据: unit=db, raw_reg=40001...
    #[prost(map = "string, string", tag = "5")]
    pub meta: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}

/// 命令/写入帧
#[derive(Clone, PartialEq, Message)]
pub struct CmdFrame {
    /// 目标点位
    #[prost(string, tag = "1")]
    pub tag: ::prost::alloc::string::String,
    /// 期望写值
    #[prost(message, optional, tag = "2")]
    pub value: ::core::option::Option<Value>,
    /// "north" / "bridge_opcua" / "mqtt"
    #[prost(string, tag = "3")]
    pub origin: ::prost::alloc::string::String,
    /// 命令唯一ID
    #[prost(uint64, tag = "4")]
    pub cmd_id: u64,
    /// 命令创建时间戳
    #[prost(uint64, tag = "5")]
    pub timestamp: u64,
    /// 命令优先级 (0=低, 1=正常, 2=高, 3=紧急)
    #[prost(int32, tag = "6")]
    pub priority: i32,
    /// 超时时间(毫秒)
    #[prost(uint32, tag = "7")]
    pub timeout_ms: u32,
    /// 可选元数据: ack_needed=true
    #[prost(map = "string, string", tag = "8")]
    pub meta: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}

/// 命令确认帧
#[derive(Clone, PartialEq, Message)]
pub struct CmdAckFrame {
    /// 对应的命令ID
    #[prost(uint64, tag = "1")]
    pub cmd_id: u64,
    /// 目标点位
    #[prost(string, tag = "2")]
    pub tag: ::prost::alloc::string::String,
    /// 执行是否成功
    #[prost(bool, tag = "3")]
    pub success: bool,
    /// 错误信息(如果失败)
    #[prost(string, tag = "4")]
    pub error_msg: ::prost::alloc::string::String,
    /// 确认时间戳
    #[prost(uint64, tag = "5")]
    pub timestamp: u64,
    /// 实际写入的值(可选)
    #[prost(message, optional, tag = "6")]
    pub actual_value: ::core::option::Option<Value>,
    /// 执行驱动ID
    #[prost(string, tag = "7")]
    pub driver_id: ::prost::alloc::string::String,
}

/// 帧封装（总线内部使用）
#[derive(Clone, PartialEq, Message)]
pub struct FrameEnvelope {
    /// 单调递增序列号
    #[prost(uint64, tag = "1")]
    pub seq: u64,
    /// 0=DATA, 1=CMD, 2=CMD_ACK
    #[prost(int32, tag = "2")]
    pub kind: i32,
    /// DataFrame、CmdFrame或CmdAckFrame序列化
    #[prost(bytes, tag = "3")]
    pub payload: ::prost::alloc::vec::Vec<u8>,
}

// 便利构造函数实现
impl Value {
    pub fn bool(v: bool) -> Self {
        Self {
            value: Some(value::Value::BoolV(v)),
        }
    }

    pub fn int(v: i64) -> Self {
        Self {
            value: Some(value::Value::IntV(v)),
        }
    }

    pub fn float(v: f64) -> Self {
        Self {
            value: Some(value::Value::FloatV(v)),
        }
    }

    pub fn string<S: Into<String>>(v: S) -> Self {
        Self {
            value: Some(value::Value::StrV(v.into())),
        }
    }

    pub fn bytes(v: Vec<u8>) -> Self {
        Self {
            value: Some(value::Value::BinV(v)),
        }
    }
}