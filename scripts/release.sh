#!/bin/bash
# 版本发布脚本

set -e

VERSION=${1:-"0.1.0"}
REGISTRY=${2:-"ghcr.io/your-org"}

echo "🚀 开始发布 Edge Gateway v${VERSION}"

# 检查Git状态
if [[ -n $(git status --porcelain) ]]; then
    echo "❌ Git工作目录不干净，请先提交或储藏更改"
    exit 1
fi

# 检查是否在主分支
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$CURRENT_BRANCH" != "main" && "$CURRENT_BRANCH" != "master" ]]; then
    echo "⚠️  当前不在主分支 ($CURRENT_BRANCH)，确认继续？(y/N)"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "📝 更新版本号到 ${VERSION}"

# 更新Cargo.toml版本
sed -i "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml

# 提交版本更新
git add Cargo.toml
git commit -m "chore: bump version to ${VERSION}"

echo "🏷️ 创建Git标签"
git tag -a "v${VERSION}" -m "Release v${VERSION}"

echo "📦 构建发布版本"
cargo build --release

echo "🧪 运行测试"
cargo test --release

echo "🐳 构建Docker镜像"
docker build -t "${REGISTRY}/edge-gateway:${VERSION}" .
docker build -t "${REGISTRY}/edge-gateway:latest" .

echo "📋 生成发布说明"
cat > "RELEASE_NOTES_${VERSION}.md" << EOF
# Edge Gateway v${VERSION} 发布说明

## 🚀 新特性

### 核心架构
- ✅ 分层架构设计 (L0-L7)
- ✅ 异步事件驱动框架
- ✅ 插件化驱动系统
- ✅ 高性能数据总线

### 协议支持
- ✅ Modbus-TCP驱动 (功能码0x03/0x04)
- ✅ MQTT5上云连接器
- ✅ TCP/TLS/Serial连接抽象

### 数据处理
- ✅ 环形缓冲区+背压控制
- ✅ RocksDB WAL持久化
- ✅ 批量数据传输
- ✅ zstd数据压缩

### 配置管理
- ✅ 三表YAML配置
- ✅ 配置文件热重载
- ✅ 类型安全配置结构

### 监控观测
- ✅ Prometheus指标导出
- ✅ Grafana仪表板
- ✅ 结构化日志
- ✅ 健康检查端点

### 开发运维
- ✅ Docker容器化部署
- ✅ docker-compose开发环境
- ✅ PLC模拟器
- ✅ 性能测试工具

## 📊 性能指标

- **吞吐量**: 5,000+ fps 连续采集
- **延迟**: P99 < 150微秒
- **内存**: 稳态 < 100MB
- **可靠性**: QoS2精确一次传输

## 🛠️ 技术栈

- **语言**: Rust 1.76+
- **异步运行时**: Tokio
- **数据库**: RocksDB
- **消息队列**: MQTT5 (rumqttc)
- **监控**: Prometheus + Grafana
- **容器化**: Docker + Docker Compose

## 📁 项目结构

\`\`\`
Gateway_Rust/
├── core/                # 核心模块
├── drivers/             # 协议驱动
├── connectors/          # 北向连接器
├── docker/              # 容器配置
├── examples/            # 配置示例
└── scripts/             # 工具脚本
\`\`\`

## 🚀 快速开始

\`\`\`bash
# 1. 启动开发环境
./scripts/start-dev.sh

# 2. 复制配置文件
cp examples/*.yml data/config/

# 3. 启动网关
cargo run --bin edge-gateway

# 4. 访问监控
# - 指标: http://localhost:9090/metrics
# - Grafana: http://localhost:3000 (admin/admin)
# - EMQX: http://localhost:18083 (admin/public)
\`\`\`

## 📖 文档

- **README**: 项目概览和快速开始
- **API文档**: docs/API.md
- **性能测试**: docs/PERFORMANCE.md
- **配置示例**: examples/

## 🐛 已知问题

- TLS支持尚未完全实现
- 串口驱动暂时禁用
- 需要预安装protobuf编译器

## 🔮 后续计划

- OPC UA协议支持
- 规则引擎
- Web管理界面
- 集群部署支持

## 🙏 致谢

感谢所有为此项目贡献的开发者和测试人员！

---

完整更新日志请参见: [CHANGELOG.md](CHANGELOG.md)
EOF

echo "✅ 发布准备完成！"
echo ""
echo "📋 下一步操作："
echo "1. 推送代码和标签:"
echo "   git push origin main"
echo "   git push origin v${VERSION}"
echo ""
echo "2. 推送Docker镜像:"
echo "   docker push ${REGISTRY}/edge-gateway:${VERSION}"
echo "   docker push ${REGISTRY}/edge-gateway:latest"
echo ""
echo "3. 创建GitHub Release:"
echo "   使用 RELEASE_NOTES_${VERSION}.md 内容"
echo ""
echo "🎉 Edge Gateway v${VERSION} 发布完成！"