//! protocol_mapper.rs —— 协议到驱动名称/标识映射（最小实现）
//!
//! 迁移期用于在路由/服务中统一协议与驱动映射逻辑。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    Modbus,
    OpcUa,
    Mqtt,
}

/// 协议字符串到内部枚举的解析（宽松匹配）
pub fn parse_protocol(s: &str) -> Option<Protocol> {
    let l = s.to_ascii_lowercase();
    if l.contains("modbus") { Some(Protocol::Modbus) }
    else if l.contains("opc") { Some(Protocol::OpcUa) }
    else if l.contains("mqtt") { Some(Protocol::Mqtt) }
    else { None }
}

/// 获取静态驱动注册名称（供 DriverManager.load_static_driver 使用）
pub fn static_driver_name(proto: Protocol) -> Option<&'static str> {
    match proto {
        Protocol::Modbus => Some("modbus-tcp"),
        Protocol::OpcUa => None,
        Protocol::Mqtt => None,
    }
}

/// 供外部快速使用的一体化入口
pub struct ProtocolMapper;

impl ProtocolMapper {
    pub fn map_static_driver(protocol_str: &str) -> Option<&'static str> {
        parse_protocol(protocol_str).and_then(static_driver_name)
    }
}

/// 简易帮助函数，便于在其它模块直接调用
pub fn get_protocol_mapper() -> ProtocolMapper { ProtocolMapper }
