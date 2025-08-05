# CLAUDE.md

本文件为Claude Code提供在此项目中工作的指导规范。

## 🔧 构建和开发命令

### 环境设置
Windows开发环境所需环境变量：
```powershell
$env:PROTOC = "C:\tools\protoc\bin\protoc.exe"
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:PATH = $env:PATH + ";C:\tools\protoc\bin;C:\Program Files\LLVM\bin"
```

### 核心开发命令
```bash
# 构建整个工作空间
cargo build --workspace

# 运行所有测试
cargo test --workspace

# 启动主网关服务 (推荐)
cargo run -p edge-gateway
# 端口: REST API (50013), Web界面 (50014), 监控 (50015)

# 或单独运行API服务
cargo run -p web-gw-api  # 端口: 50010

# 前端开发
cd web-ui
npm install && npm run dev  # 端口: 50020
npm run build               # 生产构建
npm run lint && npm run type-check  # 代码检查
```

### 快速启动
1. **后端**: `cargo run -p edge-gateway`
2. **前端**: `cd web-ui && npm run dev`
3. **访问**: http://localhost:50020 (前端) → http://localhost:50013 (API)

## 🚫 生产级数据政策

**重要**：本系统已就绪交付客户，绝对不允许使用任何模拟/假数据：
- ✅ 所有Mock数据已清理完毕
- ✅ 必须连接真实PLC设备和真实IP地址 (192.168.x.x)
- ✅ 必须使用真实传感器数据和工业协议
- ❌ 禁止任何测试数据、模拟算法或localhost连接

## 📋 开发工作流程和规范

### 1. 深度思考要求
遇到以下复杂情况时必须使用深度思考：
- 新增/修改 ≥30 行代码
- 影响多个模块/层
- 涉及并发、异步、性能优化
- 前后端接口变更

**流程**：编写💭 Thoughts注释块 → 问题拆分 → 方案对比 → 异常处理 → 接口影响分析

### 2. 编译和测试要求
**每次代码修改后必须执行**：
```bash
# 后端检查
cargo check && cargo test --workspace

# 前端检查  
cd web-ui && npm run lint && npm run test:unit
```
- 无警告、无失败方可提交
- 新增public函数必须添加测试
- 禁止自动启动服务，仅编译和测试

### 3. MCP工具使用
对于需要操作其他工具的任务，优先检查MCP工具列表：
- 浏览器自动化测试：使用适当的MCP浏览器工具
- 外部系统集成：查找对应的MCP连接器
- 系统操作：选择合适的MCP系统工具

### 4. 目录整理要求
**每次修改代码后检查**：
- 文件是否放在正确的目录层级
- 模块依赖关系是否清晰
- 命名是否符合项目约定
- 必要时重新组织和分类文件结构

### 5. 弃用代码清理
**每次修改后必须检查**：
- 删除未使用的导入和变量
- 清理注释掉的代码块
- 移除废弃的函数和类型
- 确保不影响代码可读性

### 6. 任务步骤管理
使用 `tasks/` 文件夹管理开发任务：
- **backend/**: Rust后端开发任务
- **frontend/**: Vue.js前端开发任务  
- **database/**: 数据库相关任务
- **deployment/**: 部署运维任务
- **testing/**: 测试开发任务
- **documentation/**: 文档维护任务
- **monitoring/**: 监控观测任务
- **drivers/**: 驱动开发任务

每个任务文件包含：目标、步骤、验证标准、进度跟踪

## 🏗️ 系统架构概述

### 核心架构
**事件驱动微服务架构**，围绕中央**Frame Bus**构建，支持插件化驱动系统。

### 关键组件
- **Frame Bus** (`core/frame-bus`): 中央消息总线，RocksDB WAL持久化
- **驱动系统** (`core/driver-manager`): 静态/动态驱动加载，热插拔支持
- **Web API** (`core/web-gw-api`): Actix-Web REST API + WebSocket
- **协议桥接** (`core/protocol-bridge`): Modbus/OPC-UA协议转换
- **配置管理** (`core/config-manager`): 多层配置，热重载

### 数据流
```
[设备] → [驱动] → [Frame Bus] → [协议桥接] → [连接器] → [云端/MQTT]
                      ↓
              [Web API] ← [前端]
                      ↓
          [InfluxDB时序] + [PostgreSQL元数据]
```

### 技术栈
- **后端**: Rust + Actix-Web + SQLx + InfluxDB
- **前端**: Vue 3 + TypeScript + Element Plus + Pinia  
- **数据库**: PostgreSQL (元数据) + InfluxDB (时序数据)
- **监控**: Prometheus + Grafana + Loki

## 🔄 开发模式

### 新功能开发流程
1. **深度思考** → 设计方案
2. **领域模型** → 定义类型到 `models.rs`
3. **数据层** → 添加Repository trait + PG实现
4. **API层** → DTO + 路由处理 + OpenAPI
5. **前端** → 生成API客户端 + Pinia Store + Vue组件
6. **测试** → 单元测试 + 集成测试

### 代码规范
- 使用 `rustfmt` + `clippy` 
- 前端遵循 ESLint + TypeScript严格模式
- 所有public函数必须有文档注释
- 错误处理使用 `thiserror` + 自定义枚举

## ⚙️ 配置和部署

### 配置文件层级
```
config/
├── default.yaml    # 基础配置
├── dev.yaml        # 开发环境覆盖
└── prod.yaml       # 生产环境覆盖
```

### 环境变量
使用 `WEBGW_` 前缀覆盖YAML配置：
```bash
WEBGW_HTTP_ADDR=0.0.0.0:8080
WEBGW_PG_DSN=postgres://user:pass@localhost:5432/db
WEBGW_INFLUX_URL=http://localhost:8086
```

### 部署方式
```bash
# 开发环境
docker-compose up -d

# 生产环境
docker-compose -f docker-compose.prod.yml up -d

# 监控栈
docker-compose -f docker/compose.monitor.yml up -d
```

## 📊 监控和观测

### 指标收集
- **Prometheus**: `http://localhost:9090` 
- **Grafana**: `http://localhost:3000` (admin/admin)
- **API指标**: `/metrics` 端点

### 日志管理
- **结构化日志**: JSON格式，tracing框架
- **日志聚合**: Loki + Promtail
- **日志查询**: `{container="web-gw-api"} |= "error"`

### 健康检查
```bash
curl http://localhost:50013/health
```

## 🔗 相关文档

- `docs/API.md` - API接口文档
- `docs/DEPLOYMENT_GUIDE.md` - 部署指南
- `tasks/` - 开发任务管理
- `tests/integration/` - 集成测试环境

---

**重要提醒**：本系统为生产就绪系统，严禁使用任何模拟数据。所有开发和测试必须连接真实设备和真实数据源。
