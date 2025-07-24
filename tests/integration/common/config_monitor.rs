//! 配置监控工具
//! 用于监控配置文件变化和系统响应

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::fs;
use tokio::time::sleep;
use anyhow::{Result, Context};
use serde_json::Value;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use tokio::sync::mpsc;

/// 配置变更事件
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    pub file_path: PathBuf,
    pub change_type: ConfigChangeType,
    pub timestamp: Instant,
    pub content_hash: String,
}

/// 配置变更类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// 系统响应事件
#[derive(Debug, Clone)]
pub struct SystemResponseEvent {
    pub event_type: SystemEventType,
    pub timestamp: Instant,
    pub details: HashMap<String, String>,
}

/// 系统事件类型
#[derive(Debug, Clone, PartialEq)]
pub enum SystemEventType {
    ConfigReloadStarted,
    ConfigReloadCompleted,
    ConfigValidationFailed,
    DriverRestarted,
    ConnectorReconnected,
    ErrorOccurred,
}

/// 配置监控器
pub struct ConfigMonitor {
    config_dir: PathBuf,
    change_events: Vec<ConfigChangeEvent>,
    system_events: Vec<SystemResponseEvent>,
    file_watcher: Option<Box<dyn Watcher + Send>>,
    event_receiver: Option<mpsc::Receiver<ConfigChangeEvent>>,
}

impl ConfigMonitor {
    /// 创建新的配置监控器
    pub fn new<P: AsRef<Path>>(config_dir: P) -> Self {
        Self {
            config_dir: config_dir.as_ref().to_path_buf(),
            change_events: Vec::new(),
            system_events: Vec::new(),
            file_watcher: None,
            event_receiver: None,
        }
    }

    /// 开始监控配置文件变化
    pub async fn start_monitoring(&mut self) -> Result<()> {
        println!("🔍 开始监控配置目录: {:?}", self.config_dir);

        let (tx, rx) = mpsc::channel(100);
        
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    let change_type = match event.kind {
                        EventKind::Create(_) => ConfigChangeType::Created,
                        EventKind::Modify(_) => ConfigChangeType::Modified,
                        EventKind::Remove(_) => ConfigChangeType::Deleted,
                        _ => ConfigChangeType::Modified,
                    };

                    for path in event.paths {
                        if let Some(extension) = path.extension() {
                            if extension == "yml" || extension == "yaml" {
                                let content_hash = Self::calculate_file_hash(&path)
                                    .unwrap_or_else(|_| "error".to_string());
                                
                                let change_event = ConfigChangeEvent {
                                    file_path: path,
                                    change_type: change_type.clone(),
                                    timestamp: Instant::now(),
                                    content_hash,
                                };

                                let _ = tx.try_send(change_event);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("配置文件监控错误: {:?}", e);
                }
            }
        })?;

        watcher.watch(&self.config_dir, RecursiveMode::Recursive)?;
        
        self.file_watcher = Some(Box::new(watcher));
        self.event_receiver = Some(rx);

        println!("✅ 配置文件监控启动成功");
        Ok(())
    }

    /// 停止监控
    pub fn stop_monitoring(&mut self) {
        self.file_watcher = None;
        self.event_receiver = None;
        println!("🛑 配置文件监控已停止");
    }

    /// 收集配置变更事件
    pub async fn collect_change_events(&mut self, duration: Duration) -> Result<Vec<ConfigChangeEvent>> {
        println!("📊 收集配置变更事件，持续 {:?}...", duration);

        let start_time = Instant::now();
        let mut collected_events = Vec::new();

        while start_time.elapsed() < duration {
            if let Some(ref mut receiver) = self.event_receiver {
                match tokio::time::timeout(Duration::from_millis(100), receiver.recv()).await {
                    Ok(Some(event)) => {
                        println!("📝 检测到配置变更: {:?} - {:?}", 
                                event.file_path.file_name().unwrap_or_default(), 
                                event.change_type);
                        collected_events.push(event.clone());
                        self.change_events.push(event);
                    }
                    Ok(None) => break, // 通道关闭
                    Err(_) => {
                        // 超时，继续循环
                        sleep(Duration::from_millis(50)).await;
                    }
                }
            } else {
                sleep(Duration::from_millis(100)).await;
            }
        }

        println!("✅ 配置变更事件收集完成，共 {} 个事件", collected_events.len());
        Ok(collected_events)
    }

    /// 模拟系统响应监控
    pub async fn monitor_system_response(&mut self, duration: Duration) -> Result<Vec<SystemResponseEvent>> {
        println!("🖥️  监控系统响应，持续 {:?}...", duration);

        let start_time = Instant::now();
        let mut system_events = Vec::new();

        // 这里模拟系统响应事件的监控
        // 在实际实现中，这些事件应该通过日志、指标或API来收集
        
        while start_time.elapsed() < duration {
            // 模拟配置重载检测
            if start_time.elapsed() > Duration::from_secs(2) && 
               start_time.elapsed() < Duration::from_secs(3) {
                system_events.push(SystemResponseEvent {
                    event_type: SystemEventType::ConfigReloadStarted,
                    timestamp: Instant::now(),
                    details: [("component".to_string(), "driver-manager".to_string())]
                        .iter().cloned().collect(),
                });
            }

            if start_time.elapsed() > Duration::from_secs(5) && 
               start_time.elapsed() < Duration::from_secs(6) {
                system_events.push(SystemResponseEvent {
                    event_type: SystemEventType::ConfigReloadCompleted,
                    timestamp: Instant::now(),
                    details: [
                        ("component".to_string(), "driver-manager".to_string()),
                        ("reload_time_ms".to_string(), "3000".to_string()),
                    ].iter().cloned().collect(),
                });
            }

            sleep(Duration::from_millis(100)).await;
        }

        self.system_events.extend(system_events.clone());
        println!("✅ 系统响应监控完成，共 {} 个事件", system_events.len());
        Ok(system_events)
    }

    /// 分析配置热重载性能
    pub fn analyze_hotreload_performance(&self) -> HotReloadAnalysis {
        println!("📈 分析配置热重载性能...");

        let mut reload_times = Vec::new();
        let mut validation_failures = 0;
        let mut component_restarts = 0;

        // 分析配置变更和系统响应的关联性
        for change_event in &self.change_events {
            // 寻找对应的系统响应
            let response_start = self.system_events.iter()
                .find(|e| e.event_type == SystemEventType::ConfigReloadStarted &&
                         e.timestamp >= change_event.timestamp &&
                         e.timestamp.duration_since(change_event.timestamp) < Duration::from_secs(10));

            let response_end = self.system_events.iter()
                .find(|e| e.event_type == SystemEventType::ConfigReloadCompleted &&
                         e.timestamp >= change_event.timestamp &&
                         e.timestamp.duration_since(change_event.timestamp) < Duration::from_secs(30));

            if let (Some(start), Some(end)) = (response_start, response_end) {
                let reload_time = end.timestamp.duration_since(start.timestamp);
                reload_times.push(reload_time);
            }
        }

        // 统计错误事件
        for event in &self.system_events {
            match event.event_type {
                SystemEventType::ConfigValidationFailed => validation_failures += 1,
                SystemEventType::DriverRestarted => component_restarts += 1,
                _ => {}
            }
        }

        let average_reload_time = if !reload_times.is_empty() {
            reload_times.iter().sum::<Duration>() / reload_times.len() as u32
        } else {
            Duration::from_secs(0)
        };

        let max_reload_time = reload_times.iter().max().copied()
            .unwrap_or(Duration::from_secs(0));

        println!("📊 热重载性能分析结果:");
        println!("  配置变更次数: {}", self.change_events.len());
        println!("  成功重载次数: {}", reload_times.len());
        println!("  平均重载时间: {:?}", average_reload_time);
        println!("  最大重载时间: {:?}", max_reload_time);
        println!("  验证失败次数: {}", validation_failures);
        println!("  组件重启次数: {}", component_restarts);

        HotReloadAnalysis {
            total_config_changes: self.change_events.len(),
            successful_reloads: reload_times.len(),
            failed_reloads: self.change_events.len() - reload_times.len(),
            average_reload_time,
            max_reload_time,
            validation_failures,
            component_restarts,
            reload_times,
        }
    }

    /// 验证配置变更响应时间
    pub fn verify_response_time(&self, max_allowed_time: Duration) -> Vec<ResponseTimeViolation> {
        let mut violations = Vec::new();

        for change_event in &self.change_events {
            let response_time = self.system_events.iter()
                .filter(|e| e.timestamp >= change_event.timestamp)
                .min_by_key(|e| e.timestamp)
                .map(|e| e.timestamp.duration_since(change_event.timestamp));

            if let Some(time) = response_time {
                if time > max_allowed_time {
                    violations.push(ResponseTimeViolation {
                        config_file: change_event.file_path.clone(),
                        change_time: change_event.timestamp,
                        response_time: time,
                        max_allowed: max_allowed_time,
                    });
                }
            }
        }

        violations
    }

    /// 计算文件内容哈希
    fn calculate_file_hash(path: &Path) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let content = std::fs::read_to_string(path)
            .context("Failed to read file content")?;
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        
        Ok(format!("{:x}", hasher.finish()))
    }

    /// 获取所有收集的事件
    pub fn get_all_events(&self) -> (&[ConfigChangeEvent], &[SystemResponseEvent]) {
        (&self.change_events, &self.system_events)
    }
}

/// 热重载分析结果
#[derive(Debug)]
pub struct HotReloadAnalysis {
    pub total_config_changes: usize,
    pub successful_reloads: usize,
    pub failed_reloads: usize,
    pub average_reload_time: Duration,
    pub max_reload_time: Duration,
    pub validation_failures: usize,
    pub component_restarts: usize,
    pub reload_times: Vec<Duration>,
}

/// 响应时间违规
#[derive(Debug)]
pub struct ResponseTimeViolation {
    pub config_file: PathBuf,
    pub change_time: Instant,
    pub response_time: Duration,
    pub max_allowed: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_monitor_creation() {
        let temp_dir = TempDir::new().unwrap();
        let monitor = ConfigMonitor::new(temp_dir.path());
        
        assert_eq!(monitor.config_dir, temp_dir.path());
        assert_eq!(monitor.change_events.len(), 0);
    }

    #[tokio::test]
    async fn test_file_hash_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.yml");
        
        tokio::fs::write(&test_file, "test: value").await.unwrap();
        
        let hash1 = ConfigMonitor::calculate_file_hash(&test_file).unwrap();
        let hash2 = ConfigMonitor::calculate_file_hash(&test_file).unwrap();
        
        assert_eq!(hash1, hash2);
        
        // 修改文件内容
        tokio::fs::write(&test_file, "test: different_value").await.unwrap();
        let hash3 = ConfigMonitor::calculate_file_hash(&test_file).unwrap();
        
        assert_ne!(hash1, hash3);
    }
}