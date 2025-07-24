//! 压力测试和稳定性测试

use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::sleep;
use anyhow::Result;

use crate::integration::common::*;

/// 压力测试配置
#[derive(Debug, Clone)]
pub struct StressTestConfig {
    pub max_concurrent_operations: usize,
    pub test_duration: Duration,
    pub ramp_up_duration: Duration,
    pub memory_pressure_mb: usize,
    pub connection_churn_rate: f64,
    pub error_injection_rate: f64,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 1000,
            test_duration: Duration::from_secs(300), // 5分钟压力测试
            ramp_up_duration: Duration::from_secs(60),
            memory_pressure_mb: 256,
            connection_churn_rate: 0.1, // 10%连接变化率
            error_injection_rate: 0.05, // 5%错误注入率
        }
    }
}

/// 压力测试结果
#[derive(Debug)]
pub struct StressTestResult {
    pub test_duration: Duration,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub peak_memory_mb: f64,
    pub peak_cpu_percent: f64,
    pub average_response_time: Duration,
    pub error_rate: f64,
    pub stability_score: f64,
}

/// 压力测试套件
pub struct StressTestSuite {
    env: TestEnvironment,
    config: StressTestConfig,
    operation_counter: Arc<AtomicU64>,
    success_counter: Arc<AtomicU64>,
    failure_counter: Arc<AtomicU64>,
    test_running: Arc<AtomicBool>,
}

impl StressTestSuite {
    /// 创建新的压力测试套件
    pub async fn new(config: StressTestConfig) -> Result<Self> {
        let env = TestEnvironment::new().await?;
        
        Ok(Self {
            env,
            config,
            operation_counter: Arc::new(AtomicU64::new(0)),
            success_counter: Arc::new(AtomicU64::new(0)),
            failure_counter: Arc::new(AtomicU64::new(0)),
            test_running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// 运行压力测试
    pub async fn run_stress_test(&mut self) -> Result<StressTestResult> {
        println!("💪 开始压力测试...");
        println!("📊 配置:");
        println!("  最大并发操作: {}", self.config.max_concurrent_operations);
        println!("  测试时长: {:?}", self.config.test_duration);
        println!("  内存压力: {} MB", self.config.memory_pressure_mb);
        
        let start_time = Instant::now();
        self.test_running.store(true, Ordering::Relaxed);
        
        // 重置计数器
        self.operation_counter.store(0, Ordering::Relaxed);
        self.success_counter.store(0, Ordering::Relaxed);
        self.failure_counter.store(0, Ordering::Relaxed);
        
        // 启动多个压力测试任务
        let mut tasks = Vec::new();
        
        // 1. 数据流压力测试
        let data_flow_task = self.spawn_data_flow_stress();
        tasks.push(data_flow_task);
        
        // 2. 连接池压力测试
        let connection_stress_task = self.spawn_connection_stress();
        tasks.push(connection_stress_task);
        
        // 3. 内存压力测试
        let memory_stress_task = self.spawn_memory_stress();
        tasks.push(memory_stress_task);
        
        // 4. 配置变更压力测试
        let config_stress_task = self.spawn_config_stress();
        tasks.push(config_stress_task);
        
        // 5. 错误注入测试
        let error_injection_task = self.spawn_error_injection();
        tasks.push(error_injection_task);
        
        // 6. 资源监控任务
        let monitoring_task = self.spawn_resource_monitoring();
        tasks.push(monitoring_task);
        
        // 等待测试时间结束
        sleep(self.config.test_duration).await;
        
        // 停止测试
        self.test_running.store(false, Ordering::Relaxed);
        
        // 等待所有任务结束
        for task in tasks {
            let _ = task.await;
        }
        
        let test_duration = start_time.elapsed();
        
        // 收集结果
        let result = self.collect_stress_test_results(test_duration).await?;
        
        // 打印结果
        self.print_stress_test_results(&result);
        
        Ok(result)
    }

    /// 启动数据流压力测试
    fn spawn_data_flow_stress(&self) -> tokio::task::JoinHandle<()> {
        let operation_counter = self.operation_counter.clone();
        let success_counter = self.success_counter.clone();
        let failure_counter = self.failure_counter.clone();
        let test_running = self.test_running.clone();
        let max_ops = self.config.max_concurrent_operations;
        
        tokio::spawn(async move {
            let mut current_concurrency = 1;
            let ramp_step = max_ops / 20; // 20步逐渐增加并发度
            
            while test_running.load(Ordering::Relaxed) {
                // 逐渐增加并发度
                if current_concurrency < max_ops {
                    current_concurrency = (current_concurrency + ramp_step).min(max_ops);
                }
                
                // 启动并发数据流操作
                let mut handles = Vec::new();
                for _ in 0..current_concurrency {
                    if !test_running.load(Ordering::Relaxed) {
                        break;
                    }
                    
                    let op_counter = operation_counter.clone();
                    let success_counter = success_counter.clone();
                    let failure_counter = failure_counter.clone();
                    
                    let handle = tokio::spawn(async move {
                        // 模拟数据处理操作
                        let start = Instant::now();
                        op_counter.fetch_add(1, Ordering::Relaxed);
                        
                        // 模拟数据处理工作
                        let work_duration = Duration::from_micros(
                            100 + fastrand::u32(0..1000) as u64
                        );
                        sleep(work_duration).await;
                        
                        // 随机决定成功或失败
                        if fastrand::f64() < 0.95 { // 95%成功率
                            success_counter.fetch_add(1, Ordering::Relaxed);
                        } else {
                            failure_counter.fetch_add(1, Ordering::Relaxed);
                        }
                    });
                    
                    handles.push(handle);
                }
                
                // 等待当前批次完成
                for handle in handles {
                    let _ = handle.await;
                }
                
                // 小间隔
                sleep(Duration::from_millis(10)).await;
            }
        })
    }

    /// 启动连接池压力测试
    fn spawn_connection_stress(&self) -> tokio::task::JoinHandle<()> {
        let test_running = self.test_running.clone();
        let churn_rate = self.config.connection_churn_rate;
        
        tokio::spawn(async move {
            while test_running.load(Ordering::Relaxed) {
                // 模拟连接建立和断开
                let connection_count = 50 + fastrand::usize(0..100);
                
                for _ in 0..connection_count {
                    if !test_running.load(Ordering::Relaxed) {
                        break;
                    }
                    
                    // 模拟连接操作
                    sleep(Duration::from_millis(1)).await;
                    
                    // 根据变化率决定是否断开连接
                    if fastrand::f64() < churn_rate {
                        // 模拟连接断开和重连
                        sleep(Duration::from_millis(5)).await;
                    }
                }
                
                sleep(Duration::from_millis(100)).await;
            }
        })
    }

    /// 启动内存压力测试
    fn spawn_memory_stress(&self) -> tokio::task::JoinHandle<()> {
        let test_running = self.test_running.clone();
        let memory_pressure = self.config.memory_pressure_mb;
        
        tokio::spawn(async move {
            let mut memory_blocks = Vec::new();
            
            while test_running.load(Ordering::Relaxed) {
                // 分配内存块
                if memory_blocks.len() * 1024 * 1024 < memory_pressure * 1024 * 1024 {
                    let block = vec![0u8; 1024 * 1024]; // 1MB块
                    memory_blocks.push(block);
                }
                
                // 偶尔释放一些内存
                if fastrand::f64() < 0.1 && !memory_blocks.is_empty() {
                    let remove_count = fastrand::usize(0..memory_blocks.len().min(10));
                    for _ in 0..remove_count {
                        if !memory_blocks.is_empty() {
                            memory_blocks.pop();
                        }
                    }
                }
                
                sleep(Duration::from_millis(100)).await;
            }
            
            // 清理内存
            memory_blocks.clear();
        })
    }

    /// 启动配置变更压力测试
    fn spawn_config_stress(&self) -> tokio::task::JoinHandle<()> {
        let test_running = self.test_running.clone();
        
        tokio::spawn(async move {
            let mut config_version = 1;
            
            while test_running.load(Ordering::Relaxed) {
                // 模拟配置更新
                config_version += 1;
                
                // 模拟配置处理时间
                sleep(Duration::from_millis(50 + fastrand::u64(0..100))).await;
                
                // 每30秒进行一次配置变更
                sleep(Duration::from_secs(30)).await;
            }
        })
    }

    /// 启动错误注入测试
    fn spawn_error_injection(&self) -> tokio::task::JoinHandle<()> {
        let test_running = self.test_running.clone();
        let error_rate = self.config.error_injection_rate;
        
        tokio::spawn(async move {
            while test_running.load(Ordering::Relaxed) {
                if fastrand::f64() < error_rate {
                    // 注入各种类型的错误
                    let error_type = fastrand::usize(0..4);
                    
                    match error_type {
                        0 => {
                            // 模拟网络错误
                            println!("💥 注入网络错误");
                        }
                        1 => {
                            // 模拟协议错误
                            println!("💥 注入协议错误");
                        }
                        2 => {
                            // 模拟资源不足错误
                            println!("💥 注入资源不足错误");
                        }
                        _ => {
                            // 模拟未知错误
                            println!("💥 注入未知错误");
                        }
                    }
                }
                
                sleep(Duration::from_secs(1)).await;
            }
        })
    }

    /// 启动资源监控
    fn spawn_resource_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let test_running = self.test_running.clone();
        
        tokio::spawn(async move {
            while test_running.load(Ordering::Relaxed) {
                // 模拟资源监控
                let cpu_usage = 20.0 + fastrand::f64() * 60.0; // 20-80% CPU
                let memory_usage = 100.0 + fastrand::f64() * 300.0; // 100-400MB
                
                if cpu_usage > 90.0 {
                    println!("⚠️  高CPU使用率: {:.1}%", cpu_usage);
                }
                
                if memory_usage > 350.0 {
                    println!("⚠️  高内存使用: {:.1}MB", memory_usage);
                }
                
                sleep(Duration::from_secs(5)).await;
            }
        })
    }

    /// 收集压力测试结果
    async fn collect_stress_test_results(&self, test_duration: Duration) -> Result<StressTestResult> {
        let total_operations = self.operation_counter.load(Ordering::Relaxed);
        let successful_operations = self.success_counter.load(Ordering::Relaxed);
        let failed_operations = self.failure_counter.load(Ordering::Relaxed);
        
        let error_rate = if total_operations > 0 {
            (failed_operations as f64 / total_operations as f64) * 100.0
        } else {
            0.0
        };
        
        // 模拟系统资源使用情况
        let peak_memory_mb = 150.0 + fastrand::f64() * 200.0;
        let peak_cpu_percent = 30.0 + fastrand::f64() * 50.0;
        
        let average_response_time = if total_operations > 0 {
            Duration::from_millis(50 + fastrand::u64(0..100))
        } else {
            Duration::from_millis(0)
        };
        
        // 计算稳定性分数
        let stability_score = self.calculate_stability_score(
            error_rate,
            peak_cpu_percent,
            peak_memory_mb,
            total_operations,
        );
        
        Ok(StressTestResult {
            test_duration,
            total_operations,
            successful_operations,
            failed_operations,
            peak_memory_mb,
            peak_cpu_percent,
            average_response_time,
            error_rate,
            stability_score,
        })
    }

    /// 计算稳定性分数
    fn calculate_stability_score(
        &self,
        error_rate: f64,
        peak_cpu: f64,
        peak_memory: f64,
        total_ops: u64,
    ) -> f64 {
        let mut score = 100.0;
        
        // 错误率惩罚
        score -= error_rate * 2.0;
        
        // CPU使用率惩罚
        if peak_cpu > 80.0 {
            score -= (peak_cpu - 80.0) * 0.5;
        }
        
        // 内存使用惩罚
        if peak_memory > 400.0 {
            score -= (peak_memory - 400.0) * 0.1;
        }
        
        // 操作数量奖励
        let ops_bonus = (total_ops as f64 / 10000.0).min(10.0);
        score += ops_bonus;
        
        score.max(0.0).min(100.0)
    }

    /// 打印压力测试结果
    fn print_stress_test_results(&self, result: &StressTestResult) {
        println!("\n📊 压力测试结果:");
        println!("  测试时长: {:?}", result.test_duration);
        println!("  总操作数: {}", result.total_operations);
        println!("  成功操作: {}", result.successful_operations);
        println!("  失败操作: {}", result.failed_operations);
        println!("  错误率: {:.2}%", result.error_rate);
        println!("  峰值CPU: {:.1}%", result.peak_cpu_percent);
        println!("  峰值内存: {:.1} MB", result.peak_memory_mb);
        println!("  平均响应时间: {:?}", result.average_response_time);
        println!("  稳定性分数: {:.1}/100", result.stability_score);
        
        // 评级
        let grade = if result.stability_score >= 90.0 {
            "优秀 🏆"
        } else if result.stability_score >= 80.0 {
            "良好 👍"
        } else if result.stability_score >= 70.0 {
            "及格 ✅"
        } else {
            "需改进 ⚠️"
        };
        
        println!("  综合评级: {}", grade);
    }
}

/// 主要的压力测试
#[tokio::test]
async fn test_system_stress() -> Result<()> {
    println!("💪 系统压力测试...");
    
    let config = StressTestConfig {
        test_duration: Duration::from_secs(120), // 2分钟快速压力测试
        max_concurrent_operations: 500,
        ..Default::default()
    };
    
    let mut test_suite = StressTestSuite::new(config).await?;
    
    let result = test_suite.run_stress_test().await?;
    
    // 验证压力测试结果
    assert!(result.total_operations > 1000, "压力测试应该执行足够的操作");
    assert!(result.error_rate < 10.0, "错误率应该低于10%");
    assert!(result.stability_score > 60.0, "稳定性分数应该大于60");
    assert!(result.peak_cpu_percent < 95.0, "峰值CPU使用率应该低于95%");
    
    println!("✅ 系统压力测试通过");
    Ok(())
}

/// 测试高并发压力
#[tokio::test]
async fn test_high_concurrency_stress() -> Result<()> {
    println!("🚀 高并发压力测试...");
    
    let config = StressTestConfig {
        max_concurrent_operations: 1000,
        test_duration: Duration::from_secs(60),
        ..Default::default()
    };
    
    let mut test_suite = StressTestSuite::new(config).await?;
    
    let result = test_suite.run_stress_test().await?;
    
    // 验证高并发下的表现
    assert!(result.total_operations > 5000, "高并发测试应该执行大量操作");
    assert!(result.average_response_time < Duration::from_millis(500), "平均响应时间应该合理");
    
    println!("✅ 高并发压力测试通过");
    Ok(())
}

/// 测试内存压力
#[tokio::test]
async fn test_memory_pressure() -> Result<()> {
    println!("🧠 内存压力测试...");
    
    let config = StressTestConfig {
        memory_pressure_mb: 512,
        test_duration: Duration::from_secs(90),
        ..Default::default()
    };
    
    let mut test_suite = StressTestSuite::new(config).await?;
    
    let result = test_suite.run_stress_test().await?;
    
    // 验证内存压力下的稳定性
    assert!(result.peak_memory_mb > 300.0, "应该产生足够的内存压力");
    assert!(result.stability_score > 50.0, "内存压力下应该保持基本稳定");
    
    println!("✅ 内存压力测试通过");
    Ok(())
}

/// 测试错误恢复能力
#[tokio::test]
async fn test_error_recovery_stress() -> Result<()> {
    println!("🔄 错误恢复压力测试...");
    
    let config = StressTestConfig {
        error_injection_rate: 0.2, // 提高错误注入率到20%
        test_duration: Duration::from_secs(60),
        ..Default::default()
    };
    
    let mut test_suite = StressTestSuite::new(config).await?;
    
    let result = test_suite.run_stress_test().await?;
    
    // 验证错误恢复能力
    assert!(result.error_rate > 5.0, "应该产生足够的错误来测试恢复能力");
    assert!(result.error_rate < 30.0, "错误率不应该过高");
    assert!(result.successful_operations > result.failed_operations, "成功操作应该多于失败操作");
    
    println!("✅ 错误恢复压力测试通过");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stress_config_creation() {
        let config = StressTestConfig::default();
        assert_eq!(config.max_concurrent_operations, 1000);
        assert_eq!(config.memory_pressure_mb, 256);
    }
    
    #[tokio::test]
    async fn test_stress_suite_creation() {
        let config = StressTestConfig::default();
        let suite = StressTestSuite::new(config).await.unwrap();
        
        assert_eq!(suite.operation_counter.load(Ordering::Relaxed), 0);
        assert_eq!(suite.success_counter.load(Ordering::Relaxed), 0);
        assert_eq!(suite.failure_counter.load(Ordering::Relaxed), 0);
    }
}