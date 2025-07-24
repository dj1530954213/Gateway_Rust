//! WAL持久化在真实故障中的恢复测试

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::time::sleep;
use tokio::fs;
use serde_json::{json, Value};
use anyhow::{Result, Context};

use crate::integration::common::*;

/// WAL恢复测试配置
#[derive(Debug, Clone)]
pub struct WalRecoveryTestConfig {
    pub wal_directory: PathBuf,
    pub test_data_count: usize,
    pub corruption_test_enabled: bool,
    pub max_recovery_time: Duration,
    pub expected_recovery_rate: f64,
    pub data_integrity_threshold: f64,
}

impl Default for WalRecoveryTestConfig {
    fn default() -> Self {
        Self {
            wal_directory: PathBuf::from("/tmp/gateway_test_wal"),
            test_data_count: 1000,
            corruption_test_enabled: true,
            max_recovery_time: Duration::from_secs(60),
            expected_recovery_rate: 99.9,
            data_integrity_threshold: 99.5,
        }
    }
}

/// WAL恢复测试套件
pub struct WalRecoveryTestSuite {
    env: TestEnvironment,
    config: WalRecoveryTestConfig,
    test_data: Vec<TestDataFrame>,
    recovery_results: Vec<RecoveryResult>,
}

/// 测试数据帧
#[derive(Debug, Clone)]
pub struct TestDataFrame {
    pub sequence_id: u64,
    pub tag: String,
    pub value: f64,
    pub timestamp: u64,
    pub checksum: u32,
}

/// 恢复结果
#[derive(Debug)]
pub struct RecoveryResult {
    pub scenario: String,
    pub total_frames: usize,
    pub recovered_frames: usize,
    pub corrupted_frames: usize,
    pub recovery_time: Duration,
    pub data_integrity_score: f64,
}

/// WAL测试场景
#[derive(Debug, Clone)]
pub enum WalTestScenario {
    CleanShutdown,        // 正常关闭
    PowerFailure,         // 断电故障
    DiskFull,            // 磁盘空间不足
    FileCorruption,      // 文件损坏
    PartialWrite,        // 部分写入
    ConcurrentAccess,    // 并发访问冲突
}

impl WalRecoveryTestSuite {
    /// 创建新的WAL恢复测试套件
    pub async fn new(config: WalRecoveryTestConfig) -> Result<Self> {
        let env = TestEnvironment::new().await?;
        
        // 确保WAL目录存在
        fs::create_dir_all(&config.wal_directory).await?;
        
        Ok(Self {
            env,
            config,
            test_data: Vec::new(),
            recovery_results: Vec::new(),
        })
    }

    /// 生成测试数据
    fn generate_test_data(&mut self) {
        println!("📝 生成测试数据...");
        
        self.test_data.clear();
        
        for i in 0..self.config.test_data_count {
            let frame = TestDataFrame {
                sequence_id: i as u64,
                tag: format!("test.sensor_{}", i % 10),
                value: (i as f64) * 0.1 + (i % 100) as f64,
                timestamp: 1640995200 + i as u64, // 固定基准时间戳
                checksum: self.calculate_checksum(i as u64, &format!("test.sensor_{}", i % 10), (i as f64) * 0.1),
            };
            
            self.test_data.push(frame);
        }
        
        println!("✅ 生成 {} 个测试数据帧", self.test_data.len());
    }

    /// 计算校验和
    fn calculate_checksum(&self, seq: u64, tag: &str, value: f64) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        seq.hash(&mut hasher);
        tag.hash(&mut hasher);
        value.to_bits().hash(&mut hasher);
        
        hasher.finish() as u32
    }

    /// 模拟WAL写入
    async fn simulate_wal_write(&self, scenario: WalTestScenario) -> Result<()> {
        println!("💾 模拟WAL写入 - 场景: {:?}", scenario);
        
        let wal_file = self.config.wal_directory.join("test_wal.log");
        
        match scenario {
            WalTestScenario::CleanShutdown => {
                // 正常写入所有数据
                self.write_wal_data(&wal_file, &self.test_data).await?;
                self.write_wal_checkpoint(&wal_file).await?;
            }
            
            WalTestScenario::PowerFailure => {
                // 写入部分数据，然后突然停止（模拟断电）
                let partial_data = &self.test_data[..self.test_data.len() * 3 / 4];
                self.write_wal_data(&wal_file, partial_data).await?;
                // 不写入checkpoint，模拟突然断电
            }
            
            WalTestScenario::DiskFull => {
                // 写入一半数据后模拟磁盘满
                let half_data = &self.test_data[..self.test_data.len() / 2];
                self.write_wal_data(&wal_file, half_data).await?;
                // 创建一个标记文件表示磁盘满
                fs::write(self.config.wal_directory.join("disk_full.marker"), "disk full").await?;
            }
            
            WalTestScenario::FileCorruption => {
                // 写入所有数据
                self.write_wal_data(&wal_file, &self.test_data).await?;
                // 故意损坏文件的一部分
                self.corrupt_wal_file(&wal_file).await?;
            }
            
            WalTestScenario::PartialWrite => {
                // 模拟部分写入（写入过程中被中断）
                self.write_partial_wal_data(&wal_file, &self.test_data).await?;
            }
            
            WalTestScenario::ConcurrentAccess => {
                // 模拟并发写入冲突
                self.write_concurrent_wal_data(&wal_file, &self.test_data).await?;
            }
        }
        
        println!("✅ WAL写入模拟完成");
        Ok(())
    }

    /// 写入WAL数据
    async fn write_wal_data(&self, wal_file: &PathBuf, data: &[TestDataFrame]) -> Result<()> {
        let mut content = String::new();
        
        // WAL文件头
        content.push_str("# WAL File - Test Data\n");
        content.push_str(&format!("# Created: {}\n", chrono::Utc::now().to_rfc3339()));
        content.push_str(&format!("# Frame Count: {}\n", data.len()));
        content.push_str("# Format: seq,tag,value,timestamp,checksum\n");
        
        // 写入数据帧
        for frame in data {
            content.push_str(&format!(
                "{},{},{},{},{}\n",
                frame.sequence_id,
                frame.tag,
                frame.value,
                frame.timestamp,
                frame.checksum
            ));
        }
        
        fs::write(wal_file, content).await?;
        Ok(())
    }

    /// 写入WAL检查点
    async fn write_wal_checkpoint(&self, wal_file: &PathBuf) -> Result<()> {
        let checkpoint_file = wal_file.with_extension("checkpoint");
        let checkpoint_data = json!({
            "last_sequence": self.test_data.last().map(|f| f.sequence_id).unwrap_or(0),
            "frame_count": self.test_data.len(),
            "checksum": self.calculate_file_checksum(wal_file).await?,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        fs::write(checkpoint_file, serde_json::to_string_pretty(&checkpoint_data)?).await?;
        Ok(())
    }

    /// 损坏WAL文件
    async fn corrupt_wal_file(&self, wal_file: &PathBuf) -> Result<()> {
        let mut content = fs::read(wal_file).await?;
        
        // 在文件中间位置注入错误数据
        if content.len() > 100 {
            let corruption_start = content.len() / 2;
            let corruption_end = (content.len() / 2) + 50;
            
            for i in corruption_start..corruption_end.min(content.len()) {
                content[i] = 0xFF; // 注入无效字节
            }
        }
        
        fs::write(wal_file, content).await?;
        Ok(())
    }

    /// 写入部分WAL数据（模拟中断）
    async fn write_partial_wal_data(&self, wal_file: &PathBuf, data: &[TestDataFrame]) -> Result<()> {
        let mut content = String::new();
        
        // WAL文件头
        content.push_str("# WAL File - Partial Write Test\n");
        
        // 写入大部分数据，最后一条记录不完整
        for (i, frame) in data.iter().enumerate() {
            if i == data.len() - 1 {
                // 最后一条记录只写一半
                content.push_str(&format!("{},{}", frame.sequence_id, frame.tag));
                break;
            } else {
                content.push_str(&format!(
                    "{},{},{},{},{}\n",
                    frame.sequence_id,
                    frame.tag,
                    frame.value,
                    frame.timestamp,
                    frame.checksum
                ));
            }
        }
        
        fs::write(wal_file, content).await?;
        Ok(())
    }

    /// 模拟并发写入
    async fn write_concurrent_wal_data(&self, wal_file: &PathBuf, data: &[TestDataFrame]) -> Result<()> {
        // 模拟两个进程同时写入（通过创建多个临时文件）
        let temp_file1 = wal_file.with_extension("tmp1");
        let temp_file2 = wal_file.with_extension("tmp2");
        
        let half_point = data.len() / 2;
        let data1 = &data[..half_point];
        let data2 = &data[half_point..];
        
        // 并发写入
        let write1 = self.write_wal_data(&temp_file1, data1);
        let write2 = self.write_wal_data(&temp_file2, data2);
        
        tokio::try_join!(write1, write2)?;
        
        // 合并文件（可能导致数据交错）
        let mut merged_content = fs::read_to_string(&temp_file1).await?;
        let content2 = fs::read_to_string(&temp_file2).await?;
        merged_content.push_str(&content2);
        
        fs::write(wal_file, merged_content).await?;
        
        // 清理临时文件
        let _ = fs::remove_file(&temp_file1).await;
        let _ = fs::remove_file(&temp_file2).await;
        
        Ok(())
    }

    /// 计算文件校验和
    async fn calculate_file_checksum(&self, file_path: &PathBuf) -> Result<String> {
        let content = fs::read(file_path).await?;
        
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        
        Ok(format!("{:x}", hasher.finish()))
    }

    /// 执行WAL恢复
    async fn perform_wal_recovery(&self, scenario: WalTestScenario) -> Result<RecoveryResult> {
        println!("🔄 执行WAL恢复 - 场景: {:?}", scenario);
        
        let start_time = Instant::now();
        let wal_file = self.config.wal_directory.join("test_wal.log");
        
        let mut recovered_frames = Vec::new();
        let mut corrupted_count = 0;
        
        // 检查文件是否存在
        if !wal_file.exists() {
            return Ok(RecoveryResult {
                scenario: format!("{:?}", scenario),
                total_frames: self.test_data.len(),
                recovered_frames: 0,
                corrupted_frames: 0,
                recovery_time: start_time.elapsed(),
                data_integrity_score: 0.0,
            });
        }
        
        // 读取WAL文件内容
        match fs::read_to_string(&wal_file).await {
            Ok(content) => {
                for (line_no, line) in content.lines().enumerate() {
                    if line.starts_with('#') || line.trim().is_empty() {
                        continue; // 跳过注释和空行
                    }
                    
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 5 {
                        // 尝试解析数据帧
                        match self.parse_wal_line(&parts) {
                            Ok(frame) => {
                                // 验证校验和
                                let expected_checksum = self.calculate_checksum(
                                    frame.sequence_id,
                                    &frame.tag,
                                    frame.value
                                );
                                
                                if frame.checksum == expected_checksum {
                                    recovered_frames.push(frame);
                                } else {
                                    corrupted_count += 1;
                                    println!("⚠️  行 {} 校验和不匹配", line_no + 1);
                                }
                            }
                            Err(_) => {
                                corrupted_count += 1;
                                println!("⚠️  行 {} 解析失败: {}", line_no + 1, line);
                            }
                        }
                    } else if parts.len() > 0 {
                        // 部分数据行（可能是不完整的写入）
                        corrupted_count += 1;
                        println!("⚠️  行 {} 数据不完整: {}", line_no + 1, line);
                    }
                }
            }
            Err(e) => {
                println!("❌ 读取WAL文件失败: {:?}", e);
                return Ok(RecoveryResult {
                    scenario: format!("{:?}", scenario),
                    total_frames: self.test_data.len(),
                    recovered_frames: 0,
                    corrupted_frames: self.test_data.len(),
                    recovery_time: start_time.elapsed(),
                    data_integrity_score: 0.0,
                });
            }
        }
        
        let recovery_time = start_time.elapsed();
        let recovery_rate = (recovered_frames.len() as f64 / self.test_data.len() as f64) * 100.0;
        
        println!("📊 WAL恢复结果:");
        println!("  总数据帧: {}", self.test_data.len());
        println!("  恢复数据帧: {}", recovered_frames.len());
        println!("  损坏数据帧: {}", corrupted_count);
        println!("  恢复率: {:.2}%", recovery_rate);
        println!("  恢复时间: {:?}", recovery_time);
        
        Ok(RecoveryResult {
            scenario: format!("{:?}", scenario),
            total_frames: self.test_data.len(),
            recovered_frames: recovered_frames.len(),
            corrupted_frames: corrupted_count,
            recovery_time,
            data_integrity_score: recovery_rate,
        })
    }

    /// 解析WAL行
    fn parse_wal_line(&self, parts: &[&str]) -> Result<TestDataFrame> {
        if parts.len() < 5 {
            anyhow::bail!("WAL行格式不正确");
        }
        
        let frame = TestDataFrame {
            sequence_id: parts[0].parse()?,
            tag: parts[1].to_string(),
            value: parts[2].parse()?,
            timestamp: parts[3].parse()?,
            checksum: parts[4].parse()?,
        };
        
        Ok(frame)
    }

    /// 运行WAL恢复测试
    pub async fn run_wal_recovery_test(&mut self, scenario: WalTestScenario) -> Result<RecoveryResult> {
        println!("🚀 开始WAL恢复测试 - 场景: {:?}", scenario);
        
        // 清理之前的测试数据
        self.cleanup_wal_directory().await?;
        
        // 生成测试数据
        self.generate_test_data();
        
        // 模拟WAL写入
        self.simulate_wal_write(scenario.clone()).await?;
        
        // 执行恢复
        let result = self.perform_wal_recovery(scenario).await?;
        
        // 保存结果
        self.recovery_results.push(result.clone());
        
        println!("✅ WAL恢复测试完成");
        Ok(result)
    }

    /// 清理WAL目录
    async fn cleanup_wal_directory(&self) -> Result<()> {
        if self.config.wal_directory.exists() {
            let mut entries = fs::read_dir(&self.config.wal_directory).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    let _ = fs::remove_file(path).await;
                }
            }
        }
        
        Ok(())
    }

    /// 获取所有恢复结果
    pub fn get_recovery_results(&self) -> &[RecoveryResult] {
        &self.recovery_results
    }

    /// 运行所有WAL恢复场景
    pub async fn run_all_scenarios(&mut self) -> Result<Vec<RecoveryResult>> {
        let scenarios = vec![
            WalTestScenario::CleanShutdown,
            WalTestScenario::PowerFailure,
            WalTestScenario::DiskFull,
            WalTestScenario::FileCorruption,
            WalTestScenario::PartialWrite,
            WalTestScenario::ConcurrentAccess,
        ];
        
        let mut results = Vec::new();
        
        for scenario in scenarios {
            let result = self.run_wal_recovery_test(scenario).await?;
            results.push(result);
            
            // 等待一段时间再运行下一个测试
            sleep(Duration::from_secs(2)).await;
        }
        
        Ok(results)
    }
}

/// 测试正常关闭时的WAL恢复
#[tokio::test]
async fn test_wal_recovery_clean_shutdown() -> Result<()> {
    println!("✅ 测试正常关闭WAL恢复...");
    
    let config = WalRecoveryTestConfig::default();
    let mut test_suite = WalRecoveryTestSuite::new(config.clone()).await?;
    
    let result = test_suite.run_wal_recovery_test(WalTestScenario::CleanShutdown).await?;
    
    // 验证恢复率
    assert!(
        result.data_integrity_score >= config.expected_recovery_rate,
        "WAL恢复率 {:.2}% 低于期望值 {:.2}%",
        result.data_integrity_score,
        config.expected_recovery_rate
    );
    
    // 验证恢复时间
    assert!(
        result.recovery_time <= config.max_recovery_time,
        "WAL恢复时间 {:?} 超过最大允许时间 {:?}",
        result.recovery_time,
        config.max_recovery_time
    );
    
    // 验证数据完整性
    assert_eq!(result.corrupted_frames, 0, "正常关闭不应该有损坏的数据帧");
    
    println!("✅ 正常关闭WAL恢复测试通过");
    Ok(())
}

/// 测试断电故障时的WAL恢复
#[tokio::test]
async fn test_wal_recovery_power_failure() -> Result<()> {
    println!("⚡ 测试断电故障WAL恢复...");
    
    let config = WalRecoveryTestConfig {
        expected_recovery_rate: 75.0, // 降低期望恢复率，因为断电可能导致数据丢失
        ..Default::default()
    };
    
    let mut test_suite = WalRecoveryTestSuite::new(config.clone()).await?;
    
    let result = test_suite.run_wal_recovery_test(WalTestScenario::PowerFailure).await?;
    
    // 验证恢复率（断电场景下期望较低）
    assert!(
        result.data_integrity_score >= config.expected_recovery_rate,
        "断电WAL恢复率 {:.2}% 低于期望值 {:.2}%",
        result.data_integrity_score,
        config.expected_recovery_rate
    );
    
    // 应该有部分数据丢失
    assert!(
        result.recovered_frames < result.total_frames,
        "断电场景应该有部分数据丢失"
    );
    
    println!("✅ 断电故障WAL恢复测试通过");
    Ok(())
}

/// 测试文件损坏时的WAL恢复
#[tokio::test]
async fn test_wal_recovery_file_corruption() -> Result<()> {
    println!("🗃️  测试文件损坏WAL恢复...");
    
    let config = WalRecoveryTestConfig {
        expected_recovery_rate: 60.0, // 文件损坏时期望更低的恢复率
        ..Default::default()
    };
    
    let mut test_suite = WalRecoveryTestSuite::new(config.clone()).await?;
    
    let result = test_suite.run_wal_recovery_test(WalTestScenario::FileCorruption).await?;
    
    // 验证恢复率
    assert!(
        result.data_integrity_score >= config.expected_recovery_rate,
        "文件损坏WAL恢复率 {:.2}% 低于期望值 {:.2}%",
        result.data_integrity_score,
        config.expected_recovery_rate
    );
    
    // 应该检测到损坏的数据帧
    assert!(result.corrupted_frames > 0, "应该检测到损坏的数据帧");
    
    println!("✅ 文件损坏WAL恢复测试通过");
    Ok(())
}

/// 测试所有WAL恢复场景
#[tokio::test]
async fn test_all_wal_recovery_scenarios() -> Result<()> {
    println!("🎯 测试所有WAL恢复场景...");
    
    let config = WalRecoveryTestConfig {
        test_data_count: 500, // 减少数据量以加快测试
        ..Default::default()
    };
    
    let mut test_suite = WalRecoveryTestSuite::new(config).await?;
    
    let results = test_suite.run_all_scenarios().await?;
    
    // 验证所有场景都有结果
    assert_eq!(results.len(), 6, "应该有6个测试场景的结果");
    
    // 验证至少有一个场景达到高恢复率
    let max_recovery_rate = results.iter()
        .map(|r| r.data_integrity_score)
        .fold(0.0, f64::max);
    
    assert!(max_recovery_rate >= 95.0, "至少应该有一个场景达到95%以上的恢复率");
    
    // 打印所有结果摘要
    println!("📊 所有WAL恢复场景测试结果:");
    for result in &results {
        println!("  {}: {:.2}% 恢复率, {:?} 恢复时间",
                result.scenario,
                result.data_integrity_score,
                result.recovery_time);
    }
    
    println!("✅ 所有WAL恢复场景测试通过");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_checksum_calculation() {
        let suite = WalRecoveryTestSuite {
            env: unsafe { std::mem::zeroed() }, // 这里只是为了测试，实际不应该这样做
            config: WalRecoveryTestConfig::default(),
            test_data: Vec::new(),
            recovery_results: Vec::new(),
        };
        
        let checksum1 = suite.calculate_checksum(123, "test.tag", 45.67);
        let checksum2 = suite.calculate_checksum(123, "test.tag", 45.67);
        let checksum3 = suite.calculate_checksum(124, "test.tag", 45.67);
        
        assert_eq!(checksum1, checksum2);
        assert_ne!(checksum1, checksum3);
    }
    
    #[tokio::test]
    async fn test_config_creation() {
        let config = WalRecoveryTestConfig::default();
        assert_eq!(config.test_data_count, 1000);
        assert_eq!(config.expected_recovery_rate, 99.9);
    }
}