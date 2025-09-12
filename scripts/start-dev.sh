#!/bin/bash
# 开发环境启动脚本

set -e

echo "🚀 启动Edge Gateway开发环境"

# 检查Docker是否运行
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker未运行，请先启动Docker"
    exit 1
fi

# 检查docker-compose是否安装
if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose未安装"
    exit 1
fi

# 创建必要的目录
mkdir -p data/config
mkdir -p data/logs  
mkdir -p data/metrics

echo "📦 启动Docker服务..."

# 启动所有服务
docker-compose up -d

echo "⏳ 等待服务启动..."
sleep 10

# 检查服务状态
echo "📊 服务状态检查："
echo "- PLC模拟器: http://localhost:502 (Modbus TCP)"
echo "- EMQX: http://localhost:18083 (admin/public)"
echo "- Prometheus: http://localhost:50003"
echo "- Grafana: http://localhost:50002 (admin/admin)"
echo "- InfluxDB: http://localhost:8086"

echo ""
echo "🔧 开发环境配置："
echo "- 配置文件目录: ./data/config/"
echo "- 日志目录: ./data/logs/"
echo "- 网关指标端口: 9090"

echo ""
echo "✅ 开发环境已就绪！"
echo ""
echo "📝 下一步："
echo "1. 复制示例配置: cp examples/*.yml data/config/"
echo "2. 启动网关: cargo run --bin edge-gateway"
echo "3. 查看指标: http://localhost:9090/metrics"
echo "4. 查看仪表板: http://localhost:3000"