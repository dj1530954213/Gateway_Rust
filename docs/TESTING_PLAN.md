# Edge Gateway 测试计划

## 📋 测试策略概览

### 测试目标
- **功能正确性**: 确保所有模块按设计要求工作
- **性能达标**: 验证5k fps吞吐量和微秒级延迟目标
- **稳定性**: 长时间运行无内存泄漏和崩溃
- **健壮性**: 异常情况下的恢复能力
- **集成完整性**: 模块间协作的端到端验证

### 测试分层

```
┌─────────────────────────────────────────┐
│           E2E / 系统测试                 │
├─────────────────────────────────────────┤
│           集成测试                       │
├─────────────────────────────────────────┤
│           单元测试                       │
└─────────────────────────────────────────┘
```

### 测试覆盖率目标
- **单元测试覆盖率**: ≥ 85%
- **集成测试覆盖率**: ≥ 70%
- **关键路径覆盖**: 100%

## 🔧 测试工具和框架

### 核心测试框架
- **单元测试**: Rust内置 `#[cfg(test)]` + `assert!`
- **异步测试**: `tokio-test` 
- **Mock框架**: `mockall` + `wiremock`
- **属性测试**: `proptest` (模糊测试)
- **性能测试**: `criterion` (微基准测试)

### 集成测试工具
- **容器测试**: `testcontainers-rs`
- **HTTP测试**: `reqwest` + `wiremock`
- **负载测试**: `k6` + 自定义Rust工具
- **内存分析**: `valgrind` + `heaptrack`

### CI/CD工具
- **代码覆盖率**: `tarpaulin`
- **静态分析**: `clippy` + `audit`
- **格式检查**: `rustfmt`
- **文档测试**: `cargo test --doc`

## 📦 单元测试规划

### 1. EndpointKit (L0) 测试

#### 测试文件结构
```
core/endpoint-kit/tests/
├── url_tests.rs          # URL解析测试
├── pool_tests.rs         # 连接池测试
├── decorators_tests.rs   # 装饰器测试
└── control_tests.rs      # 控制通道测试
```

#### 关键测试用例

**URL解析测试**
```rust
#[test]
fn test_tcp_url_parsing() {
    let url = "tcp://192.168.1.100:502";
    let parsed = EndpointUrl::parse(url).unwrap();
    assert_eq!(parsed.scheme(), "tcp");
    assert_eq!(parsed.host(), "192.168.1.100");
    assert_eq!(parsed.port(), 502);
}

#[test]
fn test_scheme_stacking() {
    let url = "tcp+tls://example.com:8883?sni=test";
    let parsed = EndpointUrl::parse(url).unwrap();
    assert_eq!(parsed.schemes(), vec!["tcp", "tls"]);
}
```

**连接池测试**
```rust
#[tokio::test]
async fn test_connection_pool_lifecycle() {
    let pool = ConnectionPool::new(config).await;
    
    // 测试获取连接
    let conn1 = pool.acquire().await.unwrap();
    assert_eq!(pool.active_connections(), 1);
    
    // 测试连接复用
    drop(conn1);
    let conn2 = pool.acquire().await.unwrap();
    assert_eq!(pool.active_connections(), 1);
}

#[tokio::test]
async fn test_pool_exhaustion() {
    let pool = ConnectionPool::new(small_config).await;
    let _conns: Vec<_> = (0..10).map(|_| pool.acquire()).collect();
    
    // 应该达到最大连接数限制
    let result = timeout(Duration::from_millis(100), pool.acquire()).await;
    assert!(result.is_err());
}
```

### 2. FrameBus (L2) 测试

#### 测试文件结构
```
core/frame-bus/tests/
├── ring_buffer_tests.rs    # 环形缓冲测试
├── wal_tests.rs           # WAL持久化测试
├── backpressure_tests.rs  # 背压控制测试
├── filter_tests.rs        # 过滤器测试
└── integration_tests.rs   # 总线集成测试
```

#### 关键测试用例

**环形缓冲测试**
```rust
#[tokio::test]
async fn test_ring_buffer_publish_subscribe() {
    let bus = FrameBus::new(config).await.unwrap();
    let mut rx = bus.subscribe(Filter::All).unwrap();
    
    let frame = DataFrame::new("test.tag", Value::int(42));
    bus.publish_data(frame.clone()).unwrap();
    
    let received = rx.recv().await.unwrap();
    assert_eq!(received.tag, "test.tag");
}

#[tokio::test]
async fn test_backpressure_control() {
    let config = FrameBusConfig {
        capacity: 10,
        pause_hi: 0.8,
        pause_lo: 0.6,
    };
    let bus = FrameBus::new(config).await.unwrap();
    
    // 填满缓冲区触发背压
    for i in 0..12 {
        let frame = DataFrame::new(&format!("test.{}", i), Value::int(i));
        let result = bus.publish_data(frame);
        if i >= 10 {
            assert!(result.is_err()); // 应该触发背压
        }
    }
}
```

**WAL持久化测试**
```rust
#[tokio::test]
async fn test_wal_persistence_and_recovery() {
    let temp_dir = tempdir().unwrap();
    let wal = WAL::new(temp_dir.path()).await.unwrap();
    
    // 写入数据
    let frame = DataFrame::new("persistent.tag", Value::float(3.14));
    let envelope = FrameEnvelope::wrap_data(1, frame).unwrap();
    wal.append(&envelope).await.unwrap();
    
    // 重新打开WAL验证恢复
    drop(wal);
    let wal2 = WAL::new(temp_dir.path()).await.unwrap();
    let recovered = wal2.recover().await.unwrap();
    assert_eq!(recovered.len(), 1);
}
```

### 3. Modbus驱动测试

#### 测试文件结构
```
drivers/modbus-static/tests/
├── codec_tests.rs         # 编解码测试
├── driver_tests.rs        # 驱动逻辑测试
├── batch_tests.rs         # 批量读取测试
└── mock_server_tests.rs   # Mock服务器测试
```

#### 关键测试用例

**编解码测试**
```rust
#[test]
fn test_uint16_decoding() {
    let regs = vec![0x1234];
    let point = RegPoint {
        datatype: DataType::Uint16,
        addr: 0,
        len: 1,
        // ...
    };
    
    let value = decode_registers(&regs, &point, 0, &Endian::Big).unwrap();
    assert_eq!(value.to_i64(), Some(0x1234));
}

#[test]
fn test_float32_encoding() {
    let value = Value::float(3.14159);
    let regs = encode_value(&value, &DataType::Float32, &Endian::Big).unwrap();
    assert_eq!(regs.len(), 2);
    
    // 验证往返转换
    let decoded = decode_float32(&regs, &Endian::Big).unwrap();
    assert!((decoded - 3.14159).abs() < 0.0001);
}
```

**批量读取测试**
```rust
#[test]
fn test_point_grouping() {
    let points = vec![
        RegPoint { addr: 40001, len: 1, func: ReadHolding, .. },
        RegPoint { addr: 40002, len: 1, func: ReadHolding, .. },
        RegPoint { addr: 40010, len: 2, func: ReadHolding, .. },
    ];
    
    let batches = group_points_to_batches(&points, 125);
    assert_eq!(batches.len(), 2); // 应该分成2个批次
    assert_eq!(batches[0].qty, 2); // 前两个点合并
    assert_eq!(batches[1].qty, 2); // 第三个点单独批次
}
```

### 4. MQTT连接器测试

#### 测试文件结构
```
connectors/mqtt5/tests/
├── connector_tests.rs     # 连接器测试
├── batcher_tests.rs       # 批量处理测试
├── compression_tests.rs   # 压缩测试
└── reconnect_tests.rs     # 重连测试
```

#### 关键测试用例

**消息格式测试**
```rust
#[test]
fn test_mqtt_message_serialization() {
    let message = MqttMessage {
        device_id: "gateway-001".to_string(),
        timestamp: 1609459200000,
        points: vec![
            DataPoint {
                tag: "temp.sensor1".to_string(),
                value: json!(25.5),
                quality: 2,
                meta: HashMap::new(),
            }
        ],
    };
    
    let json = serde_json::to_string(&message).unwrap();
    let parsed: MqttMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.device_id, "gateway-001");
}
```

**批量处理测试**
```rust
#[tokio::test]
async fn test_batch_accumulation() {
    let config = BatchConfig {
        size: 3,
        timeout: Duration::from_millis(100),
    };
    let batcher = Batcher::new(config);
    
    // 添加2个点，不应该触发发送
    batcher.add_point(point1).await;
    batcher.add_point(point2).await;
    assert!(!batcher.should_flush());
    
    // 添加第3个点，应该触发发送
    batcher.add_point(point3).await;
    assert!(batcher.should_flush());
}
```

## 🔗 集成测试规划

### 1. 端到端数据流测试

#### 测试场景
```rust
#[tokio::test]
async fn test_end_to_end_data_flow() {
    // 1. 启动测试环境
    let test_env = TestEnvironment::new().await;
    let mock_plc = test_env.start_mock_plc().await;
    let mqtt_broker = test_env.start_mqtt_broker().await;
    
    // 2. 配置网关
    let gateway = EdgeGateway::new(test_config()).await;
    gateway.start().await;
    
    // 3. 模拟PLC数据变化
    mock_plc.set_register(40001, 1500).await;
    mock_plc.set_register(40002, 500).await;
    
    // 4. 验证MQTT消息接收
    let mqtt_client = mqtt_broker.create_test_client().await;
    mqtt_client.subscribe("gateway/data/+").await;
    
    let message = mqtt_client.wait_for_message(Duration::from_secs(5)).await;
    let data: MqttMessage = serde_json::from_slice(&message.payload).unwrap();
    
    assert_eq!(data.points.len(), 2);
    assert_eq!(data.points[0].value, json!(150.0)); // 1500/10
    assert_eq!(data.points[1].value, json!(5.0));   // 500/100
}
```

### 2. 配置热重载测试

```rust
#[tokio::test]
async fn test_configuration_hot_reload() {
    let test_env = TestEnvironment::new().await;
    let gateway = EdgeGateway::new(test_config()).await;
    
    // 启动网关
    gateway.start().await;
    
    // 修改配置文件
    let new_config = r#"
    drivers:
      modbus1:
        polling: "500ms"  # 从1s改为500ms
    "#;
    fs::write("config/drivers.yml", new_config).await.unwrap();
    
    // 等待配置重载
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 验证轮询间隔已更新
    let metrics = gateway.get_metrics().await;
    assert!(metrics.modbus_pdu_rate > 1.8); // 应该接近2 Hz
}
```

### 3. 故障恢复测试

```rust
#[tokio::test]
async fn test_network_failure_recovery() {
    let test_env = TestEnvironment::new().await;
    let mqtt_broker = test_env.start_mqtt_broker().await;
    let gateway = EdgeGateway::new(test_config()).await;
    
    gateway.start().await;
    
    // 模拟网络故障
    mqtt_broker.stop().await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // 验证缓冲区积累
    let metrics = gateway.get_metrics().await;
    assert!(metrics.mqtt_buffer_used > 0);
    
    // 恢复网络
    mqtt_broker.start().await;
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // 验证数据续传
    let final_metrics = gateway.get_metrics().await;
    assert_eq!(final_metrics.mqtt_buffer_used, 0);
}
```

## ⚡ 性能和压力测试

### 1. 吞吐量基准测试

```rust
#[tokio::test]
async fn test_5k_fps_throughput() {
    let config = PerformanceTestConfig {
        target_fps: 5000,
        duration: Duration::from_secs(600), // 10分钟
        point_count: 1000,
    };
    
    let test_env = TestEnvironment::new().await;
    let mock_plc = test_env.start_high_frequency_plc(config).await;
    let gateway = EdgeGateway::new(performance_config()).await;
    
    // 启动性能监控
    let monitor = PerformanceMonitor::new();
    monitor.start().await;
    
    gateway.start().await;
    
    // 运行10分钟
    tokio::time::sleep(config.duration).await;
    
    let results = monitor.get_results().await;
    
    // 验证性能指标
    assert!(results.avg_fps >= 4900.0); // 允许2%误差
    assert!(results.p99_latency_us < 150.0);
    assert!(results.memory_growth_mb < 10.0);
    assert_eq!(results.dropped_frames, 0);
}
```

### 2. 内存泄漏测试

```rust
#[tokio::test]
async fn test_memory_leak_detection() {
    let gateway = EdgeGateway::new(test_config()).await;
    gateway.start().await;
    
    let initial_memory = get_memory_usage();
    
    // 运行1小时，每秒1000次操作
    for _ in 0..3600 {
        simulate_high_load_operations().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // 每5分钟检查一次内存
        if i % 300 == 0 {
            let current_memory = get_memory_usage();
            let growth = current_memory - initial_memory;
            assert!(growth < 50 * 1024 * 1024); // 增长不超过50MB
        }
    }
}
```

### 3. 并发安全测试

```rust
#[tokio::test]
async fn test_concurrent_safety() {
    let gateway = EdgeGateway::new(test_config()).await;
    gateway.start().await;
    
    // 启动多个并发任务
    let tasks: Vec<_> = (0..100).map(|i| {
        tokio::spawn(async move {
            for j in 0..1000 {
                let frame = DataFrame::new(
                    &format!("concurrent.test.{}.{}", i, j),
                    Value::int(j)
                );
                frame_bus::publish_data(frame).unwrap();
            }
        })
    }).collect();
    
    // 等待所有任务完成
    for task in tasks {
        task.await.unwrap();
    }
    
    // 验证数据完整性
    let metrics = gateway.get_metrics().await;
    assert_eq!(metrics.frame_publish_total, 100_000);
    assert_eq!(metrics.frame_drop_total, 0);
}
```

## 🛠️ 测试工具实现

### 1. Mock PLC服务器

```rust
// tests/common/mock_plc.rs
pub struct MockPLCServer {
    registers: Arc<RwLock<HashMap<u16, u16>>>,
    server: Option<TcpListener>,
    port: u16,
}

impl MockPLCServer {
    pub async fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        
        Self {
            registers: Arc::new(RwLock::new(HashMap::new())),
            server: Some(listener),
            port,
        }
    }
    
    pub async fn start(&mut self) {
        let listener = self.server.take().unwrap();
        let registers = self.registers.clone();
        
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let registers = registers.clone();
                tokio::spawn(handle_modbus_connection(stream, registers));
            }
        });
    }
    
    pub async fn set_register(&self, addr: u16, value: u16) {
        self.registers.write().await.insert(addr, value);
    }
    
    pub fn endpoint_url(&self) -> String {
        format!("tcp://127.0.0.1:{}", self.port)
    }
}
```

### 2. 测试环境管理器

```rust
// tests/common/test_env.rs
pub struct TestEnvironment {
    temp_dir: TempDir,
    mock_plc: Option<MockPLCServer>,
    mqtt_broker: Option<TestMqttBroker>,
}

impl TestEnvironment {
    pub async fn new() -> Self {
        let temp_dir = tempdir().unwrap();
        
        Self {
            temp_dir,
            mock_plc: None,
            mqtt_broker: None,
        }
    }
    
    pub async fn start_mock_plc(&mut self) -> &MockPLCServer {
        let mut plc = MockPLCServer::new().await;
        plc.start().await;
        self.mock_plc = Some(plc);
        self.mock_plc.as_ref().unwrap()
    }
    
    pub fn create_test_config(&self) -> GatewayConfig {
        GatewayConfig {
            config_dir: self.temp_dir.path().to_path_buf(),
            // ... 其他测试配置
        }
    }
}
```

### 3. 性能监控工具

```rust
// tests/common/performance_monitor.rs
pub struct PerformanceMonitor {
    start_time: Instant,
    metrics_history: Vec<MetricsSnapshot>,
}

impl PerformanceMonitor {
    pub async fn start(&mut self) {
        self.start_time = Instant::now();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let snapshot = collect_metrics_snapshot().await;
                self.metrics_history.push(snapshot);
            }
        });
    }
    
    pub fn get_results(&self) -> PerformanceResults {
        let total_frames: u64 = self.metrics_history.iter()
            .map(|s| s.frame_publish_total)
            .sum();
        
        let duration_secs = self.start_time.elapsed().as_secs_f64();
        let avg_fps = total_frames as f64 / duration_secs;
        
        PerformanceResults {
            avg_fps,
            p99_latency_us: calculate_p99_latency(&self.metrics_history),
            memory_growth_mb: calculate_memory_growth(&self.metrics_history),
            dropped_frames: self.metrics_history.last().unwrap().frame_drop_total,
        }
    }
}
```

## 📊 测试覆盖率和报告

### 代码覆盖率配置

```toml
# Cargo.toml
[dependencies]
tarpaulin = "0.27"

[profile.test]
opt-level = 0
debug = true
```

### 覆盖率脚本

```bash
#!/bin/bash
# scripts/test-coverage.sh

echo "🧪 运行测试覆盖率分析"

# 单元测试覆盖率
cargo tarpaulin --out Html --output-dir target/coverage/unit \
    --exclude-files "tests/*" "*/tests/*" \
    --timeout 300

# 集成测试覆盖率
cargo tarpaulin --out Html --output-dir target/coverage/integration \
    --test-threads 1 \
    --tests \
    --timeout 600

echo "📊 覆盖率报告生成完成"
echo "单元测试: target/coverage/unit/tarpaulin-report.html"
echo "集成测试: target/coverage/integration/tarpaulin-report.html"
```

## 🚀 CI/CD集成

### GitHub Actions配置

```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y protobuf-compiler
          
      - name: Run unit tests
        run: cargo test --lib --bins
        
      - name: Run doc tests
        run: cargo test --doc
        
  integration-tests:
    runs-on: ubuntu-latest
    services:
      redis:
        image: redis:6
        ports:
          - 6379:6379
          
    steps:
      - uses: actions/checkout@v3
      - name: Run integration tests
        run: cargo test --tests
        
  performance-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v3
      - name: Run performance tests
        run: |
          cargo build --release
          ./scripts/performance-test.sh
```

这个全面的测试计划涵盖了从单元测试到端到端集成测试的所有层面，确保Edge Gateway项目的代码质量、功能完整性和性能稳定性。