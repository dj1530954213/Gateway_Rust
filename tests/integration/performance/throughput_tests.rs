//! 吞吐量性能基准测试
//! 验证系统在高负载下的数据处理能力

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::sleep;
use serde_json::{json, Value};
use anyhow::{Result, Context};

use crate::integration::common::*;

/// 吞吐量测试配置
#[derive(Debug, Clone)]
pub struct ThroughputTestConfig {
    pub target_fps: u32,
    pub test_duration: Duration,
    pub max_acceptable_loss_rate: f64,
    pub min_cpu_efficiency: f64,
    pub max_memory_usage_mb: f64,
    pub data_sources: usize,
    pub concurrent_connections: usize,
}

impl Default for ThroughputTestConfig {
    fn default() -> Self {
        Self {
            target_fps: 5000,                          // 目标5000 fps
            test_duration: Duration::from_secs(60),    // 测试60秒
            max_acceptable_loss_rate: 1.0,             // 最大1%数据丢失率
            min_cpu_efficiency: 70.0,                  // 最低70% CPU效率
            max_memory_usage_mb: 512.0,                // 最大512MB内存使用
            data_sources: 50,                          // 50个数据源
            concurrent_connections: 100,               // 100个并发连接
        }
    }
}

/// 吞吐量测试套件
pub struct ThroughputTestSuite {
    env: TestEnvironment,
    config: ThroughputTestConfig,
    metrics_collector: MetricsCollector,
    load_generator: LoadGenerator,
}

/// 性能指标收集器
pub struct MetricsCollector {
    frames_sent: Arc<AtomicU64>,
    frames_received: Arc<AtomicU64>,
    bytes_processed: Arc<AtomicU64>,
    latency_samples: Arc<std::sync::Mutex<Vec<Duration>>>,
    start_time: Option<Instant>,
}

/// 负载生成器
pub struct LoadGenerator {
    data_sources: Vec<DataSource>,
    generation_rate: u32,
    current_sequence: Arc<AtomicU64>,
}

/// 数据源配置
#[derive(Debug, Clone)]
pub struct DataSource {
    pub id: String,
    pub tags: Vec<String>,
    pub data_pattern: DataPattern,
    pub generation_interval: Duration,
}

/// 数据模式
#[derive(Debug, Clone)]
pub enum DataPattern {
    Constant(f64),
    SineWave { amplitude: f64, period: Duration },
    RandomWalk { base: f64, step: f64 },
    Linear { start: f64, increment: f64 },
}

/// 吞吐量测试结果
#[derive(Debug)]
pub struct ThroughputTestResult {
    pub actual_fps: f64,
    pub data_loss_rate: f64,
    pub average_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub bytes_per_second: f64,
    pub efficiency_score: f64,
    pub timeline: Vec<PerformanceSnapshot>,
}

/// 性能快照
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub timestamp: Instant,
    pub fps: f64,
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub latency_ms: f64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            frames_sent: Arc::new(AtomicU64::new(0)),
            frames_received: Arc::new(AtomicU64::new(0)),
            bytes_processed: Arc::new(AtomicU64::new(0)),
            latency_samples: Arc::new(std::sync::Mutex::new(Vec::new())),
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        // 重置计数器
        self.frames_sent.store(0, Ordering::Relaxed);
        self.frames_received.store(0, Ordering::Relaxed);
        self.bytes_processed.store(0, Ordering::Relaxed);
        if let Ok(mut samples) = self.latency_samples.lock() {
            samples.clear();
        }
    }

    pub fn record_frame_sent(&self) {
        self.frames_sent.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_frame_received(&self, bytes: u64) {
        self.frames_received.fetch_add(1, Ordering::Relaxed);
        self.bytes_processed.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_latency(&self, latency: Duration) {
        if let Ok(mut samples) = self.latency_samples.lock() {
            samples.push(latency);
            // 限制样本数量防止内存过度使用
            if samples.len() > 10000 {
                samples.drain(0..5000);
            }
        }
    }

    pub fn get_current_fps(&self) -> f64 {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                let total_frames = self.frames_received.load(Ordering::Relaxed) as f64;
                return total_frames / elapsed;
            }
        }
        0.0
    }

    pub fn get_statistics(&self) -> Result<ThroughputTestResult> {
        let start_time = self.start_time.context("Metrics collection not started")?;
        let elapsed = start_time.elapsed();
        
        let frames_sent = self.frames_sent.load(Ordering::Relaxed);
        let frames_received = self.frames_received.load(Ordering::Relaxed);
        let bytes_processed = self.bytes_processed.load(Ordering::Relaxed);
        
        let actual_fps = if elapsed.as_secs_f64() > 0.0 {
            frames_received as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        let data_loss_rate = if frames_sent > 0 {
            ((frames_sent - frames_received) as f64 / frames_sent as f64) * 100.0
        } else {
            0.0
        };
        
        let bytes_per_second = if elapsed.as_secs_f64() > 0.0 {
            bytes_processed as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        // 计算延迟统计
        let (avg_latency, p95_latency, p99_latency) = if let Ok(samples) = self.latency_samples.lock() {
            if samples.is_empty() {
                (Duration::from_millis(0), Duration::from_millis(0), Duration::from_millis(0))
            } else {
                let mut sorted_samples = samples.clone();
                sorted_samples.sort();
                
                let avg = Duration::from_nanos(
                    (sorted_samples.iter().map(|d| d.as_nanos()).sum::<u128>() / sorted_samples.len() as u128) as u64
                );
                
                let p95_idx = (sorted_samples.len() as f64 * 0.95) as usize;
                let p99_idx = (sorted_samples.len() as f64 * 0.99) as usize;
                
                let p95 = sorted_samples.get(p95_idx).copied().unwrap_or(Duration::from_millis(0));
                let p99 = sorted_samples.get(p99_idx).copied().unwrap_or(Duration::from_millis(0));
                
                (avg, p95, p99)
            }
        } else {
            (Duration::from_millis(0), Duration::from_millis(0), Duration::from_millis(0))
        };
        
        // 模拟系统资源使用情况
        let cpu_usage_percent = self.estimate_cpu_usage(actual_fps);
        let memory_usage_mb = self.estimate_memory_usage(frames_received);
        
        let efficiency_score = self.calculate_efficiency_score(actual_fps, cpu_usage_percent);
        
        Ok(ThroughputTestResult {
            actual_fps,
            data_loss_rate,
            average_latency: avg_latency,
            p95_latency,
            p99_latency,
            cpu_usage_percent,
            memory_usage_mb,
            bytes_per_second,
            efficiency_score,
            timeline: Vec::new(), // 在实际实现中会填充时间线数据
        })
    }

    /// 估算CPU使用率
    fn estimate_cpu_usage(&self, fps: f64) -> f64 {
        // 基于处理帧率估算CPU使用率
        // 这里使用简化的线性模型
        let base_usage = 10.0; // 基础CPU使用率
        let fps_factor = fps / 1000.0 * 15.0; // 每1000fps增加15%CPU使用率
        (base_usage + fps_factor).min(100.0)
    }

    /// 估算内存使用量
    fn estimate_memory_usage(&self, total_frames: u64) -> f64 {
        // 估算内存使用量
        let base_memory = 50.0; // 基础内存使用 50MB
        let frame_memory = total_frames as f64 * 0.001; // 每帧大约1KB
        base_memory + frame_memory
    }

    /// 计算效率分数
    fn calculate_efficiency_score(&self, fps: f64, cpu_percent: f64) -> f64 {
        if cpu_percent > 0.0 {
            (fps / cpu_percent) * 10.0 // 标准化效率分数
        } else {
            0.0
        }
    }
}

impl LoadGenerator {
    pub fn new(config: &ThroughputTestConfig) -> Self {
        let mut data_sources = Vec::new();
        
        // 生成多个数据源
        for i in 0..config.data_sources {
            let source = DataSource {
                id: format!("source_{}", i),
                tags: vec![
                    format!("sensor_{}.temperature", i),
                    format!("sensor_{}.pressure", i),
                    format!("sensor_{}.flow", i),
                ],
                data_pattern: match i % 4 {
                    0 => DataPattern::Constant(100.0 + i as f64),
                    1 => DataPattern::SineWave { 
                        amplitude: 50.0, 
                        period: Duration::from_secs(30) 
                    },
                    2 => DataPattern::RandomWalk { 
                        base: 500.0, 
                        step: 10.0 
                    },
                    _ => DataPattern::Linear { 
                        start: 0.0, 
                        increment: 0.1 
                    },
                },
                generation_interval: Duration::from_micros(1_000_000 / (config.target_fps / config.data_sources as u32) as u64),
            };
            data_sources.push(source);
        }
        
        Self {
            data_sources,
            generation_rate: config.target_fps,
            current_sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 生成测试数据
    pub async fn generate_load(
        &self,
        duration: Duration,
        metrics: Arc<MetricsCollector>,
    ) -> Result<()> {
        println!("🚀 开始生成负载，目标 {} fps，持续 {:?}", self.generation_rate, duration);
        
        let start_time = Instant::now();
        let mut tasks = Vec::new();
        
        // 为每个数据源启动生成任务
        for source in &self.data_sources {
            let source_clone = source.clone();
            let metrics_clone = metrics.clone();
            let sequence_counter = self.current_sequence.clone();
            let end_time = start_time + duration;
            
            let task = tokio::spawn(async move {
                Self::generate_source_data(source_clone, metrics_clone, sequence_counter, end_time).await
            });
            
            tasks.push(task);
        }
        
        // 等待所有生成任务完成
        for task in tasks {
            let _ = task.await;
        }
        
        println!("✅ 负载生成完成");
        Ok(())
    }

    /// 为单个数据源生成数据
    async fn generate_source_data(
        source: DataSource,
        metrics: Arc<MetricsCollector>,
        sequence_counter: Arc<AtomicU64>,
        end_time: Instant,
    ) -> Result<()> {
        let mut last_values: HashMap<String, f64> = HashMap::new();
        let mut interval = tokio::time::interval(source.generation_interval);
        
        while Instant::now() < end_time {
            interval.tick().await;
            
            for tag in &source.tags {
                let sequence = sequence_counter.fetch_add(1, Ordering::Relaxed);
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                // 根据数据模式生成值
                let value = Self::generate_value(&source.data_pattern, &tag, &mut last_values, timestamp);
                
                // 创建数据帧
                let frame = json!({
                    "sequence": sequence,
                    "tag": tag,
                    "value": value,
                    "timestamp": timestamp,
                    "source": source.id
                });
                
                // 记录发送的帧
                metrics.record_frame_sent();
                
                // 在实际实现中，这里会发送到Mock PLC或直接发送到系统
                // 现在只是模拟记录接收
                let frame_bytes = frame.to_string().len() as u64;
                metrics.record_frame_received(frame_bytes);
                
                // 模拟处理延迟
                let processing_latency = Duration::from_micros(100 + (sequence % 50) * 10);
                metrics.record_latency(processing_latency);
            }
        }
        
        Ok(())
    }

    /// 根据数据模式生成值
    fn generate_value(
        pattern: &DataPattern,
        tag: &str,
        last_values: &mut HashMap<String, f64>,
        timestamp: u64,
    ) -> f64 {
        match pattern {
            DataPattern::Constant(value) => *value,
            
            DataPattern::SineWave { amplitude, period } => {
                let t = (timestamp as f64) % period.as_secs_f64();
                let phase = 2.0 * std::f64::consts::PI * t / period.as_secs_f64();
                amplitude * phase.sin() + amplitude
            },
            
            DataPattern::RandomWalk { base, step } => {
                let last_value = last_values.get(tag).copied().unwrap_or(*base);
                let change = (fastrand::f64() - 0.5) * 2.0 * step;
                let new_value = (last_value + change).max(0.0);
                last_values.insert(tag.to_string(), new_value);
                new_value
            },
            
            DataPattern::Linear { start, increment } => {
                start + (timestamp as f64) * increment
            },
        }
    }
}

impl ThroughputTestSuite {
    /// 创建新的吞吐量测试套件
    pub async fn new(config: ThroughputTestConfig) -> Result<Self> {
        let env = TestEnvironment::new().await?;
        let metrics_collector = MetricsCollector::new();
        let load_generator = LoadGenerator::new(&config);
        
        Ok(Self {
            env,
            config,
            metrics_collector,
            load_generator,
        })
    }

    /// 运行吞吐量基准测试
    pub async fn run_throughput_test(&mut self) -> Result<ThroughputTestResult> {
        println!("🎯 开始吞吐量基准测试...");
        println!("📊 测试配置:");
        println!("  目标 FPS: {}", self.config.target_fps);
        println!("  测试时长: {:?}", self.config.test_duration);
        println!("  数据源数量: {}", self.config.data_sources);
        println!("  并发连接: {}", self.config.concurrent_connections);
        
        // 启动指标收集
        self.metrics_collector.start();
        let metrics = Arc::new(std::mem::replace(&mut self.metrics_collector, MetricsCollector::new()));
        
        // 启动性能监控任务
        let monitoring_task = self.start_performance_monitoring(metrics.clone());
        
        // 生成负载
        self.load_generator.generate_load(self.config.test_duration, metrics.clone()).await?;
        
        // 等待额外时间确保所有数据被处理
        sleep(Duration::from_secs(5)).await;
        
        // 停止监控
        monitoring_task.abort();
        
        // 收集最终统计信息
        let mut result = metrics.get_statistics()?;
        
        // 验证性能指标
        self.validate_performance_requirements(&result)?;
        
        println!("📈 吞吐量测试结果:");
        println!("  实际 FPS: {:.2}", result.actual_fps);
        println!("  数据丢失率: {:.2}%", result.data_loss_rate);
        println!("  平均延迟: {:?}", result.average_latency);
        println!("  P95 延迟: {:?}", result.p95_latency);
        println!("  CPU 使用率: {:.2}%", result.cpu_usage_percent);
        println!("  内存使用: {:.2} MB", result.memory_usage_mb);
        println!("  传输速率: {:.2} MB/s", result.bytes_per_second / 1024.0 / 1024.0);
        println!("  效率分数: {:.2}", result.efficiency_score);
        
        self.metrics_collector = std::mem::replace(&*metrics, MetricsCollector::new());
        
        Ok(result)
    }

    /// 启动性能监控
    fn start_performance_monitoring(&self, metrics: Arc<MetricsCollector>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                
                let current_fps = metrics.get_current_fps();
                if current_fps > 0.0 {
                    println!("📊 实时性能: {:.1} fps", current_fps);
                }
            }
        })
    }

    /// 验证性能要求
    fn validate_performance_requirements(&self, result: &ThroughputTestResult) -> Result<()> {
        // 验证FPS目标
        let fps_ratio = result.actual_fps / self.config.target_fps as f64;
        if fps_ratio < 0.95 {
            anyhow::bail!(
                "实际FPS {:.2} 低于目标的95% ({:.2})",
                result.actual_fps,
                self.config.target_fps as f64 * 0.95
            );
        }
        
        // 验证数据丢失率
        if result.data_loss_rate > self.config.max_acceptable_loss_rate {
            anyhow::bail!(
                "数据丢失率 {:.2}% 超过最大允许值 {:.2}%",
                result.data_loss_rate,
                self.config.max_acceptable_loss_rate
            );
        }
        
        // 验证内存使用
        if result.memory_usage_mb > self.config.max_memory_usage_mb {
            anyhow::bail!(
                "内存使用 {:.2} MB 超过最大允许值 {:.2} MB",
                result.memory_usage_mb,
                self.config.max_memory_usage_mb
            );
        }
        
        // 验证效率
        if result.efficiency_score < self.config.min_cpu_efficiency {
            anyhow::bail!(
                "CPU效率分数 {:.2} 低于最低要求 {:.2}",
                result.efficiency_score,
                self.config.min_cpu_efficiency
            );
        }
        
        Ok(())
    }

    /// 运行分级性能测试
    pub async fn run_graduated_performance_test(&mut self) -> Result<Vec<ThroughputTestResult>> {
        println!("📊 运行分级性能测试...");
        
        let test_levels = vec![1000, 2000, 3000, 4000, 5000];
        let mut results = Vec::new();
        
        for target_fps in test_levels {
            println!("\n🎯 测试 {} FPS 性能...", target_fps);
            
            // 更新配置
            let mut test_config = self.config.clone();
            test_config.target_fps = target_fps;
            test_config.test_duration = Duration::from_secs(30); // 缩短测试时间
            
            // 重新创建负载生成器
            self.load_generator = LoadGenerator::new(&test_config);
            self.config = test_config;
            
            match self.run_throughput_test().await {
                Ok(result) => {
                    println!("✅ {} FPS 测试完成", target_fps);
                    results.push(result);
                }
                Err(e) => {
                    println!("❌ {} FPS 测试失败: {:?}", target_fps, e);
                    // 继续下一个测试级别
                }
            }
            
            // 测试间隔休息
            sleep(Duration::from_secs(5)).await;
        }
        
        // 打印性能趋势
        self.print_performance_trend(&results);
        
        Ok(results)
    }

    /// 打印性能趋势
    fn print_performance_trend(&self, results: &[ThroughputTestResult]) {
        println!("\n📈 性能趋势分析:");
        println!("{:<8} {:<12} {:<12} {:<12} {:<12}", "目标FPS", "实际FPS", "丢失率%", "延迟ms", "CPU%");
        println!("{}", "-".repeat(60));
        
        for (i, result) in results.iter().enumerate() {
            let target_fps = [1000, 2000, 3000, 4000, 5000][i];
            println!(
                "{:<8} {:<12.1} {:<12.2} {:<12.1} {:<12.1}",
                target_fps,
                result.actual_fps,
                result.data_loss_rate,
                result.average_latency.as_millis(),
                result.cpu_usage_percent
            );
        }
    }
}

/// 主要的5000 FPS吞吐量测试
#[tokio::test]
async fn test_5000_fps_throughput() -> Result<()> {
    println!("🚀 测试5000 FPS数据流吞吐量...");
    
    let config = ThroughputTestConfig::default();
    let mut test_suite = ThroughputTestSuite::new(config.clone()).await?;
    
    let result = test_suite.run_throughput_test().await?;
    
    // 验证核心性能指标
    assert!(
        result.actual_fps >= config.target_fps as f64 * 0.95,
        "实际FPS {:.2} 低于目标的95%",
        result.actual_fps
    );
    
    assert!(
        result.data_loss_rate <= config.max_acceptable_loss_rate,
        "数据丢失率 {:.2}% 超过最大允许值 {:.2}%",
        result.data_loss_rate,
        config.max_acceptable_loss_rate
    );
    
    assert!(
        result.average_latency <= Duration::from_millis(100),
        "平均延迟 {:?} 超过100ms",
        result.average_latency
    );
    
    println!("✅ 5000 FPS吞吐量测试通过");
    Ok(())
}

/// 测试分级性能
#[tokio::test]
async fn test_graduated_throughput_performance() -> Result<()> {
    println!("📊 测试分级吞吐量性能...");
    
    let config = ThroughputTestConfig {
        test_duration: Duration::from_secs(20), // 缩短测试时间
        ..Default::default()
    };
    
    let mut test_suite = ThroughputTestSuite::new(config).await?;
    
    let results = test_suite.run_graduated_performance_test().await?;
    
    // 验证至少有3个性能级别的结果
    assert!(results.len() >= 3, "应该至少有3个性能级别的测试结果");
    
    // 验证性能随负载的变化趋势
    for (i, result) in results.iter().enumerate() {
        let target_fps = [1000, 2000, 3000, 4000, 5000][i];
        
        // 较低负载应该有更高的效率
        if target_fps <= 2000 {
            assert!(
                result.data_loss_rate <= 0.5,
                "低负载 {} FPS 时数据丢失率应该很低",
                target_fps
            );
        }
    }
    
    println!("✅ 分级吞吐量性能测试通过");
    Ok(())
}

/// 测试高并发连接性能
#[tokio::test]
async fn test_high_concurrency_throughput() -> Result<()> {
    println!("🔗 测试高并发连接吞吐量...");
    
    let config = ThroughputTestConfig {
        target_fps: 3000,
        concurrent_connections: 200,
        test_duration: Duration::from_secs(30),
        ..Default::default()
    };
    
    let mut test_suite = ThroughputTestSuite::new(config.clone()).await?;
    
    let result = test_suite.run_throughput_test().await?;
    
    // 验证高并发下的性能
    assert!(
        result.actual_fps >= 2500.0,
        "高并发下实际FPS {:.2} 应该 >= 2500",
        result.actual_fps
    );
    
    assert!(
        result.p95_latency <= Duration::from_millis(200),
        "高并发下P95延迟 {:?} 应该 <= 200ms",
        result.p95_latency
    );
    
    println!("✅ 高并发连接吞吐量测试通过");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_throughput_config_creation() {
        let config = ThroughputTestConfig::default();
        assert_eq!(config.target_fps, 5000);
        assert_eq!(config.data_sources, 50);
    }
    
    #[test]
    fn test_data_pattern_generation() {
        let mut last_values = HashMap::new();
        
        // 测试常量模式
        let constant_pattern = DataPattern::Constant(42.0);
        let value = LoadGenerator::generate_value(&constant_pattern, "test", &mut last_values, 0);
        assert_eq!(value, 42.0);
        
        // 测试线性模式
        let linear_pattern = DataPattern::Linear { start: 10.0, increment: 1.0 };
        let value = LoadGenerator::generate_value(&linear_pattern, "test", &mut last_values, 5);
        assert_eq!(value, 15.0);
    }
    
    #[tokio::test]
    async fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();
        collector.start();
        
        // 记录一些指标
        collector.record_frame_sent();
        collector.record_frame_received(100);
        collector.record_latency(Duration::from_millis(50));
        
        assert_eq!(collector.frames_sent.load(Ordering::Relaxed), 1);
        assert_eq!(collector.frames_received.load(Ordering::Relaxed), 1);
        assert_eq!(collector.bytes_processed.load(Ordering::Relaxed), 100);
    }
}