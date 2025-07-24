# 集成测试指南

## 概述

本目录包含边缘网关的完整集成测试套件，包括端到端数据流测试、故障恢复测试和性能基准测试。

## 目录结构

```
tests/integration/
├── README.md                    # 本文件
├── Makefile                     # 测试环境管理
├── docker-compose.yml          # 测试服务定义
├── mod.rs                       # 模块声明
├── common/                      # 公共测试工具
│   ├── mod.rs
│   ├── test_env.rs             # 环境管理
│   ├── docker_helpers.rs       # Docker工具
│   └── assertions.rs           # 测试断言
├── config/                      # 服务配置文件
│   ├── mosquitto.conf          # MQTT配置
│   ├── prometheus.yml          # 监控配置
│   ├── influxdb-init.sh        # 数据库初始化
│   └── toxiproxy.json          # 故障注入配置
├── mock-plc/                    # Mock PLC模拟器
│   ├── modbus_simulator.py     # Python模拟器
│   └── requirements.txt        # Python依赖
├── test_config/                 # 测试配置
├── e2e/                         # 端到端测试
│   ├── mod.rs
│   ├── data_flow_tests.rs      # 数据流测试
│   └── config_tests.rs         # 配置测试
├── performance/                 # 性能测试
├── recovery/                    # 故障恢复测试
└── integration_tests.rs        # 测试入口
```

## 快速开始

### 1. 检查依赖

确保已安装以下工具：
- Docker (≥ 20.10)
- docker-compose (≥ 1.28)
- Rust (≥ 1.70)

### 2. 运行完整集成测试

```bash
# 使用脚本运行（推荐）
./scripts/run-integration-tests.sh

# 或使用Makefile
cd tests/integration
make test
```

### 3. 运行特定测试

```bash
# 运行端到端数据流测试
./scripts/run-integration-tests.sh run test_end_to_end_data_flow

# 运行配置热重载测试
cargo test --test integration_tests test_config_hot_reload -- --test-threads=1
```

### 4. 手动管理测试环境

```bash
cd tests/integration

# 启动测试环境
make setup

# 查看服务状态
make status

# 查看日志
make logs

# 清理环境
make teardown
```

## 测试服务

集成测试环境包含以下服务：

| 服务 | 端口 | 描述 |
|------|------|------|
| MQTT Broker | 1883, 9001 | Eclipse Mosquitto |
| InfluxDB | 8086 | 时序数据库 |
| Mock PLC | 1502, 8080 | Modbus TCP模拟器 |
| Redis | 6379 | 缓存数据库 |
| Grafana | 3000 | 监控面板 |
| Prometheus | 9090 | 指标收集 |
| Toxiproxy | 8474, 21883 | 故障注入代理 |

## Mock PLC 模拟器

Mock PLC提供以下功能：

### 数据模式

- **温度传感器** (40001-40010): 正弦波数据
- **压力传感器** (40011-40020): 随机游走
- **流量传感器** (40021-40030): 阶跃函数  
- **液位传感器** (40031-40040): 线性斜坡

### 故障注入API

```bash
# 注入10%错误率，500ms延迟
curl -X POST http://localhost:8080/ \
  -H "Content-Type: application/json" \
  -d '{"command": "inject_fault", "error_rate": 0.1, "response_delay": 0.5}'

# 清除故障
curl -X POST http://localhost:8080/ \
  -H "Content-Type: application/json" \
  -d '{"command": "clear_faults"}'

# 查看状态
curl http://localhost:8080/
```

## 测试类型

### 1. 端到端数据流测试

验证完整的数据链路：Mock PLC → Modbus → FrameBus → MQTT

```rust
#[tokio::test]
async fn test_end_to_end_data_flow() -> Result<()> {
    // 启动测试环境
    // 配置网关
    // 验证数据流
    // 检查准确性和延迟
}
```

### 2. 配置热重载测试

验证配置更新不会中断数据流：

```rust
#[tokio::test] 
async fn test_config_hot_reload() -> Result<()> {
    // 启动网关
    // 修改配置文件
    // 验证配置生效
    // 确保数据流无中断
}
```

### 3. 故障恢复测试

模拟各种故障场景：

- 网络断连和重连
- 设备故障模拟
- WAL持久化恢复
- 驱动崩溃重启

### 4. 性能基准测试

验证系统性能指标：

- 5000 fps数据处理能力
- 端到端延迟 < 100ms
- 内存使用稳定性
- CPU负载控制

## 测试配置

### 网关配置

测试环境使用以下配置文件：

- `test_config/endpoints.yml`: 端点定义
- `test_config/drivers.yml`: 驱动配置
- `test_config/variables.yml`: 变量映射

### 验证标准

| 指标 | 要求 | 描述 |
|------|------|------|
| 数据准确性 | ≥ 99% | 数据转换准确率 |
| 端到端延迟 | < 1s | PLC到MQTT的延迟 |
| 配置重载时间 | < 5s | 热重载响应时间 |
| 故障恢复时间 | < 30s | 网络断连后恢复时间 |

## 故障排查

### 常见问题

1. **Docker服务无法启动**
   ```bash
   # 检查Docker状态
   docker info
   
   # 查看服务日志
   make logs
   ```

2. **端口冲突**
   ```bash
   # 查看端口占用
   netstat -tlnp | grep :1883
   
   # 修改docker-compose.yml中的端口映射
   ```

3. **测试超时**
   ```bash
   # 增加环境变量中的超时时间
   export TEST_TIMEOUT=60
   ```

### 日志查看

```bash
# 查看所有服务日志
make logs

# 查看特定服务日志
make logs-mqtt
make logs-plc
make logs-influx

# 查看网关日志（如果已启动）
tail -f /tmp/gateway.log
```

## 开发指南

### 添加新测试

1. 在相应目录创建测试文件
2. 使用公共工具`TestEnvironment`
3. 遵循现有的测试模式
4. 添加适当的断言和验证

### 扩展Mock服务

1. 修改`mock-plc/modbus_simulator.py`
2. 添加新的数据模式或协议支持
3. 更新配置文件和文档

### 性能调优

- 调整`docker-compose.yml`中的资源限制
- 修改`config/prometheus.yml`的采样频率
- 优化测试并发度和超时设置

## CI/CD 集成

```yaml
# GitHub Actions示例
- name: Run Integration Tests
  run: |
    ./scripts/run-integration-tests.sh
  env:
    RUST_LOG: debug
```

## 贡献指南

1. 新测试必须通过现有CI流水线
2. 添加适当的文档和注释
3. 确保测试隔离性和可重复性
4. 遵循项目的代码风格规范

---

*最后更新: 2025-01-17*