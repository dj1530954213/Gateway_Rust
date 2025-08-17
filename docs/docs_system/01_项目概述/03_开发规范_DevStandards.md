# 开发规范文档

## 概述

本文档定义了Gateway_Rust工业网关项目的开发规范，包括代码标准、项目结构、开发流程、质量保证等方面的要求。所有团队成员必须遵循这些规范，确保代码质量和项目的长期可维护性。

## 代码规范

### 1. Rust代码规范

#### 1.1 命名约定

```rust
// 模块名：snake_case
mod driver_manager;
mod frame_bus;

// 结构体和枚举：PascalCase
pub struct ModbusTcpClient;
pub enum ProtocolKind;

// 函数和变量：snake_case
fn read_holding_registers();
let device_config = DeviceConfig::new();

// 常量：SCREAMING_SNAKE_CASE
const MAX_REGISTER_COUNT: u16 = 125;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

// 生命周期参数：单个小写字母
fn process_data<'a>(data: &'a [u8]) -> &'a str;

// 泛型参数：单个大写字母或描述性PascalCase
fn convert<T: Serialize>(value: T) -> Result<String>;
fn handle_driver<D: Driver>(driver: D) -> DriverResult<()>;
```

#### 1.2 文档注释规范

```rust
/// 读取Modbus保持寄存器
/// 
/// # 参数
/// 
/// * `slave_id` - 从站ID (1-255)
/// * `address` - 起始地址 (0-65535)
/// * `count` - 寄存器数量 (1-125)
/// 
/// # 返回值
/// 
/// 成功时返回包含寄存器值的向量，失败时返回ModbusError
/// 
/// # 错误
/// 
/// * `ModbusError::InvalidParameter` - 参数超出有效范围
/// * `ModbusError::Timeout` - 响应超时
/// * `ModbusError::Exception` - Modbus异常响应
/// 
/// # 示例
/// 
/// ```rust
/// let mut client = ModbusTcpClient::new(config);
/// client.connect(&config).await?;
/// 
/// let registers = client.read_holding_registers(1, 0, 10).await?;
/// println!("读取到 {} 个寄存器", registers.len());
/// ```
pub async fn read_holding_registers(
    &mut self,
    slave_id: u8,
    address: u16,
    count: u16,
) -> ModbusResult<Vec<u16>> {
    // 实现...
}
```

#### 1.3 错误处理规范

```rust
// 使用thiserror定义结构化错误
#[derive(Error, Debug)]
pub enum DriverError {
    #[error("连接错误: {message}")]
    Connection { message: String },
    
    #[error("配置错误: {field} - {reason}")]
    Configuration { field: String, reason: String },
    
    #[error("超时错误: 操作耗时超过 {timeout}ms")]
    Timeout { timeout: u64 },
    
    #[error("IO错误")]
    Io(#[from] std::io::Error),
    
    #[error("序列化错误")]
    Serialization(#[from] serde_json::Error),
}

// 错误传播：优先使用?操作符
pub async fn process_request(request: Request) -> Result<Response> {
    let validated = validate_request(&request)?;
    let processed = transform_data(validated).await?;
    let response = build_response(processed)?;
    Ok(response)
}

// 错误上下文：提供有意义的错误信息
pub fn parse_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("无法读取配置文件: {}", path.display()))?;
    
    let config: Config = serde_yaml::from_str(&content)
        .with_context(|| format!("配置文件格式错误: {}", path.display()))?;
    
    Ok(config)
}
```

#### 1.4 异步编程规范

```rust
// 优先使用async/await而非手动Future构建
pub async fn fetch_device_data(device_id: Uuid) -> Result<DeviceData> {
    let device = device_repo.get_by_id(device_id).await?;
    let data = driver_manager.read_device_data(&device).await?;
    Ok(data)
}

// 使用tokio::spawn进行并发任务
pub async fn process_multiple_devices(device_ids: Vec<Uuid>) -> Vec<Result<DeviceData>> {
    let tasks = device_ids.into_iter().map(|id| {
        tokio::spawn(async move {
            fetch_device_data(id).await
        })
    });
    
    let results = futures::future::join_all(tasks).await;
    results.into_iter().map(|r| r.unwrap()).collect()
}

// 使用tokio::select!处理多个异步操作
pub async fn handle_request_with_timeout(request: Request) -> Result<Response> {
    tokio::select! {
        result = process_request(request) => result,
        _ = tokio::time::sleep(Duration::from_secs(30)) => {
            Err(Error::Timeout)
        }
    }
}

// 正确处理异步资源清理
pub async fn connect_and_process() -> Result<()> {
    let mut connection = Connection::new().await?;
    
    // 确保连接在函数结束时关闭
    defer! {
        let _ = connection.close().await;
    }
    
    process_with_connection(&mut connection).await
}
```

### 2. TypeScript代码规范

#### 2.1 Vue 3 Composition API规范

```typescript
// 组合式函数命名：use开头
export function useDeviceManager() {
  const devices = ref<Device[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  
  const fetchDevices = async (params?: DeviceQueryParams) => {
    try {
      loading.value = true
      error.value = null
      
      const response = await deviceApi.getDevices(params)
      devices.value = response.data
    } catch (err) {
      error.value = '获取设备列表失败'
      console.error('Failed to fetch devices:', err)
    } finally {
      loading.value = false
    }
  }
  
  return {
    devices: readonly(devices),
    loading: readonly(loading),
    error: readonly(error),
    fetchDevices
  }
}

// 组件定义：使用defineComponent和setup语法糖
<script setup lang="ts">
interface Props {
  deviceId: string
  autoRefresh?: boolean
}

interface Emits {
  (e: 'update', device: Device): void
  (e: 'error', error: string): void
}

const props = withDefaults(defineProps<Props>(), {
  autoRefresh: true
})

const emit = defineEmits<Emits>()

// 使用组合式函数
const { devices, loading, error, fetchDevices } = useDeviceManager()
const { isConnected, connect, disconnect } = useWebSocket()

// 计算属性
const currentDevice = computed(() => 
  devices.value.find(d => d.id === props.deviceId)
)

// 监听器
watch(() => props.deviceId, (newId) => {
  if (newId) {
    fetchDeviceDetails(newId)
  }
}, { immediate: true })

// 生命周期
onMounted(() => {
  if (props.autoRefresh) {
    startAutoRefresh()
  }
})

onUnmounted(() => {
  stopAutoRefresh()
})
</script>
```

#### 2.2 类型定义规范

```typescript
// 接口命名：使用描述性名称，避免I前缀
export interface DeviceConfiguration {
  readonly id: string
  readonly name: string
  readonly protocol: ProtocolType
  readonly endpoint: string
  readonly enabled: boolean
  readonly tags: TagConfiguration[]
  readonly metadata?: Record<string, unknown>
}

// 联合类型：使用字面量类型
export type ProtocolType = 'ModbusTcp' | 'ModbusRtu' | 'OpcUa' | 'Mqtt'
export type DataType = 'Boolean' | 'Int16' | 'UInt16' | 'Float32' | 'String'
export type AlertLevel = 'INFO' | 'WARNING' | 'CRITICAL' | 'FATAL'

// 泛型约束：使用有意义的约束
export interface Repository<T extends { id: string }> {
  getById(id: string): Promise<T | null>
  getAll(filter?: FilterOptions<T>): Promise<T[]>
  create(entity: Omit<T, 'id'>): Promise<T>
  update(id: string, updates: Partial<T>): Promise<T>
  delete(id: string): Promise<void>
}

// 实用类型：充分利用TypeScript内置实用类型
export type CreateDeviceRequest = Omit<Device, 'id' | 'createdAt' | 'updatedAt'>
export type UpdateDeviceRequest = Partial<Pick<Device, 'name' | 'description' | 'enabled'>>
export type DeviceListResponse = {
  readonly data: ReadonlyArray<Device>
  readonly pagination: PaginationInfo
}
```

## 项目结构规范

### 1. Rust项目结构

```
Gateway_Rust/
├── Cargo.toml                  # 工作空间配置
├── Cargo.lock                  # 依赖锁定文件
├── README.md                   # 项目说明
├── CLAUDE.md                   # Claude Code配置
├── .gitignore                  # Git忽略规则
├── 
├── core/                       # 核心模块
│   ├── frame-bus/              # 消息总线
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs          # 库入口
│   │   │   ├── error.rs        # 错误定义
│   │   │   ├── config.rs       # 配置结构
│   │   │   ├── bus.rs          # 总线实现
│   │   │   └── tests/          # 单元测试
│   │   └── benches/            # 性能测试
│   │
│   ├── driver-manager/         # 驱动管理
│   ├── web-gw-api/            # Web API
│   └── ...
│
├── infra/                      # 基础设施
│   ├── pg-repo/               # PostgreSQL仓库
│   └── ...
│
├── drivers/                    # 协议驱动
│   ├── modbus-static/         # Modbus驱动
│   └── ...
│
├── connectors/                 # 北向连接器
│   ├── mqtt5/                 # MQTT连接器
│   └── ...
│
├── config/                     # 配置文件
│   ├── default.yaml           # 默认配置
│   ├── dev.yaml              # 开发环境
│   └── prod.yaml             # 生产环境
│
├── scripts/                    # 构建脚本
├── docs/                      # 文档
├── tests/                     # 集成测试
└── target/                    # 构建输出
```

### 2. 前端项目结构

```
web-ui/
├── package.json               # NPM配置
├── vite.config.ts            # Vite配置
├── tsconfig.json             # TypeScript配置
├── .eslintrc.js              # ESLint配置
├── 
├── public/                    # 静态资源
│   ├── index.html
│   └── favicon.ico
│
├── src/
│   ├── main.ts               # 应用入口
│   ├── App.vue               # 根组件
│   │
│   ├── views/                # 页面组件
│   │   ├── devices/          # 设备管理页面
│   │   ├── monitoring/       # 监控页面
│   │   └── system/           # 系统管理页面
│   │
│   ├── components/           # 可复用组件
│   │   ├── charts/           # 图表组件
│   │   ├── forms/            # 表单组件
│   │   └── common/           # 通用组件
│   │
│   ├── composables/          # 组合式函数
│   │   ├── useWebSocket.ts   # WebSocket管理
│   │   ├── useDataQuery.ts   # 数据查询
│   │   └── useCache.ts       # 缓存管理
│   │
│   ├── stores/               # Pinia状态管理
│   │   ├── devices.ts        # 设备状态
│   │   ├── realtime.ts       # 实时数据
│   │   └── system.ts         # 系统状态
│   │
│   ├── api/                  # API客户端
│   │   ├── generated/        # 自动生成的客户端
│   │   ├── client.ts         # API客户端配置
│   │   └── types.ts          # API类型定义
│   │
│   ├── router/               # 路由配置
│   │   └── index.ts
│   │
│   ├── utils/                # 工具函数
│   │   ├── dataProcessing.ts # 数据处理
│   │   ├── timeUtils.ts      # 时间工具
│   │   └── validation.ts     # 验证工具
│   │
│   ├── styles/               # 样式文件
│   │   ├── main.css          # 主样式
│   │   ├── variables.css     # CSS变量
│   │   └── components.css    # 组件样式
│   │
│   └── types/                # 类型定义
│       ├── api.ts            # API类型
│       ├── components.ts     # 组件类型
│       └── global.ts         # 全局类型
│
├── tests/                    # 测试文件
│   ├── unit/                 # 单元测试
│   └── e2e/                  # 端到端测试
│
└── dist/                     # 构建输出
```

## 开发流程规范

### 1. Git工作流程

#### 1.1 分支策略

```bash
# 主分支
main                    # 生产环境代码
develop                 # 开发集成分支

# 功能分支
feature/device-management    # 新功能开发
feature/alert-system        # 新功能开发

# 修复分支
bugfix/modbus-connection-issue   # Bug修复
hotfix/security-patch           # 紧急修复

# 发布分支
release/v1.2.0              # 发布准备
```

#### 1.2 提交信息规范

```bash
# 提交信息格式
<type>(<scope>): <subject>

<body>

<footer>

# 类型（type）
feat:     新功能
fix:      Bug修复
docs:     文档更新
style:    代码格式调整
refactor: 代码重构
test:     测试相关
chore:    构建配置等

# 范围（scope）
frame-bus:      消息总线
driver-manager: 驱动管理
web-api:        Web API
frontend:       前端
docs:           文档

# 示例
feat(driver-manager): 添加Modbus RTU支持

实现了Modbus RTU协议的完整支持，包括：
- 串口通信管理
- CRC校验算法
- RTU帧格式处理
- 超时和重试机制

Closes #123
```

#### 1.3 代码审查规范

```yaml
# .github/pull_request_template.md
## 变更说明
- [ ] 新功能
- [ ] Bug修复
- [ ] 代码重构
- [ ] 文档更新
- [ ] 性能优化

## 测试确认
- [ ] 单元测试通过
- [ ] 集成测试通过
- [ ] 手动测试完成
- [ ] 性能测试通过（如适用）

## 检查清单
- [ ] 代码遵循项目规范
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] 没有硬编码的敏感信息
- [ ] 考虑了向后兼容性

## 影响范围
描述此次变更的影响范围和潜在风险...

## 测试计划
描述如何测试此次变更...
```

### 2. 代码质量检查

#### 2.1 Rust代码检查

```toml
# Cargo.toml - 开发依赖
[dev-dependencies]
tokio-test = "0.4"
mockall = "0.11"
criterion = "0.4"

# .cargo/config.toml - Clippy配置
[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"

# 禁用过于严格的检查
too_many_arguments = "allow"
module_name_repetitions = "allow"
```

```bash
# 代码检查命令
cargo fmt --all                    # 代码格式化
cargo clippy --all-targets --all-features -- -D warnings  # 静态检查
cargo test --all                   # 运行测试
cargo doc --no-deps               # 生成文档
cargo audit                       # 安全检查
```

#### 2.2 前端代码检查

```json
// .eslintrc.js
{
  "extends": [
    "@vue/eslint-config-typescript",
    "@vue/eslint-config-prettier"
  ],
  "rules": {
    "@typescript-eslint/no-unused-vars": "error",
    "@typescript-eslint/no-explicit-any": "warn",
    "vue/component-name-in-template-casing": ["error", "PascalCase"],
    "vue/component-definition-name-casing": ["error", "PascalCase"],
    "vue/multi-word-component-names": "error"
  }
}

// package.json
{
  "scripts": {
    "lint": "eslint src --ext .vue,.ts,.js",
    "lint:fix": "eslint src --ext .vue,.ts,.js --fix",
    "type-check": "vue-tsc --noEmit",
    "test:unit": "vitest",
    "test:e2e": "playwright test"
  }
}
```

### 3. 持续集成配置

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  rust_check:
    name: Rust Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
          
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Format check
        run: cargo fmt --all -- --check
        
      - name: Clippy check
        run: cargo clippy --all-targets --all-features -- -D warnings
        
      - name: Run tests
        run: cargo test --all
        
      - name: Security audit
        run: cargo audit

  frontend_check:
    name: Frontend Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
          cache-dependency-path: 'web-ui/package-lock.json'
          
      - name: Install dependencies
        working-directory: web-ui
        run: npm ci
        
      - name: Type check
        working-directory: web-ui
        run: npm run type-check
        
      - name: Lint check
        working-directory: web-ui
        run: npm run lint
        
      - name: Unit tests
        working-directory: web-ui
        run: npm run test:unit
        
      - name: Build
        working-directory: web-ui
        run: npm run build
```

## 测试规范

### 1. 单元测试规范

```rust
// 测试模块结构
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_device_connection_success() {
        // Given
        let config = create_test_config();
        let mut client = ModbusTcpClient::new(config.clone());
        
        // When
        let result = client.connect(&config).await;
        
        // Then
        assert!(result.is_ok());
        assert!(client.is_connected());
    }
    
    #[tokio::test]
    async fn test_device_connection_timeout() {
        // Given
        let mut config = create_test_config();
        config.endpoint = "192.168.999.999:502".to_string(); // 无效地址
        config.connect_timeout = Duration::from_millis(100);
        
        let mut client = ModbusTcpClient::new(config.clone());
        
        // When
        let result = client.connect(&config).await;
        
        // Then
        assert!(matches!(result, Err(ModbusError::ConnectionTimeout)));
        assert!(!client.is_connected());
    }
    
    fn create_test_config() -> ModbusConfig {
        ModbusConfig {
            endpoint: "127.0.0.1:502".to_string(),
            slave_id: 1,
            connect_timeout: Duration::from_secs(5),
            response_timeout: Duration::from_secs(3),
            max_retries: 3,
            retry_interval: Duration::from_millis(100),
            byte_order: ByteOrder::BigEndian,
            word_order: WordOrder::HighWordFirst,
            tcp_config: None,
            rtu_config: None,
        }
    }
}
```

### 2. 集成测试规范

```rust
// tests/integration/modbus_integration.rs
use gateway_rust::*;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_full_modbus_workflow() {
    // 需要Docker运行Modbus服务器
    if !is_test_environment_available().await {
        return;
    }
    
    let config = create_integration_config();
    let mut client = ModbusTcpClient::new(config.clone());
    
    // 测试连接
    client.connect(&config).await.unwrap();
    
    // 测试读取
    let registers = client.read_holding_registers(1, 0, 10).await.unwrap();
    assert_eq!(registers.len(), 10);
    
    // 测试写入
    client.write_single_register(1, 0, 1234).await.unwrap();
    
    // 验证写入
    let verification = client.read_holding_registers(1, 0, 1).await.unwrap();
    assert_eq!(verification[0], 1234);
    
    // 测试断开
    client.disconnect().await.unwrap();
}

async fn is_test_environment_available() -> bool {
    timeout(
        Duration::from_secs(1),
        TcpStream::connect("127.0.0.1:502")
    ).await.is_ok()
}
```

### 3. 性能测试规范

```rust
// benches/modbus_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gateway_rust::*;

fn benchmark_register_read(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("single_register_read", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_benchmark_config();
            let mut client = ModbusTcpClient::new(config.clone());
            client.connect(&config).await.unwrap();
            
            let result = client.read_holding_registers(1, 0, 1).await;
            black_box(result)
        });
    });
}

fn benchmark_batch_read(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("batch_register_read", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_benchmark_config();
            let mut client = ModbusTcpClient::new(config.clone());
            client.connect(&config).await.unwrap();
            
            let result = client.read_holding_registers(1, 0, 100).await;
            black_box(result)
        });
    });
}

criterion_group!(benches, benchmark_register_read, benchmark_batch_read);
criterion_main!(benches);
```

## 文档规范

### 1. 代码文档

```rust
//! # Gateway_Rust
//! 
//! 工业网关系统的Rust实现，提供高性能的工业协议通信和数据处理能力。
//! 
//! ## 功能特性
//! 
//! - 支持Modbus TCP/RTU协议
//! - 高性能消息总线
//! - 插件化驱动架构
//! - Web API和实时数据推送
//! 
//! ## 快速开始
//! 
//! ```rust
//! use gateway_rust::*;
//! 
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let config = Config::from_file("config/default.yaml")?;
//!     let gateway = Gateway::new(config).await?;
//!     gateway.start().await?;
//!     Ok(())
//! }
//! ```

/// 表示一个Modbus TCP客户端连接
/// 
/// 此结构体封装了与Modbus TCP设备的通信逻辑，提供异步API
/// 用于读写设备寄存器和线圈。
/// 
/// # 线程安全
/// 
/// 此类型是线程安全的，可以在多个异步任务之间共享。
/// 
/// # 示例
/// 
/// ```rust
/// use gateway_rust::modbus::*;
/// 
/// let config = ModbusConfig::builder()
///     .endpoint("192.168.1.100:502")
///     .slave_id(1)
///     .timeout(Duration::from_secs(5))
///     .build();
/// 
/// let mut client = ModbusTcpClient::new(config);
/// client.connect().await?;
/// 
/// let registers = client.read_holding_registers(0, 10).await?;
/// println!("读取到寄存器: {:?}", registers);
/// ```
pub struct ModbusTcpClient {
    // 字段文档...
}
```

### 2. API文档

```yaml
# 使用OpenAPI 3.0规范
openapi: 3.0.3
info:
  title: Gateway_Rust API
  description: 工业网关REST API接口
  version: 1.0.0
  contact:
    name: Gateway Team
    email: team@gateway.com

paths:
  /api/v1/devices:
    get:
      summary: 获取设备列表
      description: |
        获取系统中配置的所有设备列表，支持分页和过滤。
        
        ## 权限要求
        - 需要 `device:read` 权限
        
        ## 速率限制
        - 每分钟最多100次请求
      parameters:
        - name: page
          in: query
          description: 页码（从1开始）
          schema:
            type: integer
            minimum: 1
            default: 1
        - name: limit
          in: query
          description: 每页数量
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 20
      responses:
        '200':
          description: 成功返回设备列表
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/DeviceListResponse'
              example:
                data:
                  - id: "550e8400-e29b-41d4-a716-446655440000"
                    name: "PLC-001"
                    protocol: "ModbusTcp"
                    status: "Online"
                pagination:
                  page: 1
                  limit: 20
                  total: 156
                  total_pages: 8
```

## 安全规范

### 1. 安全编码

```rust
// 避免硬编码敏感信息
const DATABASE_URL: &str = env!("DATABASE_URL"); // ❌ 错误

fn get_database_url() -> Result<String> {
    std::env::var("DATABASE_URL")
        .or_else(|_| config::get_string("database.url"))
        .ok_or_else(|| Error::Configuration("DATABASE_URL not set".to_string()))
} // ✅ 正确

// 使用安全的随机数生成
use rand::Rng;
use uuid::Uuid;

fn generate_session_token() -> String {
    let mut rng = rand::thread_rng();
    (0..32).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect()
}

fn generate_device_id() -> Uuid {
    Uuid::new_v4() // 使用加密安全的随机数
}

// 输入验证和清理
fn validate_device_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::Validation("设备名称不能为空".to_string()));
    }
    
    if name.len() > 255 {
        return Err(Error::Validation("设备名称长度不能超过255字符".to_string()));
    }
    
    // 检查特殊字符
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(Error::Validation("设备名称只能包含字母、数字、连字符和下划线".to_string()));
    }
    
    Ok(())
}
```

### 2. 密码和认证

```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};

pub struct PasswordManager {
    argon2: Argon2<'static>,
}

impl PasswordManager {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }
    
    /// 哈希密码
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Error::PasswordHashing(e.to_string()))?;
        
        Ok(password_hash.to_string())
    }
    
    /// 验证密码
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| Error::PasswordVerification(e.to_string()))?;
            
        Ok(self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
```

## 部署规范

### 1. 容器化

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY core/ core/
COPY infra/ infra/
COPY drivers/ drivers/
COPY connectors/ connectors/

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/web-gw-api ./
COPY config/ config/

EXPOSE 50013
EXPOSE 50014
EXPOSE 50015

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:50013/health || exit 1

CMD ["./web-gw-api"]
```

### 2. 配置管理

```yaml
# config/prod.yaml
server:
  host: "0.0.0.0"
  port: 50013
  workers: 4

database:
  url: "${DATABASE_URL}"
  max_connections: 100
  min_connections: 20
  connect_timeout: 10
  idle_timeout: 600

security:
  jwt_secret: "${JWT_SECRET}"
  session_timeout: 3600
  max_login_attempts: 5
  lockout_duration: 900

logging:
  level: "info"
  format: "json"
  file: "/var/log/gateway/app.log"
  max_size: "100MB"
  max_files: 10
```

---

**文档版本**: v1.0  
**最后更新**: 2025-01-17  
**适用范围**: 所有Gateway_Rust开发团队成员  
**审核周期**: 每季度更新一次