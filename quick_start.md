# 🚀 快速启动指南

## 当前运行状态

✅ **前端已启动**: http://localhost:3001

## 简化启动步骤

### 1. 前端界面已可用
访问: **http://localhost:3001**
- Vue.js 开发服务器已运行
- 所有前端功能界面可正常访问

### 2. 启动核心依赖服务（可选）
```bash
# 只启动核心依赖服务，跳过后端编译
cd "C:\Program Files\Git\code\Gateway_Rust"
docker-compose up -d emqx prometheus grafana influxdb plc-simulator
```

### 3. 手动启动 Rust 后端（开发模式）
```bash
cd "C:\Program Files\Git\code\Gateway_Rust"

# 检查是否已有编译好的版本
cargo check

# 如果需要编译（首次运行）
cargo build --release

# 启动后端服务
cargo run --bin edge-gateway
```

## 📱 立即体验

**现在就可以体验前端界面**:
1. 打开浏览器访问: http://localhost:3001
2. 查看完整的企业级前端界面
3. 测试各个功能模块：
   - 🏠 仪表板
   - 🔧 驱动管理  
   - 🔗 连接器管理
   - 📊 数据点管理
   - 📈 实时监控
   - ⚠️ 告警管理
   - ⚙️ 系统配置

## 🎯 核心访问地址

| 服务 | 地址 | 状态 |
|------|------|------|
| 前端界面 | http://localhost:3001 | ✅ 运行中 |
| 后端 API | http://localhost:8080 | ⏳ 需启动 |
| MQTT 服务 | http://localhost:18083 | ⏳ 需启动 |
| 监控面板 | http://localhost:3000 | ⏳ 需启动 |

## 🔧 故障排除

如果遇到问题：

1. **端口被占用**: 前端自动使用了 3001 端口
2. **后端连接失败**: 前端界面仍可正常浏览，只是数据为模拟数据
3. **Docker 构建慢**: 可以跳过，先体验前端界面

## 💡 建议

**现在最佳体验方式**:
1. ✅ 先浏览前端界面功能 (http://localhost:3001)
2. ⏳ 后台继续让 Docker 构建完成
3. 🔄 稍后启动完整后端服务

**您的工业 IoT 边缘网关项目前端已完全可用！** 🎉