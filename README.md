# 工控物联网边缘网关 - Industrial IoT Edge Gateway

![Status](https://img.shields.io/badge/Status-Production_Ready-green)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![Tests](https://img.shields.io/badge/Tests-Passing-green)
![Architecture](https://img.shields.io/badge/Architecture-Microservices-blue)

一个高性能、安全的工业物联网边缘网关，使用Rust语言实现，具备完整的生产级功能。

## 🎯 项目概述

本项目实现了一个功能完整的工业IoT边缘网关，涵盖从基础MVP功能到高级分析引擎的全套特性。支持动态驱动加载、云端命令下发、实时数据分析、机器学习推理等企业级功能。

## ✨ 核心特性

### 🏗️ 基础架构 (Steps 1-35: MVP-1 & MVP-2)
- ✅ **动态驱动加载**: 支持 .so/.dll 驱动热插拔
- ✅ **安全验证**: ed25519数字签名和权限管理
- ✅ **命令框架**: 双向命令和确认系统 (CmdFrame/CmdAckFrame)
- ✅ **数据持久化**: WAL (Write-Ahead Log) 保证数据可靠性
- ✅ **高性能**: Rust零拷贝优化，支持万级并发

### 🌐 Web管理和API (Steps 36-50)
- ✅ **REST API服务器**: 完整的RESTful API接口
- ✅ **Web管理界面**: 现代化响应式管理界面
- ✅ **实时监控**: 系统状态、驱动状态、性能指标
- ✅ **健康检查**: 多组件健康监控和告警
- ✅ **诊断工具**: 连接性、文件系统、内存、协议测试

### 🚀 高级功能 (Steps 51-70: 生产就绪特性)
- ✅ **生产级配置**: 环境变量、加密、验证支持
- ✅ **机器学习引擎**: 边缘ML推理，支持多种模型
- ✅ **实时分析引擎**: 异常检测、模式识别、数据聚合
- ✅ **边缘计算**: WASM运行时，自定义逻辑执行
- ✅ **数据管道**: 高吞吐数据处理和转换
- ✅ **时序数据**: 专业时序数据处理和存储
- ✅ **预测分析**: 预测建模和趋势分析
- ✅ **自动化控制**: 规则引擎和智能控制
- ✅ **性能优化**: 自动性能调优和资源管理

## 🏢 项目结构

```
├── core/                           # 核心框架模块
│   ├── endpoint-kit/               # 连接管理和熔断器
│   ├── frame-bus/                  # 数据帧总线和WAL
│   ├── driver-manager/             # 静态驱动管理
│   ├── dynamic-driver/             # 动态驱动加载
│   ├── config-manager/             # 配置管理
│   ├── metrics-server/             # 指标服务器
│   ├── rest-api/                   # REST API服务器和RBAC
│   ├── web-server/                 # Web管理界面
│   ├── monitoring/                 # 监控和诊断
│   ├── production-config/          # 生产级配置管理
│   └── protocol-bridge/            # 协议桥接插件系统
│       ├── opcua.rs               # OPC-UA Server桥接
│       ├── modbus.rs              # Modbus Slave桥接
│       └── bridge.rs              # 桥接抽象层
├── drivers/                        # 设备驱动
│   └── modbus-static/              # Modbus驱动实现
├── connectors/                     # 云端连接器
│   └── mqtt5/                      # MQTT5云端连接器
├── edge-gateway/                   # 主网关程序
├── web-ui/                         # Vue3前端界面
│   ├── src/                        # 前端源码
│   ├── dist/                       # 构建产物
│   └── package.json                # 前端依赖
├── dev/                            # 开发文件
│   ├── docs/                       # 开发文档
│   │   ├── quick_start.md          # 快速开始
│   │   ├── startup_guide.md        # 启动指南
│   │   └── ...                     # 其他开发文档
│   └── scripts/                    # 开发脚本
│       ├── build_all.ps1           # 构建脚本
│       ├── test_all.ps1            # 测试脚本
│       └── ...                     # 其他开发脚本
├── temp/                           # 临时文件
│   ├── mock-api.js                 # 模拟API
│   ├── test_*.js                   # 测试文件
│   └── ...                         # 其他临时文件
├── archive/                        # 归档文件
│   ├── STATUS.md                   # 历史状态
│   └── ...                         # 其他归档文档
├── tests/                          # 集成测试
├── tools/                          # 工具文件
│   └── debug/                      # 调试工具
├── scripts/                        # 系统脚本
├── config/                         # 配置文件
├── docs/                           # 正式文档
├── docker/                         # Docker配置
├── data/                           # 数据目录
│   └── wal/                        # WAL数据
├── examples/                       # 示例配置
└── .github/                        # CI/CD配置
    └── workflows/                  # GitHub Actions工作流
```

## 🚀 快速开始

### 前置要求

- Rust 1.70+
- Protocol Buffers编译器 (protoc)
- Windows/Linux/macOS

### 构建项目

```bash
# 完整构建
cargo build --release

# 运行测试
cargo test

# 完整系统测试
./test_complete_system.ps1  # Windows
./test_complete_system.sh   # Linux/macOS
```

### 启动网关

```bash
# 启动完整功能版本
cargo run --bin edge-gateway

# 使用自定义配置
cargo run --bin edge-gateway -- --config config/production.yaml

# 调试模式
cargo run --bin edge-gateway -- --debug
```

### 访问系统

启动后可以访问以下地址：

- 🌐 **Web管理界面**: http://127.0.0.1:8090
- 🔗 **REST API**: http://127.0.0.1:8080
- 📊 **监控指标**: http://127.0.0.1:9090/metrics

## 📊 功能模块详解

### 机器学习引擎
支持多种模型类型：
- 线性回归、逻辑回归
- 决策树、随机森林
- 神经网络、SVM
- 异常检测、时序预测

### 实时分析引擎
提供多种分析处理器：
- 移动平均计算
- 异常检测
- 模式识别
- 数据聚合

### 边缘计算运行时
基于WASM的自定义逻辑执行：
- 安全沙箱环境
- 资源限制管理
- 燃料计量机制
- 多模块支持

### 数据管道
高性能数据处理：
- 并行处理支持
- 可配置转换器
- 错误处理策略
- 批处理优化

## 🔧 配置管理

### 基本配置

```yaml
general:
  instance_id: "edge-gateway-001"
  environment: "production"
  enable_debug: false

network:
  rest_api_bind: "0.0.0.0:8080"
  web_ui_bind: "0.0.0.0:8090"
  metrics_bind: "0.0.0.0:9090"

security:
  jwt_secret: "${GATEWAY_JWT_SECRET}"
  enable_signature_verification: true
  enable_rate_limiting: true
```

### 环境变量支持

```bash
# 安全配置
export GATEWAY__SECURITY__JWT_SECRET="your-secret-key"
export GATEWAY__SECURITY__ENABLE_TLS=true

# 网络配置
export GATEWAY__NETWORK__REST_API_BIND="0.0.0.0:8080"

# 监控配置
export GATEWAY__MONITORING__ENABLE_ALERTS=true
```

## 🔒 安全特性

- **加密配置**: 敏感配置值自动加密存储
- **数字签名**: ed25519签名验证驱动完整性
- **权限控制**: 基于角色的访问控制
- **TLS支持**: 全链路TLS加密
- **速率限制**: API访问频率控制
- **审计日志**: 完整的操作审计记录

## 📈 监控和运维

### Prometheus指标
- 系统资源使用率
- 驱动运行状态
- API响应时间
- 错误率统计
- 业务指标

### 健康检查
- 组件健康状态
- 连接性测试
- 文件系统检查
- 内存使用监控

### 告警系统
- 多级告警分类
- 多种通知方式
- 告警聚合去重
- 自定义告警规则

## 🧪 测试覆盖

项目包含完整的测试套件：
- **单元测试**: 各模块功能测试
- **集成测试**: 模块间协作测试
- **性能测试**: 负载和压力测试
- **端到端测试**: 完整功能链路测试

```bash
# 运行所有测试
cargo test

# 性能基准测试
cargo bench

# 完整系统测试
./test_complete_system.ps1
```

## 📝 驱动开发

### 静态驱动示例

```rust
use driver_manager::{Driver, DriverMetadata};

pub struct CustomDriver {
    // Driver implementation
}

impl Driver for CustomDriver {
    async fn connect(&mut self) -> Result<()> {
        // Connection logic
    }
    
    async fn read_data(&self) -> Result<Vec<DataPoint>> {
        // Data reading logic
    }
}
```

### 动态驱动开发

详见 `drivers/` 目录下的示例驱动实现。

## 🌟 生产部署

### Docker部署

```dockerfile
FROM rust:1.70 as builder
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /target/release/edge-gateway /usr/local/bin/
CMD ["edge-gateway"]
```

### Kubernetes部署

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: edge-gateway
spec:
  replicas: 3
  selector:
    matchLabels:
      app: edge-gateway
  template:
    metadata:
      labels:
        app: edge-gateway
    spec:
      containers:
      - name: edge-gateway
        image: edge-gateway:latest
        ports:
        - containerPort: 8080
        - containerPort: 8090
```

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

本项目使用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🎉 项目状态

**✅ 项目完成状态: 100%**

- ✅ Steps 1-35: MVP-1 & MVP-2 基础功能
- ✅ Steps 36-50: REST API 和 Web 管理界面  
- ✅ Steps 51-70: 高级功能和生产就绪特性
- ✅ 完整的测试覆盖和文档

系统已完全准备好用于生产环境部署！