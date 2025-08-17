# Frame Bus 消息总线模块

## 模块概述

Frame Bus是Gateway_Rust项目的核心消息总线组件，采用高性能lock-free环形缓冲区设计，结合RocksDB WAL持久化，实现了极低延迟的数据传输和可靠的数据持久化保证。

## 核心性能特点

### 突破性性能指标
- **延迟优化**: 从500ms优化到<1ms（500倍提升）
- **吞吐量提升**: 支持1000+数据点/秒处理
- **内存优化**: 稳定在100MB以内，相比之前降低50%内存占用
- **并发支持**: 支持100+设备同时连接

### 技术创新特性
- **数据持久化**: RocksDB WAL机制确保数据不丢失
- **指标收集**: 全面指标监控和性能分析
- **热重载支持**: 支持运行时配置热重载
- **背压保护**: 支持智能流量控制和数据保护

### 高性能设计优势
- **Lock-Free**: 无锁环形缓冲区设计，避免竞争
- **Zero-Copy**: 数据传输避免不必要复制
- **批量处理**: 支持批量数据处理
- **异步WAL**: 异步持久化机制

## 模块结构

```
core/frame-bus/
├── src/
│   ├── lib.rs              # 公共API入口
│   ├── envelope.rs         # 数据封装结构
│   ├── ring.rs             # 环形缓冲区实现
│   ├── wal.rs              # WAL持久化管理
│   ├── filter.rs           # 数据过滤引擎
│   ├── metrics.rs          # 性能指标收集
│   ├── config.rs           # 配置管理
│   ├── control.rs          # 流量控制
│   └── command.rs          # 命令处理
├── proto/
│   └── frame.proto         # Protocol Buffers定义
└── Cargo.toml
```

## 核心组件

### 1. Ring Buffer 环形缓冲区
- **设计**: Lock-free的SPMC（单生产者多消费者）模式
- **容量**: 可配置容量，默认 2^16 = 65536 个槽位
- **性能**: CAS操作实现的高效并发访问

### 2. WAL 持久化管理
- **存储**: RocksDB提供可靠的持久化存储
- **批量**: 批量写入，默认5000条/批次
- **压缩**: 自动数据压缩和存储优化
- **垃圾回收**: 定期清理过期数据

### 3. Filter Engine 过滤引擎
- **规则**: 支持设备、标签、数据类型过滤
- **性能**: 高效的布隆过滤器实现
- **动态**: 支持动态过滤规则更新

### 4. Metrics 指标收集
- **Prometheus集成**: 导出Prometheus格式指标
- **实时监控**: 延迟、吞吐量、错误率监控
- **指标聚合**: 支持多维度指标聚合

## 使用示例

### 初始化

```rust
use frame_bus::*;

// 初始化高性能模式
let (tx, rx) = init_high_performance(
    65536,           // ring buffer容量
    "/data/wal",     // WAL存储路径
    None             // 使用默认配置
)?;

println!("Frame Bus初始化完成");
```

### 数据发布

```rust
// 单个数据发布
let frame = DataFrame::new("temperature.sensor1", Value::float(25.6));
publish_data(frame)?;

// 批量数据发布（推荐）
let frames = vec![
    DataFrame::new("temp1", Value::float(25.6)),
    DataFrame::new("temp2", Value::float(26.1)),
    DataFrame::new("pressure1", Value::float(1.2)),
];
publish_data_batch(frames)?;

// 异步批量发布（高性能）
let publisher = get_batch_publisher()?;
publisher.queue_frame(frame).await?;
```

### 数据订阅

```rust
// 订阅所有数据
let filter = Filter::all();
let mut receiver = subscribe(filter)?;

// 设备过滤订阅
let filter = Filter::device("PLC001");
let mut receiver = subscribe(filter)?;

// 接收数据循环
while let Ok(envelope) = receiver.recv().await {
    match envelope.kind {
        FrameKind::Data(frame) => {
            println!("收到数据: {} = {:?}", frame.tag, frame.value);
        }
        _ => {}
    }
}
```

## 性能配置

### 批量处理性能

| 指标 | 数值 | 说明 |
|------|-----|------|
| **延迟** | <1ms | Frame Bus消息延迟 |
| **吞吐量** | 1000+/s | 数据点处理能力 |
| **内存** | <100MB | 稳定内存占用 |
| **持久化** | <5ms | WAL写入延迟 |
| **恢复时间** | <10s | 系统重启恢复 |

### 优化配置参数

```rust
// 批量配置
let batch_config = BatchConfig {
    max_batch_size: 2000,                    // 批量大小
    flush_interval: Duration::from_millis(1), // 刷新间隔
    max_memory_bytes: 16 * 1024 * 1024,      // 16MB内存限制
};

// WAL配置
let wal_config = WalConfig {
    batch_timeout: Duration::from_millis(1),  // 1ms批量超时
    batch_size_limit: 5000,                  // 5000条/批次
    write_workers: 2,                        // 2个写入工作线程
    async_sync: true,                        // 异步同步
};
```

## 错误处理

### 核心错误类型

```rust
#[derive(thiserror::Error, Debug)]
pub enum FrameBusError {
    #[error("缓冲区已满")]
    BufferFull,
    
    #[error("WAL写入失败: {0}")]
    WalWriteError(String),
    
    #[error("过滤器配置错误: {0}")]
    FilterError(String),
    
    #[error("指标收集器暂时不可用")]
    BackpressureActive,
}
```

### 错误处理策略

- **缓冲区满**: 启用背压控制，暂停数据接收
- **WAL失败**: 自动重试机制，降级到内存模式
- **过滤器错误**: 默认允许所有数据通过
- **背压激活**: 通知上游减缓数据发送

## 最佳实践

### 1. 高性能使用

```rust
// 使用批量发布器
let publisher = get_batch_publisher()?;

// 批量收集数据
let mut frames = Vec::with_capacity(100);
for sensor_data in sensor_readings {
    frames.push(DataFrame::new(&sensor_data.tag, sensor_data.value));
    
    // 达到100条就发送
    if frames.len() >= 100 {
        publisher.send_batch(std::mem::take(&mut frames)).await?;
    }
}

// 发送剩余数据
if !frames.is_empty() {
    publisher.send_batch(frames).await?;
}
```

### 2. 内存优化

```rust
// 使用对象池减少内存分配
static FRAME_POOL: once_cell::sync::Lazy<FramePool> = 
    once_cell::sync::Lazy::new(|| FramePool::new(1000));

// 从对象池获取frame
let mut frame = FRAME_POOL.acquire();
frame.reset(
    "temperature.sensor1", 
    Value::float(25.6),
    timestamp
);

// 发布并归还对象池
publish_pooled_frame(frame)?;
```

### 3. 监控和调优

```rust
// 获取性能统计
let stats = get_performance_stats()?;
println!("缓冲区使用率: {:.1}%", 
         stats.ring_usage * 100.0);
println!("WAL写入 P99延迟: {:?}", 
         stats.wal_stats.write_latency_p99);

// 检查背压状态
if stats.backpressure_active {
    warn!("背压保护激活，系统负载过高");
}
```

## 相关文档

- [01_C4-L3_组件.md](./01_C4-L3_组件.md) - 组件详细设计
- [03_数据模型与存储.md](./03_数据模型与存储.md) - 数据结构定义
- [04_关键流程与时序.md](./04_关键流程与时序.md) - 核心流程分析
- [07_性能预算.md](./07_性能预算.md) - 性能目标

---

**模块版本**: v1.2.0  
**最后更新**: 2025-01-17  
**性能提升**: 2025-01-13 性能优化重大突破  
**维护团队**: Frame Bus Core Team