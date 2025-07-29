# 🛠️ Gateway Rust 开发调试启动指南

## 📋 端口分配（50000+）

| 服务类型 | 端口 | 说明 | 访问地址 |
|---------|------|------|---------|
| **第三方Docker服务** |
| InfluxDB 3.2-core | 50001 | 时序数据库 | http://localhost:50001 |
| Grafana | 50002 | 监控仪表板 | http://localhost:50002 |
| Prometheus | 50003 | 指标收集 | http://localhost:50003 |
| EMQX Dashboard | 50004 | MQTT管理界面 | http://localhost:50004 |
| MQTT Broker | 50005 | MQTT消息代理 | mqtt://localhost:50005 |
| EMQX WebSocket | 50006 | MQTT over WebSocket | ws://localhost:50006 |
| **控制台启动服务** |
| 后端REST API | 50010 | Gateway API | http://localhost:50010 |
| 后端Web管理 | 50011 | 管理界面后端 | http://localhost:50011 |
| 后端Metrics | 50012 | Prometheus指标 | http://localhost:50012 |
| 后端WebSocket | 50013 | 实时推送 | ws://localhost:50013 |
| 前端开发服务器 | 50020 | Vue开发服务器 | http://localhost:50020 |

## 🚀 详细启动步骤

### 第一步：环境准备

```bash
# 1. 确认环境
cd "C:\Program Files\Git\code\Gateway_Rust"

# 2. 确认Docker运行
docker --version
docker-compose --version

# 3. 确认Rust环境
cargo --version

# 4. 确认Node.js环境
node --version  # 需要18+
npm --version
```

### 第二步：启动第三方Docker服务

#### 方式A：使用一键脚本（推荐）
```bash
# 运行第三方依赖启动脚本
start-debug-deps.bat
```

#### 方式B：手动启动
```bash
# 停止冲突容器
docker stop emqx prometheus grafana influxdb 2>nul
docker rm emqx prometheus grafana influxdb 2>nul

# 启动第三方服务（仅Docker部分）
docker-compose up -d emqx prometheus grafana influxdb

# 验证服务状态
docker-compose ps
```

### 第三步：验证第三方服务

访问以下地址确认服务正常：

- **InfluxDB 3.2-core**: http://localhost:50001 
  - 用户: admin / password123
  - 组织: edge-gateway
  - 存储桶: iot-data

- **Grafana**: http://localhost:50002
  - 用户: admin / admin

- **Prometheus**: http://localhost:50003
  - 指标收集界面

- **EMQX Dashboard**: http://localhost:50004
  - 用户: admin / public

### 第四步：启动后端服务（控制台）

**新开终端窗口1**：

```bash
# 进入项目目录
cd "C:\Program Files\Git\code\Gateway_Rust"

# 方式A：使用开发调试配置启动
cargo run --bin edge-gateway -- --config config/dev-debug.yaml

# 方式B：如果没有edge-gateway二进制
cargo run -p web-gw-api

# 方式C：如果有其他主程序
cargo run --bin gateway-main
```

**预期输出**：
```
🚀 Gateway Rust starting...
📊 Metrics server: http://127.0.0.1:50012
🌐 REST API: http://127.0.0.1:50010  
🖥️  Web UI: http://127.0.0.1:50011
🔗 WebSocket: ws://127.0.0.1:50013
💾 InfluxDB: http://127.0.0.1:50001
📈 Prometheus: http://127.0.0.1:50003
Server running...
```

### 第五步：启动前端服务（控制台）

**新开终端窗口2**：

```bash
# 进入前端目录
cd "C:\Program Files\Git\code\Gateway_Rust\web-ui"

# 安装依赖（首次运行）
npm install

# 启动调试模式前端服务
npm run dev:debug
```

**预期输出**：
```
  VITE v4.x.x  ready in xxx ms

  ➜  Local:   http://localhost:50020/
  ➜  Network: http://192.168.x.x:50020/
  ➜  press h to show help
```

### 第六步：验证完整系统

#### 6.1 检查所有服务状态

```bash
# 检查Docker服务
docker-compose ps

# 检查端口占用
netstat -ano | findstr ":50001\|:50002\|:50003\|:50004\|:50005\|:50010\|:50020"

# 检查进程
tasklist | findstr "cargo\|node"
```

#### 6.2 访问系统界面

- **前端主界面**: http://localhost:50020
- **后端API文档**: http://localhost:50010/docs
- **后端健康检查**: http://localhost:50010/health
- **Prometheus指标**: http://localhost:50012/metrics

#### 6.3 测试API连接

```bash
# 测试后端API
curl http://localhost:50010/api/v1/health

# 测试WebSocket连接
# 可在浏览器控制台测试:
# const ws = new WebSocket('ws://localhost:50013/ws');
```

## 🔧 开发调试技巧

### 实时代码修改
- **后端**: Rust支持`cargo-watch`热重载
- **前端**: Vite自动热重载，修改代码自动刷新

### 日志查看
```bash
# Docker服务日志
docker-compose logs -f influxdb
docker-compose logs -f grafana

# 后端日志（控制台直接显示）
# 配置了debug级别日志

# 前端日志（浏览器控制台）
# F12 -> Console查看详细日志
```

### 调试端口说明
- 所有服务使用50000+端口，避免与其他开发环境冲突
- 前后端可独立重启，不影响第三方Docker服务
- 支持同时运行多个开发环境实例

## 🚨 故障排除

### 常见问题1：端口被占用
```bash
# 查看端口占用
netstat -ano | findstr ":50010"

# 终止占用进程
taskkill /PID <PID> /F
```

### 常见问题2：Docker服务启动失败
```bash
# 查看详细日志
docker-compose logs <service_name>

# 重启单个服务
docker-compose restart <service_name>
```

### 常见问题3：后端编译错误
```bash
# 跳过有问题的包编译
cargo run --bin edge-gateway --exclude pg-repo

# 或检查可用的二进制
cargo run --bin --list
```

### 常见问题4：前端连接后端失败
```bash
# 检查后端是否在50010端口启动
curl http://localhost:50010/health

# 检查vite代理配置
# 查看浏览器Network标签页的请求是否正确代理
```

### 问题5：第三方服务连接失败
```bash
# 测试InfluxDB连接
curl http://localhost:50001/ping

# 测试MQTT连接
# 使用MQTT客户端连接 mqtt://localhost:50005
```

## 📝 配置文件说明

- **config/dev-debug.yaml**: 后端调试配置
- **web-ui/.env.development.debug**: 前端调试环境变量
- **docker-compose.yml**: 第三方服务配置（已调整端口）
- **start-debug-deps.bat**: 一键启动第三方服务脚本

## 🎯 开发工作流

1. **启动基础服务**: `start-debug-deps.bat`
2. **开发后端**: 修改Rust代码 → Ctrl+C重启 → 测试API
3. **开发前端**: 修改Vue代码 → 自动热重载 → 浏览器刷新
4. **联调测试**: 前端操作 → 后端API → 第三方服务
5. **监控调试**: Grafana查看指标 → InfluxDB查看数据

## ✅ 成功标志

当看到以下情况时，说明开发环境启动成功：

- ✅ Docker服务: `docker-compose ps` 显示4个服务Up
- ✅ 后端服务: 控制台显示"Server running on http://127.0.0.1:50010"
- ✅ 前端服务: 控制台显示"Local: http://localhost:50020/"
- ✅ 页面访问: http://localhost:50020 可以正常打开
- ✅ API连接: 前端可以正常调用后端接口

**现在您可以开始高效的开发调试工作了！** 🎉