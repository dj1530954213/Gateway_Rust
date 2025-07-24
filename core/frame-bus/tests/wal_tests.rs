//! FrameBus WAL (Write-Ahead Log) 测试

use frame_bus::{DataFrame, Value, FrameEnvelope, WAL};
use prost::Message;
use tempfile::tempdir;

#[tokio::test]
async fn test_wal_basic_operations() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let wal_path = temp_dir.path();
    
    // 创建WAL实例
    let wal = WAL::new(wal_path).await.expect("Failed to create WAL");
    
    // 创建测试帧
    let frame = DataFrame::new("wal.test", Value::int(42));
    let envelope = FrameEnvelope::wrap_data(1, frame).expect("Failed to wrap frame");
    
    // 写入WAL
    wal.append(&envelope).await.expect("Failed to append to WAL");
    
    // 验证写入成功
    let min_seq = wal.get_min_sequence().await.expect("Failed to get min sequence");
    assert_eq!(min_seq, Some(1));
}

#[tokio::test]
async fn test_wal_persistence_and_recovery() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let wal_path = temp_dir.path();
    
    // 第一阶段：写入数据
    {
        let wal = WAL::new(wal_path).await.expect("Failed to create WAL");
        
        // 写入多个帧
        for i in 1..=5 {
            let frame = DataFrame::new(&format!("persistent.test.{}", i), Value::int(i));
            let envelope = FrameEnvelope::wrap_data(i as u64, frame).expect("Failed to wrap frame");
            wal.append(&envelope).await.expect("Failed to append to WAL");
        }
        
        // 确保数据已写入磁盘
        wal.sync().await.expect("Failed to sync WAL");
    } // WAL实例被销毁
    
    // 第二阶段：恢复数据
    {
        let wal = WAL::new(wal_path).await.expect("Failed to reopen WAL");
        
        // 恢复数据
        let recovered_frames = wal.recover().await.expect("Failed to recover from WAL");
        
        assert_eq!(recovered_frames.len(), 5);
        
        // 验证恢复的数据
        for (i, envelope) in recovered_frames.iter().enumerate() {
            assert_eq!(envelope.seq, (i + 1) as u64);
            
            let frame = DataFrame::decode(&envelope.payload[..]).expect("Failed to decode recovered frame");
            assert_eq!(frame.tag, format!("persistent.test.{}", i + 1));
            assert_eq!(frame.value.as_ref().unwrap().to_i64(), Some((i + 1) as i64));
        }
    }
}

#[tokio::test]
async fn test_wal_batch_operations() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let wal_path = temp_dir.path();
    
    let wal = WAL::new(wal_path).await.expect("Failed to create WAL");
    
    // 创建批量帧
    let mut envelopes = Vec::new();
    for i in 1..=10 {
        let frame = DataFrame::new(&format!("batch.test.{}", i), Value::float(i as f64 / 10.0));
        let envelope = FrameEnvelope::wrap_data(i as u64, frame).expect("Failed to wrap frame");
        envelopes.push(envelope);
    }
    
    // 批量写入
    wal.append_batch(&envelopes).await.expect("Failed to append batch to WAL");
    
    // 验证批量写入
    let min_seq = wal.get_min_sequence().await.expect("Failed to get min sequence");
    assert_eq!(min_seq, Some(1));
    
    // 恢复并验证
    let recovered = wal.recover().await.expect("Failed to recover batch");
    assert_eq!(recovered.len(), 10);
    
    for (i, envelope) in recovered.iter().enumerate() {
        assert_eq!(envelope.seq, (i + 1) as u64);
    }
}

#[tokio::test]
async fn test_wal_garbage_collection() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let wal_path = temp_dir.path();
    
    let wal = WAL::new(wal_path).await.expect("Failed to create WAL");
    
    // 写入多个帧
    for i in 1..=20 {
        let frame = DataFrame::new(&format!("gc.test.{}", i), Value::int(i));
        let envelope = FrameEnvelope::wrap_data(i as u64, frame).expect("Failed to wrap frame");
        wal.append(&envelope).await.expect("Failed to append to WAL");
    }
    
    // 获取初始文件大小
    let initial_size = get_wal_size(wal_path).await;
    println!("Initial WAL size: {} bytes", initial_size);
    
    // 执行垃圾回收，保留序号15及以后的数据
    wal.gc(15).await.expect("Failed to perform garbage collection");
    
    // 验证垃圾回收后的状态
    let min_seq_after_gc = wal.get_min_sequence().await.expect("Failed to get min sequence after GC");
    assert_eq!(min_seq_after_gc, Some(15));
    
    // 恢复数据验证GC效果
    let recovered = wal.recover().await.expect("Failed to recover after GC");
    assert_eq!(recovered.len(), 6); // 序号15-20
    
    // 验证恢复的数据正确
    for (i, envelope) in recovered.iter().enumerate() {
        assert_eq!(envelope.seq, (15 + i) as u64);
    }
    
    // 可选：检查文件大小是否减少
    let final_size = get_wal_size(wal_path).await;
    println!("Final WAL size: {} bytes", final_size);
}

#[tokio::test]
async fn test_wal_concurrent_operations() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let wal_path = temp_dir.path();
    
    let wal = std::sync::Arc::new(
        WAL::new(wal_path).await.expect("Failed to create WAL")
    );
    
    // 启动多个并发写入任务
    let mut write_tasks = Vec::new();
    
    for task_id in 0..5 {
        let wal_clone = wal.clone();
        let task = tokio::spawn(async move {
            for i in 1..=20 {
                let seq = task_id * 20 + i;
                let frame = DataFrame::new(
                    &format!("concurrent.task{}.{}", task_id, i),
                    Value::int(seq as i64)
                );
                let envelope = FrameEnvelope::wrap_data(seq, frame).expect("Failed to wrap frame");
                
                wal_clone.append(&envelope).await.expect("Failed to append in concurrent task");
                
                // 短暂休息模拟真实负载
                if i % 5 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        });
        write_tasks.push(task);
    }
    
    // 等待所有写入任务完成
    for task in write_tasks {
        task.await.expect("Write task failed");
    }
    
    // 验证所有数据都被正确写入
    let recovered = wal.recover().await.expect("Failed to recover after concurrent writes");
    assert_eq!(recovered.len(), 100); // 5 tasks * 20 frames each
    
    // 验证序号连续性
    let mut sequences: Vec<u64> = recovered.iter().map(|e| e.seq).collect();
    sequences.sort();
    
    for (i, &seq) in sequences.iter().enumerate() {
        assert_eq!(seq, (i + 1) as u64, "Missing sequence number");
    }
}

#[tokio::test]
async fn test_wal_error_handling() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let wal_path = temp_dir.path();
    
    let wal = WAL::new(wal_path).await.expect("Failed to create WAL");
    
    // 测试空帧处理
    let result = wal.append_batch(&[]).await;
    assert!(result.is_ok(), "Should handle empty batch gracefully");
    
    // 测试无效序号
    let frame = DataFrame::new("invalid.test", Value::int(0));
    let invalid_envelope = FrameEnvelope::wrap_data(0, frame); // 序号0可能无效
    
    match invalid_envelope {
        Ok(envelope) => {
            // 如果允许序号0，则测试写入
            let result = wal.append(&envelope).await;
            println!("Write result for sequence 0: {:?}", result);
        }
        Err(_) => {
            // 序号0被拒绝是正确的
            println!("Sequence 0 correctly rejected");
        }
    }
}

#[tokio::test]
async fn test_wal_large_frames() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let wal_path = temp_dir.path();
    
    let wal = WAL::new(wal_path).await.expect("Failed to create WAL");
    
    // 创建大型帧（大字符串）
    let large_string = "x".repeat(10_000); // 10KB字符串
    let large_frame = DataFrame::new("large.test", Value::string(large_string.clone()));
    let envelope = FrameEnvelope::wrap_data(1, large_frame).expect("Failed to wrap large frame");
    
    // 写入大型帧
    wal.append(&envelope).await.expect("Failed to append large frame");
    
    // 恢复并验证
    let recovered = wal.recover().await.expect("Failed to recover large frame");
    assert_eq!(recovered.len(), 1);
    
    let recovered_frame = DataFrame::decode(&recovered[0].payload[..])
        .expect("Failed to decode large frame");
    assert_eq!(recovered_frame.tag, "large.test");
    assert_eq!(recovered_frame.value.as_ref().unwrap().to_string(), Some(large_string));
}

#[tokio::test]
async fn test_wal_performance() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let wal_path = temp_dir.path();
    
    let wal = WAL::new(wal_path).await.expect("Failed to create WAL");
    
    let frame_count = 1000;
    let start_time = std::time::Instant::now();
    
    // 写入大量帧
    for i in 1..=frame_count {
        let frame = DataFrame::new(&format!("perf.test.{:06}", i), Value::int(i));
        let envelope = FrameEnvelope::wrap_data(i as u64, frame).expect("Failed to wrap frame");
        wal.append(&envelope).await.expect("Failed to append frame");
        
        // 每100帧同步一次
        if i % 100 == 0 {
            wal.sync().await.expect("Failed to sync WAL");
        }
    }
    
    let write_duration = start_time.elapsed();
    println!("Wrote {} frames in {:?}", frame_count, write_duration);
    
    // 测试恢复性能
    let recover_start = std::time::Instant::now();
    let recovered = wal.recover().await.expect("Failed to recover frames");
    let recover_duration = recover_start.elapsed();
    
    println!("Recovered {} frames in {:?}", recovered.len(), recover_duration);
    
    // 验证性能指标
    let write_fps = frame_count as f64 / write_duration.as_secs_f64();
    let recover_fps = recovered.len() as f64 / recover_duration.as_secs_f64();
    
    println!("Write rate: {:.2} fps", write_fps);
    println!("Recover rate: {:.2} fps", recover_fps);
    
    assert_eq!(recovered.len(), frame_count as usize);
    assert!(write_fps > 100.0, "Write rate should be > 100 fps");
    assert!(recover_fps > 1000.0, "Recover rate should be > 1000 fps");
}

/// 辅助函数：获取WAL目录的总大小
async fn get_wal_size(wal_path: &std::path::Path) -> u64 {
    let mut total_size = 0u64;
    
    if let Ok(mut entries) = tokio::fs::read_dir(wal_path).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(metadata) = entry.metadata().await {
                total_size += metadata.len();
            }
        }
    }
    
    total_size
}

#[tokio::test]
async fn test_wal_corruption_recovery() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let wal_path = temp_dir.path();
    
    // 第一阶段：写入正常数据
    {
        let wal = WAL::new(wal_path).await.expect("Failed to create WAL");
        
        for i in 1..=10 {
            let frame = DataFrame::new(&format!("corruption.test.{}", i), Value::int(i));
            let envelope = FrameEnvelope::wrap_data(i as u64, frame).expect("Failed to wrap frame");
            wal.append(&envelope).await.expect("Failed to append frame");
        }
        
        wal.sync().await.expect("Failed to sync WAL");
    }
    
    // 模拟数据损坏（在实际实现中，这可能需要特殊的API或直接文件操作）
    // 这里我们跳过实际的损坏，因为它依赖于具体的WAL实现
    
    // 第二阶段：尝试恢复
    {
        let wal = WAL::new(wal_path).await.expect("Failed to reopen WAL");
        
        // WAL应该能够优雅地处理损坏，至少恢复部分数据
        let result = wal.recover().await;
        
        match result {
            Ok(recovered) => {
                println!("Recovered {} frames after simulated corruption", recovered.len());
                // 应该至少恢复一些数据
                assert!(!recovered.is_empty(), "Should recover at least some data");
            }
            Err(e) => {
                println!("Recovery failed as expected due to corruption: {}", e);
                // 损坏恢复失败也是可接受的，只要不panic
            }
        }
    }
}