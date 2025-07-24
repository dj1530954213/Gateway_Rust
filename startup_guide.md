# 🚀 工业 IoT 边缘网关完整启动指南

## 方法一：Docker 容器化启动（推荐⭐）

### 1. 启动所有服务（一键启动）
```bash
# 进入项目根目录
cd "C:\Program Files\Git\code\Gateway_Rust"

# 启动完整系统（包含前端、后端、数据库、监控）
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f edge-gateway
```

### 2. 访问各个服务
- 🌐 **前端界面**: http://localhost:8090
- 🔗 **REST API**: http://localhost:8080
- 📊 **监控指标**: http://localhost:9090
- 📈 **Grafana监控面板**: http://localhost:3000 (admin/admin)
- 📝 **EMQX MQTT管理**: http://localhost:18083 (admin/public)
- 📊 **Prometheus**: http://localhost:9091
- 💾 **InfluxDB**: http://localhost:8086

## 方法二：分别启动（开发模式）

### 后端启动
```bash
# 进入项目根目录
cd "C:\Program Files\Git\code\Gateway_Rust"

# 启动依赖服务（MQTT、监控等）
docker-compose up -d emqx prometheus grafana influxdb plc-simulator

# 编译并启动 Rust 后端
cargo run --bin edge-gateway

# 或使用预设的启动脚本
.\scripts\start-dev.sh
```

### 前端启动
```bash
# 进入前端目录
cd "C:\Program Files\Git\code\Gateway_Rust\web-ui"

# 安装依赖（如果之前没有安装）
npm install

# 启动开发服务器
npm run dev

# 或启动生产版本预览
npm run build
npm run preview
```

## 方法三：生产环境部署

### 1. 构建生产镜像
```bash
# 构建 Docker 镜像
docker-compose -f docker-compose.prod.yml build

# 启动生产环境
docker-compose -f docker-compose.prod.yml up -d
```

### 2. 手动编译部署
```bash
# 编译 Rust 后端（发布版本）
cargo build --release

# 构建前端（生产版本）
cd web-ui
npm run build

# 复制前端资源到后端静态文件目录
cp -r dist/* ../core/web-server/static/

# 启动后端
../target/release/edge-gateway --config config/prod-gateway.yml
```

## 🛠️ 常用管理命令

### Docker 管理
```bash
# 查看所有容器状态
docker-compose ps

# 重启特定服务
docker-compose restart edge-gateway

# 查看服务日志
docker-compose logs -f edge-gateway

# 停止所有服务
docker-compose down

# 完全清理（包括数据卷）
docker-compose down -v
```

### 开发调试
```bash
# 运行单元测试
cargo test

# 运行集成测试
cargo test --test integration_tests

# 验证配置文件
cargo run --bin edge-gateway -- validate

# 列出可用驱动
cargo run --bin edge-gateway -- list-drivers

# 启用调试日志
RUST_LOG=debug cargo run --bin edge-gateway
```

## 📋 系统检查清单

启动后请验证以下功能：

### ✅ 基础服务检查
- [ ] 前端界面可访问 (http://localhost:8090)
- [ ] REST API 响应正常 (http://localhost:8080/health)
- [ ] 监控指标正常 (http://localhost:9090/metrics)

### ✅ 功能模块检查
- [ ] 驱动管理：可以查看和管理驱动
- [ ] 连接器管理：可以创建和配置连接
- [ ] 数据点管理：可以配置数据采集点
- [ ] 实时监控：仪表板显示实时数据
- [ ] 告警系统：可以配置和查看告警

### ✅ 协议通信检查
- [ ] Modbus TCP: 连接到 PLC 模拟器 (localhost:502)
- [ ] MQTT5: 连接到 EMQX (localhost:1883)
- [ ] 数据传输：设备数据正常流转

## 🔧 故障排除

### 常见问题解决

1. **端口冲突**
   ```bash
   # 检查端口占用
   netstat -ano | findstr :8080
   netstat -ano | findstr :3000
   
   # 修改 docker-compose.yml 中的端口映射
   ```

2. **权限问题**
   ```bash
   # Windows 下确保 Docker 有足够权限
   # 以管理员身份运行 PowerShell
   ```

3. **配置文件错误**
   ```bash
   # 验证配置
   cargo run --bin edge-gateway -- validate
   
   # 查看示例配置
   cat config/gateway.yml
   ```

4. **依赖服务未启动**
   ```bash
   # 单独启动依赖服务
   docker-compose up -d emqx prometheus
   
   # 检查服务状态
   docker-compose ps
   ```

## 📱 快速体验

### 第一次启动推荐步骤：

1. **一键启动全部服务**
   ```bash
   cd "C:\Program Files\Git\code\Gateway_Rust"
   docker-compose up -d
   ```

2. **等待服务启动完成（约 30 秒）**
   ```bash
   docker-compose logs -f edge-gateway
   ```

3. **访问前端界面**
   - 打开浏览器访问: http://localhost:8090
   - 使用演示数据体验各个功能模块

4. **查看监控面板**
   - Grafana: http://localhost:3000 (admin/admin)
   - 查看实时系统指标和数据流

这样您就可以完整体验工业 IoT 边缘网关的所有功能！🎉