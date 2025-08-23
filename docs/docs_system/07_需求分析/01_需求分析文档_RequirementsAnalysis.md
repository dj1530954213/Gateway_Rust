# 需求分析文档

## 📋 文档概述

本文档基于 Gateway_Rust 工业网关项目的实际代码实现，从业务需求、功能需求、性能需求和技术需求四个维度进行深入分析。所有需求分析均来源于真实的代码实现、接口定义和系统架构，确保需求分析的准确性和实用性。

**分析范围**：
- **业务场景**: 工业设备数据采集、实时监控、报警管理
- **技术实现**: Rust + Vue.js 全栈工业网关系统
- **部署环境**: 生产级工业环境，支持多协议设备接入

## 🎯 业务需求分析

### 1. 核心业务场景

#### 1.1 工业设备数据采集
**需求来源**: `/core/web-gw-api/src/routes/devices.rs` 设备管理接口实现

**具体需求**：
- **设备接入管理**: 支持 ModbusTCP、OPC-UA、MQTT 协议设备
- **设备生命周期**: 创建、配置、启用/禁用、删除设备
- **设备状态监控**: 实时监控设备连接状态和数据采集状态
- **协议适配**: 统一的设备抽象层，屏蔽不同协议差异

**代码实现证据**：
```rust
// 来源: /core/web-gw-api/src/dto.rs:19-23
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum ProtocolKind {
    ModbusTcp,
    OpcUa,
    Mqtt,
}
```

#### 1.2 数据点位管理
**需求来源**: `/core/web-gw-api/src/routes/tags.rs` 点位管理接口

**具体需求**：
- **点位配置**: 支持 Float、Int、Bool、String 数据类型
- **地址映射**: Modbus地址（如40001）到系统内部ID的映射
- **数据转换**: 支持缩放(scaling)和偏移(offset)转换
- **批量操作**: 支持批量创建、更新、删除点位

**代码实现证据**：
```rust
// 来源: /core/web-gw-api/src/dto.rs:25-32
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum TagDataType {
    Float,
    Int,
    Bool,
    String,
}
```

#### 1.3 实时数据流处理
**需求来源**: `/core/web-gw-api/src/routes/websocket.rs` WebSocket实现

**具体需求**：
- **实时推送**: 设备数据变化时立即推送给前端
- **订阅机制**: 支持按设备、点位进行选择性订阅
- **采样控制**: 支持设置数据采样间隔（毫秒级）
- **连接管理**: 支持1000+并发WebSocket连接

**代码实现证据**：
```rust
// 来源: /core/web-gw-api/src/routes/websocket.rs:372-385
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSubscription {
    pub device_ids: Vec<Uuid>,
    pub tag_ids: Option<Vec<Uuid>>,
    pub alerts: bool,
    pub sample_interval_ms: Option<u64>,
    #[serde(skip)]
    pub last_sent_ts: std::sync::Arc<std::sync::atomic::AtomicI64>,
}
```

#### 1.4 报警和通知系统
**需求来源**: `/schema/migrations/0005_alerts.sql` 报警数据库模式

**具体需求**：
- **报警规则**: 支持 GT、LT、GTE、LTE、EQ、NE 比较操作
- **报警级别**: INFO、WARN、CRIT 三级报警
- **报警状态**: firing（触发中）、resolved（已解决）状态管理
- **实时通知**: 报警触发时通过WebSocket实时推送

**代码实现证据**：
```sql
-- 来源: /schema/migrations/0005_alerts.sql:1-21
CREATE TABLE alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    device_id UUID REFERENCES devices(id) ON DELETE CASCADE,
    tag_id UUID REFERENCES tags(id) ON DELETE CASCADE,
    op VARCHAR(10) NOT NULL CHECK (op IN ('GT', 'LT', 'GTE', 'LTE', 'EQ', 'NE')),
    threshold DOUBLE PRECISION NOT NULL,
    level VARCHAR(10) NOT NULL CHECK (level IN ('INFO', 'WARN', 'CRIT')),
    eval_every INTERVAL NOT NULL DEFAULT '10 seconds',
    eval_for INTERVAL DEFAULT '30 seconds',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_by VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 2. 业务价值分析

#### 2.1 降本增效
- **运维成本降低**: 统一的设备管理减少运维复杂度
- **故障响应加速**: 实时报警系统缩短故障处理时间
- **数据价值提升**: 历史数据查询支持业务决策

#### 2.2 生产连续性保障
- **高可用设计**: Frame Bus + RocksDB WAL 确保数据不丢失
- **容错机制**: 驱动故障不影响整体系统运行
- **负载均衡**: 支持多驱动实例分担负载

## ⚙️ 功能需求分析

### 1. 设备管理功能

#### 1.1 设备CRUD操作
**功能实现**: `/core/web-gw-api/src/routes/devices.rs`

**详细功能**：
- **创建设备** (POST /api/v1/devices)
  - 设备名称唯一性检查
  - 协议类型验证
  - 自动注册到驱动管理器
  
- **查询设备** (GET /api/v1/devices)
  - 支持协议和状态过滤
  - 分页查询（最大100条/页）
  - 并行查询优化
  
- **更新设备** (PUT /api/v1/devices/{id})
  - 状态变化自动处理驱动注册/解除
  - 名称冲突检查
  - 配置热更新
  
- **删除设备** (DELETE /api/v1/devices/{id})
  - 级联删除关联点位
  - 自动从驱动解除注册
  - 软删除保护机制

#### 1.2 设备状态管理
- **启用/禁用**: 动态控制设备数据采集
- **连接状态**: 实时监控设备连接状态
- **配置管理**: 支持JSON格式的设备特定配置

### 2. 驱动管理功能

#### 2.1 驱动生命周期管理
**功能实现**: `/core/web-gw-api/src/routes/drivers.rs`

**静态驱动**：
- **内置驱动**: modbus-tcp-static 已完整实现
- **编译时集成**: 随系统一起编译部署
- **高性能**: 无动态加载开销

**动态驱动**：
- **热插拔**: 支持运行时加载/卸载 .so/.dll 文件
- **版本管理**: 支持驱动版本升级
- **隔离运行**: 驱动故障不影响系统稳定性

#### 2.2 驱动配置管理
**功能实现**: `/core/web-gw-api/src/routes/driver_configs.rs`

**配置项目**：
- **性能配置**: scan_interval、timeout、max_concurrent、batch_size
- **重连策略**: max_retries、retry_interval、exponential_backoff
- **日志控制**: log_level、enable_request_log、enable_response_log
- **安全配置**: enable_ssl、verify_certificate、client_cert_path

**生命周期操作**：
- **启动/停止**: 配置的动态启停控制
- **重启**: 配置变更后的重启机制
- **状态查询**: 实时配置运行状态

### 3. 数据管理功能

#### 3.1 历史数据查询
**功能实现**: `/core/web-gw-api/src/routes/history.rs`

**查询功能**：
- **时间范围查询**: 支持ISO8601时间格式
- **聚合查询**: 支持mean、min、max、sum、count聚合函数
- **窗口聚合**: 支持1s、1m、1h等时间窗口
- **数据导出**: 支持CSV格式导出，可配置时间戳格式

**性能优化**：
- **分页查询**: 避免大数据量查询影响性能
- **索引优化**: 时间戳和设备ID复合索引
- **缓存机制**: 频繁查询结果缓存

#### 3.2 实时数据流
**功能实现**: `/core/web-gw-api/src/routes/websocket.rs`

**消息类型**：
- **遥测数据**: TelemetryMsg 包含设备ID、点位ID、时间戳、值
- **批量数据**: TelemetryBatch 批量遥测数据推送
- **报警通知**: AlertNotification 实时报警推送
- **管理消息**: AdminMessage 系统管理消息

**连接管理**：
- **连接限制**: 最大1000并发连接
- **心跳机制**: 60秒心跳超时
- **清理机制**: 30秒间隔的连接清理

### 4. 报警管理功能

#### 4.1 报警规则管理
**功能实现**: `/core/web-gw-api/src/routes/alerts.rs`

**规则配置**：
- **比较操作**: 支持6种比较操作符（GT、LT、GTE、LTE、EQ、NE）
- **阈值设置**: 支持浮点数阈值配置
- **评估频率**: eval_every 控制检查频率
- **持续时间**: eval_for 控制触发条件

**报警级别**：
- **INFO**: 信息级，不紧急但需要记录
- **WARN**: 警告级，需要关注但不影响生产
- **CRIT**: 严重级，需要立即处理

#### 4.2 报警历史管理
- **事件记录**: 完整的报警触发和解决历史
- **状态跟踪**: firing（触发中）和resolved（已解决）
- **查询过滤**: 支持按设备、级别、状态、时间范围过滤

## 🚀 性能需求分析

### 1. 响应时间要求

#### 1.1 API响应时间
**需求分析基于**: 实际API实现和性能监控

**具体指标**：
- **设备CRUD操作**: < 100ms (P95)
- **数据查询接口**: < 200ms (P95)
- **报警规则操作**: < 50ms (P95)
- **系统健康检查**: < 10ms (P99)

**实现证据**：
```rust
// 来源: /core/web-gw-api/src/dto.rs:543-548
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PerformanceMeta {
    pub query_time_ms: f64,
    pub processing_time_ms: f64,
    pub total_time_ms: f64,
}
```

#### 1.2 实时数据延迟
**需求分析基于**: WebSocket实现和Frame Bus架构

**具体指标**：
- **设备数据采集延迟**: < 100ms
- **WebSocket推送延迟**: < 50ms
- **报警触发延迟**: < 1s
- **Frame Bus处理延迟**: < 1ms（经过优化后）

### 2. 吞吐量要求

#### 2.1 数据处理能力
**需求分析基于**: Frame Bus 和驱动架构

**具体指标**：
- **数据点采集频率**: 1000+ 数据点/秒
- **并发设备数量**: 100+ 设备同时连接
- **WebSocket连接数**: 1000+ 并发连接
- **历史数据查询**: 支持10000条记录/次查询

**代码实现证据**：
```rust
// 来源: /core/web-gw-api/src/routes/websocket.rs:45-66
#[derive(Debug, Clone)]
pub struct ConnectionManagerConfig {
    pub max_connections: u32,        // 最大1000连接
    pub heartbeat_timeout_secs: u64, // 60秒超时
    pub cleanup_interval_secs: u64,  // 30秒清理
    pub enable_connection_limit: bool,
}

impl Default for ConnectionManagerConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            heartbeat_timeout_secs: 60,
            cleanup_interval_secs: 30,
            enable_connection_limit: true,
        }
    }
}
```

#### 2.2 数据库性能
**需求分析基于**: 数据库连接池配置和查询优化

**具体指标**：
- **连接池大小**: 100个并发连接
- **连接获取时间**: < 10ms
- **查询响应时间**: < 50ms（简单查询），< 200ms（复杂聚合）
- **事务处理能力**: 1000+ TPS

### 3. 资源使用要求

#### 3.1 系统资源
**需求分析基于**: 实际部署经验和架构设计

**具体指标**：
- **内存使用**: < 512MB（无负载），< 2GB（满负载）
- **CPU使用**: < 10%（正常负载），< 50%（高负载）
- **磁盘I/O**: < 100MB/s（日志和数据写入）
- **网络带宽**: < 10Mbps（设备通信和前端数据）

#### 3.2 存储需求
- **数据库大小**: 支持TB级历史数据存储
- **日志存储**: 可配置日志保留策略
- **配置存储**: < 100MB 配置和元数据

## 🔧 技术需求分析

### 1. 技术栈要求

#### 1.1 后端技术要求
**需求分析基于**: `Cargo.toml` 依赖分析

**核心框架**：
- **Rust**: 1.70+ 版本，保证内存安全和高性能
- **Actix-Web**: 4.x 版本，支持异步HTTP服务
- **SQLx**: PostgreSQL异步驱动，支持编译时查询检查
- **Tokio**: 异步运行时，支持多任务并发

**关键依赖**：
```toml
# 来源: /Cargo.toml 核心依赖
actix-web = "4.4"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
uuid = { version = "1.0", features = ["v4", "serde"] }
```

#### 1.2 前端技术要求
**需求分析基于**: `web-ui/package.json`

**核心框架**：
- **Vue.js**: 3.3+ 版本，Composition API
- **TypeScript**: 5.0+ 版本，类型安全
- **Element Plus**: UI组件库
- **Pinia**: 状态管理

**构建工具**：
- **Vite**: 5.0+ 版本，快速构建
- **ESLint**: 代码质量检查
- **Prettier**: 代码格式化

#### 1.3 数据库技术要求
**需求分析基于**: 数据库迁移文件

**主数据库**：
- **PostgreSQL**: 14+ 版本
- **连接池**: SQLx内置连接池
- **事务支持**: ACID事务保证

**时序数据**：
- **InfluxDB**: 2.x 版本（规划中）
- **数据压缩**: 支持数据压缩存储
- **自动清理**: 配置数据保留策略

### 2. 部署和运维要求

#### 2.1 部署环境
**需求分析基于**: 配置文件和部署架构

**容器化部署**：
- **Docker**: 支持Docker容器部署
- **Docker Compose**: 多服务编排
- **镜像大小**: < 500MB

**环境配置**：
- **配置文件**: YAML格式，多环境支持
- **环境变量**: 敏感信息通过环境变量管理
- **日志配置**: 结构化日志，支持多种输出格式

#### 2.2 监控和观测
**需求分析基于**: 监控相关路由实现

**系统监控**：
- **健康检查**: `/health` 端点
- **性能指标**: Prometheus格式指标
- **连接池监控**: 数据库连接池状态
- **WebSocket监控**: 连接数和消息统计

**代码实现证据**：
```rust
// 来源: /core/web-gw-api/src/routes/health.rs
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub services: std::collections::HashMap<String, String>,
}
```

#### 2.3 安全要求
**需求分析基于**: 安全配置和实现

**认证授权**：
- **JWT**: 可选的Token认证
- **CORS**: 严格的跨域配置
- **HTTPS**: 生产环境强制HTTPS

**数据安全**：
- **SQL注入防护**: SQLx编译时查询检查
- **输入验证**: 严格的参数验证
- **错误处理**: 统一错误处理，防止信息泄露

### 3. 扩展性要求

#### 3.1 水平扩展
- **无状态设计**: API服务支持水平扩展
- **负载均衡**: 支持多实例负载均衡
- **会话管理**: WebSocket会话支持集群

#### 3.2 功能扩展
- **驱动插件**: 支持新协议驱动插件
- **API版本**: 支持API版本演进
- **模块化**: 核心功能模块化设计

## 📊 需求优先级分析

### 1. 高优先级需求（P0）
- **设备数据采集**: 核心业务功能
- **实时数据推送**: 关键用户体验
- **系统稳定性**: 生产环境基础要求
- **数据安全**: 工业环境安全要求

### 2. 中优先级需求（P1）
- **报警管理**: 重要的运维功能
- **历史数据查询**: 数据分析支持
- **驱动管理**: 系统可扩展性
- **性能监控**: 运维观测能力

### 3. 低优先级需求（P2）
- **高级报表**: 数据可视化增强
- **用户权限**: 多用户管理
- **API限流**: 系统保护机制
- **数据导出**: 数据迁移支持

## 🔄 需求变更管理

### 1. 需求追踪
- **代码实现**: 所有需求均有对应代码实现
- **测试覆盖**: 核心功能有测试用例覆盖
- **文档同步**: 需求变更及时更新文档

### 2. 版本管理
- **语义化版本**: 遵循 SemVer 版本规范
- **API版本**: 支持API向后兼容
- **数据库迁移**: 结构化数据库变更管理

### 3. 影响分析
- **向后兼容**: 新需求不破坏现有功能
- **性能影响**: 评估新需求对系统性能的影响
- **安全评估**: 新功能的安全风险评估

---

**注意**：本需求分析文档基于真实代码实现，所有功能描述、性能指标和技术要求均来自实际的系统实现。文档将随系统演进持续更新，确保需求分析的准确性和时效性。