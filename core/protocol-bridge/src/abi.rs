/*!
# ABI v1 Interface

ABI v1插件接口定义，提供统一的插件交互标准
*/

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

use serde::{Deserialize, Serialize};

/// ABI v1版本常量
pub const ABI_VERSION: &str = "v1";

/// 数据交换结构
#[repr(C)]
#[derive(Debug)]
pub struct DataExchange {
    /// 数据指针
    pub data: *const c_void,
    /// 数据长度
    pub len: usize,
    /// 数据类型标识
    pub type_id: u32,
}

impl DataExchange {
    /// 创建新的数据交换结构
    pub fn new<T>(data: &T, type_id: u32) -> Self {
        Self {
            data: data as *const T as *const c_void,
            len: std::mem::size_of::<T>(),
            type_id,
        }
    }

    /// 从字节创建数据交换结构
    pub fn from_bytes(bytes: &[u8], type_id: u32) -> Self {
        Self {
            data: bytes.as_ptr() as *const c_void,
            len: bytes.len(),
            type_id,
        }
    }

    /// 转换为字节切片
    pub unsafe fn as_bytes(&self) -> &[u8] {
        std::slice::from_raw_parts(self.data as *const u8, self.len)
    }
}

/// 插件上下文
#[repr(C)]
#[derive(Debug)]
pub struct PluginContext {
    /// 插件ID
    pub plugin_id: *const c_char,
    /// 主机回调函数指针
    pub host_callbacks: *const HostCallbacks,
    /// 共享内存区域
    pub shared_memory: *mut c_void,
    /// 共享内存大小
    pub shared_memory_size: usize,
    /// 用户数据
    pub user_data: *mut c_void,
}

impl PluginContext {
    /// 获取插件ID
    pub fn get_plugin_id(&self) -> Result<String, std::str::Utf8Error> {
        unsafe {
            let cstr = CStr::from_ptr(self.plugin_id);
            Ok(cstr.to_str()?.to_string())
        }
    }

    /// 获取主机回调
    pub fn get_host_callbacks(&self) -> Option<&HostCallbacks> {
        if self.host_callbacks.is_null() {
            None
        } else {
            unsafe { Some(&*self.host_callbacks) }
        }
    }

    /// 获取共享内存
    pub fn get_shared_memory(&self) -> Option<&mut [u8]> {
        if self.shared_memory.is_null() || self.shared_memory_size == 0 {
            None
        } else {
            unsafe {
                Some(std::slice::from_raw_parts_mut(
                    self.shared_memory as *mut u8,
                    self.shared_memory_size,
                ))
            }
        }
    }
}

/// 主机回调函数结构
#[repr(C)]
#[derive(Debug)]
pub struct HostCallbacks {
    /// 日志回调
    pub log: extern "C" fn(level: u32, message: *const c_char),
    /// 发送数据回调
    pub send_data: extern "C" fn(data: *const DataExchange) -> i32,
    /// 接收数据回调
    pub receive_data: extern "C" fn(data: *mut DataExchange) -> i32,
    /// 通知事件回调
    pub notify_event: extern "C" fn(event_type: u32, event_data: *const c_char),
    /// 分配内存回调
    pub allocate_memory: extern "C" fn(size: usize) -> *mut c_void,
    /// 释放内存回调
    pub free_memory: extern "C" fn(ptr: *mut c_void),
}

/// ABI v1接口trait
pub trait ABIv1: Send + Sync {
    /// 获取插件信息
    fn get_plugin_info(&self) -> HashMap<String, serde_json::Value>;

    /// 初始化插件
    fn initialize(&self, config: HashMap<String, serde_json::Value>) -> Result<(), String>;

    /// 启动插件
    fn start(&self) -> Result<(), String>;

    /// 停止插件
    fn stop(&self) -> Result<(), String>;

    /// 暂停插件
    fn pause(&self) -> Result<(), String>;

    /// 恢复插件
    fn resume(&self) -> Result<(), String>;

    /// 配置插件
    fn configure(&self, config: HashMap<String, serde_json::Value>) -> Result<(), String>;

    /// 处理数据
    fn process_data(&self, input: &DataExchange, output: &mut DataExchange) -> Result<(), String>;

    /// 获取统计信息
    fn get_stats(&self) -> HashMap<String, serde_json::Value>;

    /// 健康检查
    fn health_check(&self) -> bool;

    /// 清理资源
    fn cleanup(&self) -> Result<(), String>;

    /// 设置上下文
    fn set_context(&self, context: *const PluginContext) -> Result<(), String>;

    /// 获取支持的协议
    fn get_supported_protocols(&self) -> Vec<String>;

    /// 处理协议消息
    fn handle_protocol_message(&self, protocol: &str, message: &[u8]) -> Result<Vec<u8>, String>;
}

/// 插件入口点函数类型
pub type PluginEntryPoint = extern "C" fn() -> *mut dyn ABIv1;

/// 标准插件入口点函数名
pub const PLUGIN_ENTRY_POINT: &[u8] = b"bridge_entry\0";

/// 基础ABI实现
pub struct BaseABI {
    plugin_info: HashMap<String, serde_json::Value>,
    context: Option<usize>, // 使用usize代替原始指针以保证Send+Sync
    stats: HashMap<String, serde_json::Value>,
    initialized: bool,
    running: bool,
}

impl BaseABI {
    /// 创建新的基础ABI实现
    pub fn new() -> Self {
        let mut plugin_info = HashMap::new();
        plugin_info.insert("abi_version".to_string(), serde_json::Value::String(ABI_VERSION.to_string()));
        plugin_info.insert("name".to_string(), serde_json::Value::String("Base Plugin".to_string()));
        plugin_info.insert("version".to_string(), serde_json::Value::String("1.0.0".to_string()));
        
        Self {
            plugin_info,
            context: None,
            stats: HashMap::new(),
            initialized: false,
            running: false,
        }
    }

    /// 设置插件信息
    pub fn set_plugin_info(&mut self, key: &str, value: serde_json::Value) {
        self.plugin_info.insert(key.to_string(), value);
    }

    /// 更新统计信息
    pub fn update_stat(&mut self, key: &str, value: serde_json::Value) {
        self.stats.insert(key.to_string(), value);
    }

    /// 记录日志
    pub fn log(&self, level: LogLevel, message: &str) {
        if let Some(context) = self.context {
            if let Some(callbacks) = unsafe { (*(context as *const PluginContext)).get_host_callbacks() } {
                let c_message = CString::new(message).unwrap_or_default();
                (callbacks.log)(level as u32, c_message.as_ptr());
            }
        }
    }

    /// 发送数据到主机
    pub fn send_data_to_host(&self, data: &DataExchange) -> Result<(), String> {
        if let Some(context) = self.context {
            if let Some(callbacks) = unsafe { (*(context as *const PluginContext)).get_host_callbacks() } {
                let result = (callbacks.send_data)(data);
                if result == 0 {
                    Ok(())
                } else {
                    Err(format!("Host send_data failed with code: {}", result))
                }
            } else {
                Err("No host callbacks available".to_string())
            }
        } else {
            Err("No context available".to_string())
        }
    }

    /// 从主机接收数据
    pub fn receive_data_from_host(&self, data: &mut DataExchange) -> Result<(), String> {
        if let Some(context) = self.context {
            if let Some(callbacks) = unsafe { (*(context as *const PluginContext)).get_host_callbacks() } {
                let result = (callbacks.receive_data)(data);
                if result == 0 {
                    Ok(())
                } else {
                    Err(format!("Host receive_data failed with code: {}", result))
                }
            } else {
                Err("No host callbacks available".to_string())
            }
        } else {
            Err("No context available".to_string())
        }
    }

    /// 通知主机事件
    pub fn notify_host_event(&self, event_type: EventType, event_data: &str) {
        if let Some(context) = self.context {
            if let Some(callbacks) = unsafe { (*(context as *const PluginContext)).get_host_callbacks() } {
                let c_event_data = CString::new(event_data).unwrap_or_default();
                (callbacks.notify_event)(event_type as u32, c_event_data.as_ptr());
            }
        }
    }
}

impl ABIv1 for BaseABI {
    fn get_plugin_info(&self) -> HashMap<String, serde_json::Value> {
        self.plugin_info.clone()
    }

    fn initialize(&self, _config: HashMap<String, serde_json::Value>) -> Result<(), String> {
        self.log(LogLevel::Info, "Plugin initialized");
        Ok(())
    }

    fn start(&self) -> Result<(), String> {
        self.log(LogLevel::Info, "Plugin started");
        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        self.log(LogLevel::Info, "Plugin stopped");
        Ok(())
    }

    fn pause(&self) -> Result<(), String> {
        self.log(LogLevel::Info, "Plugin paused");
        Ok(())
    }

    fn resume(&self) -> Result<(), String> {
        self.log(LogLevel::Info, "Plugin resumed");
        Ok(())
    }

    fn configure(&self, _config: HashMap<String, serde_json::Value>) -> Result<(), String> {
        self.log(LogLevel::Info, "Plugin configured");
        Ok(())
    }

    fn process_data(&self, _input: &DataExchange, _output: &mut DataExchange) -> Result<(), String> {
        // 基础实现什么都不做
        Ok(())
    }

    fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        self.stats.clone()
    }

    fn health_check(&self) -> bool {
        true
    }

    fn cleanup(&self) -> Result<(), String> {
        self.log(LogLevel::Info, "Plugin cleaned up");
        Ok(())
    }

    fn set_context(&self, _context: *const PluginContext) -> Result<(), String> {
        // 注意：这里我们不能修改self，因为trait方法是&self
        // 在实际实现中，需要使用内部可变性（如RefCell）
        Ok(())
    }

    fn get_supported_protocols(&self) -> Vec<String> {
        vec!["base".to_string()]
    }

    fn handle_protocol_message(&self, _protocol: &str, _message: &[u8]) -> Result<Vec<u8>, String> {
        Err("Not implemented".to_string())
    }
}

/// 日志级别
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

/// 事件类型
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum EventType {
    PluginStarted = 0,
    PluginStopped = 1,
    PluginError = 2,
    DataReceived = 3,
    DataSent = 4,
    ConfigChanged = 5,
    HealthCheckFailed = 6,
}

/// 数据类型ID常量
pub mod data_types {
    pub const UNKNOWN: u32 = 0;
    pub const JSON: u32 = 1;
    pub const BINARY: u32 = 2;
    pub const PROTOBUF: u32 = 3;
    pub const MSGPACK: u32 = 4;
    pub const OPCUA_DATA: u32 = 100;
    pub const MODBUS_DATA: u32 = 101;
}

/// 辅助宏，用于简化插件入口点的定义
#[macro_export]
macro_rules! define_plugin_entry {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn bridge_entry() -> *mut dyn $crate::abi::ABIv1 {
            let plugin = Box::new(<$plugin_type>::new());
            Box::into_raw(plugin) as *mut dyn $crate::abi::ABIv1
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_exchange() {
        let data = vec![1u8, 2, 3, 4];
        let exchange = DataExchange::from_bytes(&data, data_types::BINARY);
        
        assert_eq!(exchange.len, 4);
        assert_eq!(exchange.type_id, data_types::BINARY);
        
        unsafe {
            let bytes = exchange.as_bytes();
            assert_eq!(bytes, &[1, 2, 3, 4]);
        }
    }

    #[test]
    fn test_base_abi() {
        let mut abi = BaseABI::new();
        abi.set_plugin_info("test_key", serde_json::Value::String("test_value".to_string()));
        
        let info = abi.get_plugin_info();
        assert_eq!(info.get("test_key").unwrap().as_str(), Some("test_value"));
        assert_eq!(info.get("abi_version").unwrap().as_str(), Some(ABI_VERSION));
        
        assert!(abi.health_check());
        assert!(abi.initialize(HashMap::new()).is_ok());
    }

    #[test]
    fn test_plugin_context() {
        let plugin_id = CString::new("test-plugin").unwrap();
        let context = PluginContext {
            plugin_id: plugin_id.as_ptr(),
            host_callbacks: std::ptr::null(),
            shared_memory: std::ptr::null_mut(),
            shared_memory_size: 0,
            user_data: std::ptr::null_mut(),
        };
        
        assert_eq!(context.get_plugin_id().unwrap(), "test-plugin");
        assert!(context.get_host_callbacks().is_none());
        assert!(context.get_shared_memory().is_none());
    }
}