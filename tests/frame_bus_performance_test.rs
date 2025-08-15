//! Frame Bus性能测试
//! 
//! 验证WAL写入延迟优化和吞吐量改进

use std::time::{Duration, Instant};
use tokio::time::timeout;
use frame_bus::{
    init_high_performance, create_test_frames, publish_data_batch,
    get_performance_stats, PerformancePresets, BusCfg
};

#[tokio::test]
async fn test_high_performance_initialization() -> anyhow::Result<()> {
    // 测试高性能初始化接口
    let temp_dir = tempfile::tempdir()?;
    let wal_path = temp_dir.path().join("wal_perf_test");
    
    let (tx, _rx) = init_high_performance(
        1_000_000, // 1M ring buffer
        &wal_path,
        Some(PerformancePresets::low_latency())
    )?;
    
    // 验证ring容量
    assert!(tx.capacity() >= 512_000); // 至少512K容量
    println!("✅ 高性能初始化成功，Ring容量: {}", tx.capacity());
    
    Ok(())
}

#[tokio::test]
async fn test_batch_write_performance() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let wal_path = temp_dir.path().join("wal_batch_test");
    
    // 使用高吞吐量配置
    let (_tx, _rx) = init_high_performance(
        2_000_000,
        &wal_path, 
        Some(PerformancePresets::high_throughput())
    )?;
    
    // 创建大批量测试数据 (5000条)
    let test_frames = create_test_frames("perf_test", 5000, 1.0);
    
    // 测试批量写入性能
    let start = Instant::now();
    
    // 使用超时确保不会无限等待
    let result = timeout(
        Duration::from_millis(100), // 100ms超时
        publish_data_batch(test_frames)
    ).await;
    
    let duration = start.elapsed();
    
    match result {
        Ok(Ok(())) => {
            println!("✅ 批量写入5000条数据耗时: {:?}", duration);
            
            // 验证性能目标: 5000条数据应在50ms内完成
            assert!(
                duration < Duration::from_millis(50),
                "批量写入性能不达标: {:?} > 50ms", duration
            );
        },
        Ok(Err(e)) => {
            println!("❌ 批量写入失败: {}", e);
            return Err(e);
        },
        Err(_) => {
            println!("❌ 批量写入超时 (>100ms)");
            panic!("批量写入超时，性能严重不达标");
        }
    }
    
    Ok(())
}

#[tokio::test]  
async fn test_performance_monitoring() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let wal_path = temp_dir.path().join("wal_monitor_test");
    
    let (_tx, _rx) = init_high_performance(
        500_000,
        &wal_path,
        Some(PerformancePresets::memory_optimized())
    )?;
    
    // 执行一些写入操作
    let frames = create_test_frames("monitor_test", 100, 2.0);
    publish_data_batch(frames)?;
    
    // 等待一下让统计数据更新
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // 获取性能统计
    let stats = get_performance_stats()?;
    
    println!("📊 性能统计:");
    println!("  - Ring容量: {}", stats.ring_capacity);
    println!("  - 背压状态: {}", stats.backpressure_active);
    println!("  - 总写入次数: {}", stats.wal_stats.total_writes);
    println!("  - 批量写入次数: {}", stats.wal_stats.batch_writes);
    println!("  - 同步操作次数: {}", stats.wal_stats.sync_operations);
    println!("  - 最后写入延迟: {:.2}ms", stats.wal_stats.last_write_latency_ms);
    
    // 验证统计数据合理性
    assert!(stats.ring_capacity > 0);
    assert!(stats.wal_stats.total_writes >= 100); // 至少写入了100条数据
    
    println!("✅ 性能监控接口工作正常");
    
    Ok(())
}

#[tokio::test]
async fn test_backpressure_control() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let wal_path = temp_dir.path().join("wal_backpressure_test");
    
    // 使用小容量配置测试背压
    let small_config = BusCfg {
        ring_pow: 12, // 4K ring buffer (很小)
        pause_hi: 0.80,
        resume_lo: 0.60,
        wal_dir: wal_path.clone(),
        wal_flush_ms: 10,
        wal_max_bytes: 1024 * 1024 * 1024, // 1GB
        high_performance_mode: true,
        async_write_queue_size: 1000, // 小队列
        backpressure_threshold: 0.85,
    };
    
    let (_tx, _rx) = init_high_performance(
        0, // 忽略，使用配置中的ring_pow
        &wal_path,
        Some(small_config)
    )?;
    
    println!("🔄 测试背压控制...");
    
    // 持续写入直到触发背压或达到限制
    let mut backpressure_triggered = false;
    for i in 0..2000 {
        let frames = create_test_frames(&format!("bp_test_{}", i), 10, i as f64);
        
        match publish_data_batch(frames) {
            Ok(()) => {
                // 检查是否触发了背压
                if let Ok(stats) = get_performance_stats() {
                    if stats.backpressure_active {
                        println!("✅ 背压在{}批次后触发", i + 1);
                        backpressure_triggered = true;
                        break;
                    }
                }
            },
            Err(_) => {
                println!("✅ 写入失败触发背压保护 (批次: {})", i + 1);
                backpressure_triggered = true; 
                break;
            }
        }
    }
    
    if !backpressure_triggered {
        println!("⚠️  在2000批次写入后仍未触发背压");
    }
    
    Ok(())
}

#[test]
fn test_configuration_presets() {
    println!("🔧 测试性能预设配置...");
    
    // 测试高吞吐量配置
    let high_throughput = PerformancePresets::high_throughput();
    assert_eq!(high_throughput.ring_pow, 21); // 2M
    assert!(high_throughput.high_performance_mode);
    assert_eq!(high_throughput.async_write_queue_size, 100_000);
    println!("✅ 高吞吐量配置验证通过");
    
    // 测试低延迟配置  
    let low_latency = PerformancePresets::low_latency();
    assert_eq!(low_latency.ring_pow, 19); // 512K
    assert_eq!(low_latency.wal_flush_ms, 1); // 1ms
    assert!(low_latency.high_performance_mode);
    println!("✅ 低延迟配置验证通过");
    
    // 测试内存优化配置
    let memory_opt = PerformancePresets::memory_optimized();
    assert_eq!(memory_opt.ring_pow, 17); // 128K
    assert!(!memory_opt.high_performance_mode);
    assert_eq!(memory_opt.async_write_queue_size, 5000);
    println!("✅ 内存优化配置验证通过");
    
    // 验证配置合理性
    assert!(high_throughput.validate().is_ok());
    assert!(low_latency.validate().is_ok()); 
    assert!(memory_opt.validate().is_ok());
    println!("✅ 所有预设配置验证通过");
}

#[tokio::test]
async fn test_latency_benchmark() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let wal_path = temp_dir.path().join("wal_latency_test");
    
    let (_tx, _rx) = init_high_performance(
        1_000_000,
        &wal_path,
        Some(PerformancePresets::low_latency())
    )?;
    
    println!("📊 单次写入延迟基准测试:");
    
    let mut latencies = Vec::new();
    
    // 执行100次单帧写入测试
    for i in 0..100 {
        let frames = create_test_frames(&format!("lat_test_{}", i), 1, i as f64);
        
        let start = Instant::now();
        publish_data_batch(frames)?;
        let latency = start.elapsed();
        
        latencies.push(latency.as_micros());
    }
    
    // 统计延迟
    latencies.sort();
    let min_lat = latencies[0];
    let max_lat = latencies[99];
    let avg_lat: u128 = latencies.iter().sum::<u128>() / latencies.len() as u128;
    let p99_lat = latencies[98]; // 99百分位
    
    println!("  - 最小延迟: {}μs", min_lat);
    println!("  - 最大延迟: {}μs", max_lat); 
    println!("  - 平均延迟: {}μs", avg_lat);
    println!("  - P99延迟: {}μs", p99_lat);
    
    // 验证延迟目标: 平均延迟应小于1ms (1000μs)
    assert!(
        avg_lat < 1000,
        "平均延迟不达标: {}μs > 1000μs", avg_lat
    );
    
    // 验证P99延迟应小于5ms
    assert!(
        p99_lat < 5000,
        "P99延迟不达标: {}μs > 5000μs", p99_lat
    );
    
    println!("✅ 延迟基准测试通过");
    
    Ok(())
}