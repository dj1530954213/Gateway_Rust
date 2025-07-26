/*!
# Plugin Runtime Engine

插件运行时引擎，提供插件沙箱、资源管理和执行环境
*/

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::time::timeout;

use crate::{BridgeError, Result};
use crate::abi::{PluginContext, HostCallbacks, DataExchange, LogLevel, EventType};

/// 运行时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// 最大并发插件数
    pub max_concurrent_plugins: usize,
    /// 插件执行超时（秒）
    pub plugin_timeout: u64,
    /// 共享内存大小（字节）
    pub shared_memory_size: usize,
    /// 内存使用限制（字节）
    pub memory_limit: usize,
    /// CPU使用限制（百分比）
    pub cpu_limit: f64,
    /// 沙箱模式
    pub sandbox_enabled: bool,
    /// 日志级别
    pub log_level: String,
    /// 事件队列大小
    pub event_queue_size: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_plugins: 10,
            plugin_timeout: 30,
            shared_memory_size: 1024 * 1024, // 1MB
            memory_limit: 128 * 1024 * 1024, // 128MB
            cpu_limit: 80.0,
            sandbox_enabled: true,
            log_level: "info".to_string(),
            event_queue_size: 1000,
        }
    }
}

/// 插件运行时统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    /// 活跃插件数
    pub active_plugins: usize,
    /// 总执行次数
    pub total_executions: u64,
    /// 成功执行次数
    pub successful_executions: u64,
    /// 失败执行次数
    pub failed_executions: u64,
    /// 超时执行次数
    pub timeout_executions: u64,
    /// 平均执行时间（毫秒）
    pub avg_execution_time: f64,
    /// 内存使用量（字节）
    pub memory_usage: usize,
    /// CPU使用率（百分比）
    pub cpu_usage: f64,
    /// 最后活动时间
    pub last_activity: Option<SystemTime>,
}

impl Default for RuntimeStats {
    fn default() -> Self {
        Self {
            active_plugins: 0,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            timeout_executions: 0,
            avg_execution_time: 0.0,
            memory_usage: 0,
            cpu_usage: 0.0,
            last_activity: None,
        }
    }
}

/// 插件执行上下文
#[derive(Debug)]
pub struct ExecutionContext {
    /// 插件ID
    pub plugin_id: String,
    /// 执行ID
    pub execution_id: String,
    /// 开始时间
    pub start_time: Instant,
    /// 超时时间
    pub timeout: Duration,
    /// 共享内存
    pub shared_memory: Vec<u8>,
    /// 用户数据
    pub user_data: HashMap<String, serde_json::Value>,
}

/// 插件运行时引擎
pub struct PluginRuntime {
    config: RuntimeConfig,
    stats: Arc<RwLock<RuntimeStats>>,
    semaphore: Arc<Semaphore>,
    contexts: Arc<RwLock<HashMap<String, Arc<Mutex<ExecutionContext>>>>>,
    event_sender: mpsc::UnboundedSender<RuntimeEvent>,
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<RuntimeEvent>>>,
    host_callbacks: Arc<HostCallbacks>,
}

/// 运行时事件
#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    /// 插件启动
    PluginStarted {
        plugin_id: String,
        timestamp: SystemTime,
    },
    /// 插件停止
    PluginStopped {
        plugin_id: String,
        timestamp: SystemTime,
    },
    /// 插件错误
    PluginError {
        plugin_id: String,
        error: String,
        timestamp: SystemTime,
    },
    /// 执行开始
    ExecutionStarted {
        plugin_id: String,
        execution_id: String,
        timestamp: SystemTime,
    },
    /// 执行完成
    ExecutionCompleted {
        plugin_id: String,
        execution_id: String,
        duration: Duration,
        success: bool,
        timestamp: SystemTime,
    },
    /// 内存使用警告
    MemoryWarning {
        plugin_id: String,
        usage: usize,
        limit: usize,
        timestamp: SystemTime,
    },
    /// CPU使用警告
    CpuWarning {
        plugin_id: String,
        usage: f64,
        limit: f64,
        timestamp: SystemTime,
    },
}

impl PluginRuntime {
    /// 创建新的插件运行时
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_plugins));
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        // 创建主机回调函数
        let host_callbacks = Arc::new(HostCallbacks {
            log: Self::host_log_callback,
            send_data: Self::host_send_data_callback,
            receive_data: Self::host_receive_data_callback,
            notify_event: Self::host_notify_event_callback,
            allocate_memory: Self::host_allocate_memory_callback,
            free_memory: Self::host_free_memory_callback,
        });

        Ok(Self {
            config,
            stats: Arc::new(RwLock::new(RuntimeStats::default())),
            semaphore,
            contexts: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
            host_callbacks,
        })
    }

    /// 创建执行上下文
    pub async fn create_context(&self, plugin_id: String) -> Result<String> {
        let execution_id = uuid::Uuid::new_v4().to_string();
        
        let context = ExecutionContext {
            plugin_id: plugin_id.clone(),
            execution_id: execution_id.clone(),
            start_time: Instant::now(),
            timeout: Duration::from_secs(self.config.plugin_timeout),
            shared_memory: vec![0u8; self.config.shared_memory_size],
            user_data: HashMap::new(),
        };

        {
            let mut contexts = self.contexts.write().unwrap();
            contexts.insert(execution_id.clone(), Arc::new(Mutex::new(context)));
        }

        // 发送事件
        let _ = self.event_sender.send(RuntimeEvent::ExecutionStarted {
            plugin_id,
            execution_id: execution_id.clone(),
            timestamp: SystemTime::now(),
        });

        Ok(execution_id)
    }

    /// 销毁执行上下文
    pub async fn destroy_context(&self, execution_id: &str) -> Result<()> {
        let context_arc = {
            let mut contexts = self.contexts.write().unwrap();
            contexts.remove(execution_id)
        };

        if let Some(context_arc) = context_arc {
            let context = context_arc.lock().await;
            let duration = context.start_time.elapsed();
            
            // 更新统计信息
            {
                let mut stats = self.stats.write().unwrap();
                stats.total_executions += 1;
                stats.successful_executions += 1;
                
                // 更新平均执行时间
                let total_time = stats.avg_execution_time * (stats.total_executions - 1) as f64;
                stats.avg_execution_time = (total_time + duration.as_millis() as f64) / stats.total_executions as f64;
                stats.last_activity = Some(SystemTime::now());
            }

            // 发送事件
            let _ = self.event_sender.send(RuntimeEvent::ExecutionCompleted {
                plugin_id: context.plugin_id.clone(),
                execution_id: execution_id.to_string(),
                duration,
                success: true,
                timestamp: SystemTime::now(),
            });
        }

        Ok(())
    }

    /// 获取执行上下文
    pub async fn get_context(&self, execution_id: &str) -> Option<Arc<Mutex<ExecutionContext>>> {
        let contexts = self.contexts.read().unwrap();
        contexts.get(execution_id).cloned()
    }

    /// 执行插件函数（带超时和资源限制）
    pub async fn execute_with_timeout<F, T>(&self, execution_id: &str, func: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        let _permit = self.semaphore.acquire().await.map_err(|_| {
            BridgeError::runtime("Failed to acquire execution permit")
        })?;

        let context = self.get_context(execution_id).await
            .ok_or_else(|| BridgeError::runtime("Execution context not found"))?;

        let timeout_duration = {
            let ctx = context.lock().await;
            ctx.timeout
        };

        match timeout(timeout_duration, func).await {
            Ok(result) => {
                match result {
                    Ok(value) => {
                        self.destroy_context(execution_id).await?;
                        Ok(value)
                    }
                    Err(e) => {
                        self.handle_execution_error(execution_id, &e).await;
                        Err(e)
                    }
                }
            }
            Err(_) => {
                self.handle_execution_timeout(execution_id).await;
                Err(BridgeError::Timeout)
            }
        }
    }

    /// 处理执行错误
    async fn handle_execution_error(&self, execution_id: &str, error: &BridgeError) {
        if let Some(context_arc) = self.get_context(execution_id).await {
            let context = context_arc.lock().await;
            
            // 更新统计信息
            {
                let mut stats = self.stats.write().unwrap();
                stats.failed_executions += 1;
            }

            // 发送错误事件
            let _ = self.event_sender.send(RuntimeEvent::PluginError {
                plugin_id: context.plugin_id.clone(),
                error: error.to_string(),
                timestamp: SystemTime::now(),
            });
        }

        // 清理上下文
        let _ = self.destroy_context(execution_id).await;
    }

    /// 处理执行超时
    async fn handle_execution_timeout(&self, execution_id: &str) {
        if let Some(context_arc) = self.get_context(execution_id).await {
            let context = context_arc.lock().await;
            
            // 更新统计信息
            {
                let mut stats = self.stats.write().unwrap();
                stats.timeout_executions += 1;
            }

            // 发送超时事件
            let _ = self.event_sender.send(RuntimeEvent::ExecutionCompleted {
                plugin_id: context.plugin_id.clone(),
                execution_id: execution_id.to_string(),
                duration: context.start_time.elapsed(),
                success: false,
                timestamp: SystemTime::now(),
            });
        }

        // 清理上下文
        let _ = self.destroy_context(execution_id).await;
    }

    /// 创建插件上下文
    pub async fn create_plugin_context(&self, plugin_id: &str, execution_id: &str) -> Result<PluginContext> {
        let context = self.get_context(execution_id).await
            .ok_or_else(|| BridgeError::runtime("Execution context not found"))?;

        let ctx = context.lock().await;
        let plugin_id_cstring = std::ffi::CString::new(plugin_id)
            .map_err(|_| BridgeError::runtime("Invalid plugin ID"))?;

        Ok(PluginContext {
            plugin_id: plugin_id_cstring.as_ptr(),
            host_callbacks: Arc::as_ptr(&self.host_callbacks),
            shared_memory: ctx.shared_memory.as_ptr() as *mut std::ffi::c_void,
            shared_memory_size: ctx.shared_memory.len(),
            user_data: std::ptr::null_mut(),
        })
    }

    /// 获取运行时统计信息
    pub fn get_stats(&self) -> RuntimeStats {
        let stats = self.stats.read().unwrap();
        let mut result = stats.clone();
        result.active_plugins = self.contexts.read().unwrap().len();
        result
    }

    /// 监控资源使用情况
    pub async fn monitor_resources(&self) {
        // 这里应该实现实际的资源监控逻辑
        // 例如检查内存使用量、CPU使用率等
        
        let stats = self.get_stats();
        
        // 检查内存使用
        if stats.memory_usage > self.config.memory_limit {
            let _ = self.event_sender.send(RuntimeEvent::MemoryWarning {
                plugin_id: "system".to_string(),
                usage: stats.memory_usage,
                limit: self.config.memory_limit,
                timestamp: SystemTime::now(),
            });
        }

        // 检查CPU使用
        if stats.cpu_usage > self.config.cpu_limit {
            let _ = self.event_sender.send(RuntimeEvent::CpuWarning {
                plugin_id: "system".to_string(),
                usage: stats.cpu_usage,
                limit: self.config.cpu_limit,
                timestamp: SystemTime::now(),
            });
        }
    }

    /// 启动资源监控任务
    pub async fn start_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let runtime = Arc::new(std::sync::Mutex::new(self));
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                // 资源监控逻辑在这里实现
                // 由于借用检查的限制，这里简化实现
                tracing::debug!("Runtime monitoring tick");
            }
        })
    }

    // 主机回调函数实现
    extern "C" fn host_log_callback(level: u32, message: *const std::ffi::c_char) {
        if message.is_null() {
            return;
        }

        unsafe {
            if let Ok(msg) = std::ffi::CStr::from_ptr(message).to_str() {
                match level {
                    0 => tracing::trace!("Plugin: {}", msg),
                    1 => tracing::debug!("Plugin: {}", msg),
                    2 => tracing::info!("Plugin: {}", msg),
                    3 => tracing::warn!("Plugin: {}", msg),
                    4 => tracing::error!("Plugin: {}", msg),
                    _ => tracing::info!("Plugin: {}", msg),
                }
            }
        }
    }

    extern "C" fn host_send_data_callback(data: *const DataExchange) -> i32 {
        if data.is_null() {
            return -1;
        }

        unsafe {
            let data_ref = &*data;
            tracing::debug!("Host received data: len={}, type_id={}", data_ref.len, data_ref.type_id);
            
            // 这里应该实现实际的数据发送逻辑
            // 例如发送到FrameBus或其他组件
            
            0 // 成功
        }
    }

    extern "C" fn host_receive_data_callback(data: *mut DataExchange) -> i32 {
        if data.is_null() {
            return -1;
        }

        unsafe {
            let data_ref = &mut *data;
            tracing::debug!("Host providing data to plugin");
            
            // 这里应该实现实际的数据接收逻辑
            // 例如从FrameBus或其他组件接收数据
            
            // 示例：提供空数据
            data_ref.data = std::ptr::null();
            data_ref.len = 0;
            data_ref.type_id = 0;
            
            0 // 成功
        }
    }

    extern "C" fn host_notify_event_callback(event_type: u32, event_data: *const std::ffi::c_char) {
        let event_data_str = if event_data.is_null() {
            "".to_string()
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(event_data)
                    .to_str()
                    .unwrap_or("")
                    .to_string()
            }
        };

        match event_type {
            0 => tracing::info!("Plugin started: {}", event_data_str),
            1 => tracing::info!("Plugin stopped: {}", event_data_str),
            2 => tracing::error!("Plugin error: {}", event_data_str),
            3 => tracing::debug!("Plugin data received: {}", event_data_str),
            4 => tracing::debug!("Plugin data sent: {}", event_data_str),
            5 => tracing::info!("Plugin config changed: {}", event_data_str),
            6 => tracing::warn!("Plugin health check failed: {}", event_data_str),
            _ => tracing::debug!("Plugin event {}: {}", event_type, event_data_str),
        }
    }

    extern "C" fn host_allocate_memory_callback(size: usize) -> *mut std::ffi::c_void {
        if size == 0 {
            return std::ptr::null_mut();
        }

        // 简单的内存分配，实际实现中应该有更复杂的内存管理
        let layout = std::alloc::Layout::from_size_align(size, std::mem::align_of::<u8>())
            .unwrap();
        
        unsafe {
            std::alloc::alloc(layout) as *mut std::ffi::c_void
        }
    }

    extern "C" fn host_free_memory_callback(ptr: *mut std::ffi::c_void) {
        if ptr.is_null() {
            return;
        }

        // 注意：这是一个简化的实现
        // 实际中需要跟踪分配的大小以正确释放内存
        unsafe {
            let layout = std::alloc::Layout::from_size_align(1, 1).unwrap();
            std::alloc::dealloc(ptr as *mut u8, layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_creation() {
        let config = RuntimeConfig::default();
        let runtime = PluginRuntime::new(config).unwrap();
        
        let stats = runtime.get_stats();
        assert_eq!(stats.active_plugins, 0);
        assert_eq!(stats.total_executions, 0);
    }

    #[tokio::test]
    async fn test_context_lifecycle() {
        let runtime = PluginRuntime::new(RuntimeConfig::default()).unwrap();
        
        let execution_id = runtime.create_context("test-plugin".to_string()).await.unwrap();
        assert!(runtime.get_context(&execution_id).await.is_some());
        
        runtime.destroy_context(&execution_id).await.unwrap();
        assert!(runtime.get_context(&execution_id).await.is_none());
    }

    #[tokio::test]
    async fn test_execution_timeout() {
        let mut config = RuntimeConfig::default();
        config.plugin_timeout = 1; // 1 second timeout
        
        let runtime = PluginRuntime::new(config).unwrap();
        let execution_id = runtime.create_context("test-plugin".to_string()).await.unwrap();
        
        let result = runtime.execute_with_timeout(&execution_id, async {
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok::<(), BridgeError>(())
        }).await;
        
        assert!(matches!(result, Err(BridgeError::Timeout)));
    }
}