# 🛠️ Gateway Rust 开发调试启动指南

**文档状态**: 已移动到 `docs/development/debug-startup-guide.md`  
**原位置**: `DEBUG_STARTUP.md`  
**移动日期**: 2025-01-18  

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
rustc --version

# 4. 确认Node.js环境
node --version
npm --version
```

### 第二步：启动基础服务

```bash
# 启动Docker服务栈
docker-compose up -d emqx prometheus grafana influxdb

# 检查服务状态
docker ps
```

### 第三步：启动Rust后端

```bash
# 方式1：启动API服务
cargo run -p web-gw-api

# 方式2：启动完整网关
cargo run -p edge-gateway

# 方式3：使用脚本启动
scripts/development/start-dev.bat
```

### 第四步：启动前端开发服务器

```bash
# 进入前端目录
cd web-ui

# 安装依赖（首次运行）
npm install

# 启动开发服务器
npm run dev
```

## 🔍 调试技巧

### 后端调试
```bash
# 启用详细日志
export RUST_LOG=debug
cargo run -p web-gw-api

# 启用回溯
export RUST_BACKTRACE=1
cargo run -p web-gw-api

# 性能分析
cargo build --release
cargo run -p web-gw-api --release
```

### 前端调试
```bash
# 开发模式构建
npm run build:dev

# 类型检查
npm run type-check

# 代码检查
npm run lint

# 单元测试
npm run test:unit
```

### 数据库调试
```bash
# PostgreSQL连接测试
psql -h localhost -p 5432 -U postgres -d iot

# InfluxDB连接测试
curl http://localhost:50001/ping
```

## 🚨 常见问题排查

### 问题1：端口占用
```bash
# 检查端口占用
netstat -ano | findstr :50010
netstat -ano | findstr :50020

# 终止进程
taskkill /PID <进程ID> /F
```

### 问题2：Docker服务启动失败
```bash
# 清理容器
docker-compose down -v
docker system prune -f

# 重新启动
docker-compose up -d
```

### 问题3：Rust编译失败
```bash
# 清理构建缓存
cargo clean

# 更新依赖
cargo update

# 重新构建
cargo build
```

### 问题4：前端构建失败
```bash
# 清理node_modules
cd web-ui
rm -rf node_modules package-lock.json

# 重新安装
npm install

# 清理缓存
npm cache clean --force
```

## 📝 开发环境验证清单

- [ ] Docker服务正常运行
- [ ] Rust后端API响应正常 (http://localhost:50010/health)
- [ ] 前端界面可正常访问 (http://localhost:50020)
- [ ] 数据库连接正常
- [ ] WebSocket通信正常
- [ ] 实时数据采集正常

## 🔗 相关文档

- [API接口文档](../API.md)
- [部署指南](../DEPLOYMENT_GUIDE.md)
- [测试指南](../testing/)
- [维护手册](../maintenance/)

---
*此指南适用于开发和调试环境，生产环境部署请参考部署指南*