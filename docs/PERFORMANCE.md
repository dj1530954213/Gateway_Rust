# Edge Gateway 性能优化指南

本文档提供工控物联网边缘网关的性能分析、优化策略和监控方法。

## 目录
- [性能概述](#性能概述)
- [性能基准](#性能基准)
- [系统性能分析](#系统性能分析)
- [内存优化](#内存优化)
- [CPU优化](#cpu优化)
- [网络优化](#网络优化)
- [存储优化](#存储优化)
- [监控和分析工具](#监控和分析工具)
- [性能测试](#性能测试)
- [调优实践](#调优实践)

## 性能概述

### 关键性能指标 (KPIs)

| 指标 | 目标值 | 说明 |
|------|--------|------|
| **数据吞吐量** | >10,000 点/秒 | 每秒处理的数据点数量 |
| **响应延迟** | <50ms | API请求平均响应时间 |
| **内存使用率** | <80% | 系统内存使用率 |
| **CPU使用率** | <70% | 系统CPU平均使用率 |
| **磁盘I/O延迟** | <10ms | 数据持久化延迟 |
| **网络延迟** | <100ms | 数据上云延迟 |
| **可用性** | >99.9% | 系统正常运行时间 |

### 性能瓶颈识别

#### 常见瓶颈
1. **CPU密集型**
   - 大量数据处理和转换
   - 复杂的机器学习推理
   - 数据压缩和解压缩

2. **内存密集型**
   - 大量数据缓存
   - 历史数据保存
   - 多连接并发处理

3. **I/O密集型**
   - 频繁的数据库读写
   - 大量日志记录
   - 网络通信延迟

4. **网络带宽限制**
   - 高频数据上云
   - 实时数据同步
   - 远程配置更新

## 性能基准

### 硬件配置基准

#### 最小配置性能
```yaml
Hardware:
  CPU: 2核 2.0GHz
  Memory: 4GB RAM
  Storage: 20GB HDD
  Network: 100Mbps

Expected Performance:
  Data Points: 1,000/秒
  API Latency: <200ms
  Memory Usage: ~2GB
  CPU Usage: 60-80%
```

#### 推荐配置性能
```yaml
Hardware:
  CPU: 4核 2.5GHz
  Memory: 8GB RAM
  Storage: 100GB SSD
  Network: 1Gbps

Expected Performance:
  Data Points: 10,000/秒
  API Latency: <50ms
  Memory Usage: ~4GB
  CPU Usage: 40-60%
```

#### 高性能配置
```yaml
Hardware:
  CPU: 8核 3.0GHz
  Memory: 16GB RAM
  Storage: 500GB NVMe SSD
  Network: 10Gbps

Expected Performance:
  Data Points: 50,000/秒
  API Latency: <20ms
  Memory Usage: ~8GB
  CPU Usage: 30-50%
```

## 系统性能分析

### 性能分析工具

#### 1. 内置性能指标
```rust
// 在gateway.yml中启用详细指标
monitoring:
  enabled: true
  detailed_metrics: true
  
  # 性能分析配置
  profiling:
    enabled: true
    cpu_profiling: true
    memory_profiling: true
    flame_graph: true
```

#### 2. 系统级监控
```bash
# CPU性能分析
top -p $(pgrep edge-gateway)
htop -p $(pgrep edge-gateway)

# 内存分析
ps aux | grep edge-gateway
pmap -x $(pgrep edge-gateway)

# I/O性能
iotop -p $(pgrep edge-gateway)
iostat -x 1
```

## 内存优化

### Rust内存管理优化

#### 1. 智能指针使用
```rust
use std::sync::Arc;
use dashmap::DashMap;

// 优化前：过度使用Box
struct DataProcessor {
    processors: Vec<Box<dyn Processor>>,
    cache: Box<HashMap<String, Data>>,
}

// 优化后：合理使用引用
struct DataProcessor {
    processors: Vec<Arc<dyn Processor>>,
    cache: DashMap<String, Data>,  // 并发友好的HashMap
}
```

#### 2. 零拷贝优化
```rust
use bytes::Bytes;
use std::borrow::Cow;

// 优化前：多次内存拷贝
fn process_data(data: String) -> String {
    let processed = data.to_uppercase();
    format!("PROCESSED: {}", processed)
}

// 优化后：零拷贝处理
fn process_data_zero_copy(data: &[u8]) -> Cow<[u8]> {
    if data.iter().all(|b| b.is_ascii_uppercase()) {
        Cow::Borrowed(data)
    } else {
        Cow::Owned(data.to_ascii_uppercase())
    }
}
```

### 内存配置优化

#### 1. 内存使用监控
```rust
use sysinfo::{System, SystemExt, ProcessExt};

pub struct MemoryMonitor {
    system: System,
}

impl MemoryMonitor {
    pub fn get_memory_usage(&mut self) -> f64 {
        self.system.refresh_process(std::process::id() as usize);
        
        if let Some(process) = self.system.process(std::process::id() as usize) {
            let used = process.memory() as f64;
            let total = self.system.total_memory() as f64;
            (used / total) * 100.0
        } else {
            0.0
        }
    }
}
```

## CPU优化

### 并发处理优化

#### 1. 线程池配置
```rust
use tokio::runtime::Builder;

pub fn create_optimized_runtime() -> tokio::runtime::Runtime {
    Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .max_blocking_threads(512)
        .thread_name("gateway-worker")
        .thread_stack_size(3 * 1024 * 1024)  // 3MB stack
        .build()
        .expect("Failed to create runtime")
}
```

#### 2. 异步批处理
```rust
use tokio::time::{interval, Duration};
use futures::future::BoxFuture;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct BatchProcessor<T> {
    items: Arc<Mutex<Vec<T>>>,
    batch_size: usize,
    interval: Duration,
}

impl<T> BatchProcessor<T> {
    pub async fn start_processing<F>(&self, processor: F)
    where
        F: Fn(Vec<T>) -> BoxFuture<'static, anyhow::Result<()>> + Send + Sync + 'static,
    {
        let mut timer = interval(self.interval);
        
        loop {
            timer.tick().await;
            
            let batch = {
                let mut items = self.items.lock().await;
                if items.len() >= self.batch_size {
                    items.drain(..self.batch_size).collect()
                } else {
                    Vec::new()
                }
            };
            
            if !batch.is_empty() {
                if let Err(e) = processor(batch).await {
                    eprintln!("Batch processing failed: {}", e);
                }
            }
        }
    }
}
```

### 算法优化

#### 1. 数据结构选择
```rust
use std::collections::HashMap;

// 优化前：使用Vec进行频繁查找
struct DataIndex {
    items: Vec<(String, DataPoint)>,
}

impl DataIndex {
    fn find(&self, id: &str) -> Option<&DataPoint> {
        self.items.iter()
            .find(|(key, _)| key == id)
            .map(|(_, value)| value)
    }
}

// 优化后：使用HashMap提高查找效率
struct DataIndex {
    items: HashMap<String, DataPoint>,
}

impl DataIndex {
    fn find(&self, id: &str) -> Option<&DataPoint> {
        self.items.get(id)
    }
}
```

## 网络优化

### 连接管理优化

#### 1. 连接池配置
```rust
use std::time::Duration;

pub struct OptimizedConnectionPool {
    http_client: reqwest::Client,
}

impl OptimizedConnectionPool {
    pub fn new() -> anyhow::Result<Self> {
        // HTTP客户端配置
        let http_client = reqwest::Client::builder()
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(10))
            .tcp_keepalive(Duration::from_secs(60))
            .build()?;
        
        Ok(Self {
            http_client,
        })
    }
}
```

#### 2. 数据压缩优化
```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

pub struct CompressionManager {
    compression_level: Compression,
    threshold_size: usize,
}

impl CompressionManager {
    pub fn new() -> Self {
        Self {
            compression_level: Compression::fast(),
            threshold_size: 1024,  // 1KB阈值
        }
    }
    
    pub fn compress_if_beneficial(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < self.threshold_size {
            return data.to_vec();
        }
        
        let mut encoder = GzEncoder::new(Vec::new(), self.compression_level);
        if encoder.write_all(data).is_ok() {
            if let Ok(compressed) = encoder.finish() {
                if compressed.len() < data.len() * 9 / 10 {  // 至少10%压缩率
                    return compressed;
                }
            }
        }
        
        data.to_vec()
    }
}
```

## 存储优化

### 数据库性能优化

#### 1. RocksDB调优
```rust
use rocksdb::{DB, Options};

pub fn create_optimized_rocksdb(path: &str) -> Result<DB, rocksdb::Error> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.set_compression_type(rocksdb::DBCompressionType::Snappy);
    
    // 内存相关配置
    opts.set_write_buffer_size(64 * 1024 * 1024);  // 64MB
    opts.set_max_write_buffer_number(3);
    opts.set_target_file_size_base(64 * 1024 * 1024);
    
    // 并发配置
    opts.set_max_background_jobs(4);
    opts.set_max_subcompactions(2);
    
    DB::open(&opts, path)
}
```

#### 2. 缓存策略优化
```rust
use std::collections::HashMap;
use std::hash::Hash;

#[async_trait::async_trait]
pub trait Cache<K, V>: Send + Sync {
    async fn get(&self, key: &K) -> Option<V>;
    async fn put(&self, key: K, value: V);
}

pub struct MultiLevelCache<K, V> {
    l1_cache: HashMap<K, V>,           // 内存缓存
    l2_cache: Box<dyn Cache<K, V>>,    // 分布式缓存
}

impl<K, V> MultiLevelCache<K, V> 
where
    K: Clone + Hash + Eq,
    V: Clone,
{
    pub fn new(l2_cache: Box<dyn Cache<K, V>>) -> Self {
        Self {
            l1_cache: HashMap::new(),
            l2_cache,
        }
    }
    
    pub async fn get(&mut self, key: &K) -> Option<V> {
        // 先查L1缓存
        if let Some(value) = self.l1_cache.get(key) {
            return Some(value.clone());
        }
        
        // 再查L2缓存
        if let Some(value) = self.l2_cache.get(key).await {
            self.l1_cache.insert(key.clone(), value.clone());
            return Some(value);
        }
        
        None
    }
    
    pub async fn put(&mut self, key: K, value: V) {
        self.l1_cache.insert(key.clone(), value.clone());
        self.l2_cache.put(key, value).await;
    }
}
```

## 监控和分析工具

### 性能监控配置

#### 1. Prometheus指标收集
```yaml
# prometheus.yml配置
global:
  scrape_interval: 5s

scrape_configs:
  - job_name: 'gateway-performance'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 1s
    metrics_path: /metrics
```

#### 2. 告警规则
```yaml
# prometheus/rules/performance.yml
groups:
  - name: performance
    rules:
      - alert: HighCPUUsage
        expr: process_cpu_seconds_total > 0.8
        for: 5m
        annotations:
          summary: "Gateway CPU usage is high"
          
      - alert: HighMemoryUsage
        expr: process_resident_memory_bytes > 8*1024*1024*1024
        for: 5m
        annotations:
          summary: "Gateway memory usage is high"
          
      - alert: LowThroughput
        expr: rate(datapoints_processed_total[5m]) < 1000
        for: 10m
        annotations:
          summary: "Gateway throughput is low"
```

## 性能测试

### 基准测试套件

#### 1. 负载测试脚本
```lua
-- load-test.lua for wrk
local counter = 0

request = function()
   counter = counter + 1
   
   local data = string.format([[{
     "driver_id": "test_driver_%d",
     "points": [
       {
         "id": "temp_%d",
         "value": %f,
         "timestamp": "%s"
       }
     ]
   }]], counter % 10, counter, math.random() * 100, os.date("!%Y-%m-%dT%H:%M:%SZ"))
   
   return wrk.format("POST", "/api/v1/datapoints", {
     ["Content-Type"] = "application/json"
   }, data)
end

response = function(status, headers, body)
   if status ~= 200 then
      print("Error: " .. status .. " " .. body)
   end
end
```

#### 2. 性能验证脚本
```bash
#!/bin/bash
# performance-validation.sh

echo "🚀 Starting performance validation..."

# 基础性能测试
echo "📊 Running basic performance test..."
timeout 30s wrk -t4 -c100 http://localhost:8080/health > basic-perf.txt

# 内存使用测试
echo "💾 Monitoring memory usage..."
timeout 60s bash -c 'while true; do 
  ps -p $(pgrep edge-gateway) -o %mem= | tr -d " "
  echo
  sleep 5
done' > memory-usage.txt

# CPU使用测试
echo "⚡ Monitoring CPU usage..."
timeout 60s bash -c 'while true; do 
  ps -p $(pgrep edge-gateway) -o %cpu= | tr -d " "
  echo
  sleep 5
done' > cpu-usage.txt

echo "✅ Performance validation completed!"
```

## 调优实践

### 分步优化流程

#### 阶段1：基础优化
```bash
# 1. 启用发布模式编译
cargo build --release

# 2. 配置系统参数
echo 'net.core.somaxconn = 65535' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_max_syn_backlog = 65535' >> /etc/sysctl.conf
sysctl -p

# 3. 增加文件描述符限制
echo 'gateway soft nofile 65536' >> /etc/security/limits.conf
echo 'gateway hard nofile 65536' >> /etc/security/limits.conf
```

#### 阶段2：应用级优化
```yaml
# 在gateway.yml中调整配置
gateway:
  system:
    thread_pool_size: 16        # 根据CPU核心数调整
    max_connections: 2000       # 根据内存容量调整
    
drivers:
  modbus_plc1:
    config:
      max_batch_size: 200       # 增加批量处理大小
      polling_interval: "1s"    # 根据需求调整频率
      
connectors:
  mqtt:
    publish:
      batch_size: 1000          # 增加批量发送大小
      compression: "lz4"        # 启用压缩
```

### 性能调优检查清单

#### 系统级检查
- [ ] CPU频率是否已调至最高
- [ ] 内存交换分区是否已禁用
- [ ] 网络缓冲区是否已优化
- [ ] 文件描述符限制是否合理
- [ ] 内核参数是否已调优

#### 应用级检查
- [ ] 编译是否使用release模式
- [ ] 线程池大小是否合适
- [ ] 内存分配器是否已优化
- [ ] 数据结构选择是否合理
- [ ] 算法复杂度是否最优

#### 配置级检查
- [ ] 批处理大小是否合适
- [ ] 缓存策略是否有效
- [ ] 连接池配置是否优化
- [ ] 压缩算法是否启用
- [ ] 监控指标是否完整

### Cargo.toml性能优化配置

```toml
[profile.release]
opt-level = 3           # 最高优化级别
debug = false           # 关闭调试信息
lto = "fat"            # 启用链接时优化
codegen-units = 1      # 单一代码生成单元
panic = "abort"        # 使用abort而非unwind
strip = true           # 移除符号表

# 针对特定包的优化
[profile.dev.package.serde_json]
opt-level = 3

[profile.dev.package.regex]
opt-level = 3
```

### 性能监控命令集合

```bash
# 实时性能监控
watch -n 1 'curl -s http://localhost:9090/metrics | grep -E "(cpu|memory|datapoints)"'

# 网络连接监控
ss -tulpn | grep :8080

# 磁盘I/O监控
iostat -x 1 5

# 进程线程监控
ps -eLf | grep edge-gateway

# 火焰图生成（需要安装flamegraph）
cargo flamegraph --bin edge-gateway
```

## 相关文档

- [部署指南](./DEPLOYMENT_GUIDE.md)
- [配置指南](./CONFIGURATION.md)
- [Docker部署指南](./DOCKER_DEPLOYMENT.md)
- [故障排除](./TROUBLESHOOTING.md)