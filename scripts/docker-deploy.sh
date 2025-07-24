#!/bin/bash
# Docker部署脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# 默认参数
ENVIRONMENT="dev"
DETACH=true
BUILD=false

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--env)
            ENVIRONMENT="$2"
            shift 2
            ;;
        --no-detach)
            DETACH=false
            shift
            ;;
        --build)
            BUILD=true
            shift
            ;;
        -h|--help)
            echo "用法: $0 [选项]"
            echo "选项:"
            echo "  -e, --env ENV     设置环境 (dev/prod, 默认: dev)"
            echo "  --no-detach       前台运行，不分离"
            echo "  --build           部署前重新构建"
            echo "  -h, --help        显示此帮助信息"
            exit 0
            ;;
        *)
            echo "未知选项: $1"
            exit 1
            ;;
    esac
done

cd "$PROJECT_DIR"

echo "🚀 开始部署Edge Gateway..."
echo "📁 项目目录: $PROJECT_DIR"
echo "🌍 环境: $ENVIRONMENT"

# 检查必要文件
if [[ ! -f "docker-compose.yml" ]]; then
    echo "❌ 错误: 找不到 docker-compose.yml"
    exit 1
fi

# 创建必要的目录
echo "📁 创建必要目录..."
mkdir -p data logs config web drivers

# 检查配置文件
if [[ ! -f "config/docker-gateway.yml" ]]; then
    echo "⚠️  警告: 找不到 config/docker-gateway.yml，将使用默认配置"
fi

# 构建镜像(如果需要)
if [[ "$BUILD" == "true" ]]; then
    echo "🔨 重新构建镜像..."
    bash "$SCRIPT_DIR/docker-build.sh"
fi

# 根据环境选择compose文件
COMPOSE_FILES="-f docker-compose.yml"
if [[ "$ENVIRONMENT" == "prod" ]]; then
    if [[ ! -f "docker-compose.prod.yml" ]]; then
        echo "❌ 错误: 找不到 docker-compose.prod.yml"
        exit 1
    fi
    COMPOSE_FILES="-f docker-compose.prod.yml"
    echo "🏭 使用生产环境配置"
    
    # 检查生产环境必要的环境变量
    if [[ -z "$GRAFANA_ADMIN_PASSWORD" ]]; then
        echo "⚠️  警告: 未设置 GRAFANA_ADMIN_PASSWORD 环境变量"
    fi
    if [[ -z "$INFLUXDB_ADMIN_PASSWORD" ]]; then
        echo "⚠️  警告: 未设置 INFLUXDB_ADMIN_PASSWORD 环境变量"
    fi
else
    echo "🛠️  使用开发环境配置"
    if [[ -f "docker-compose.override.yml" ]]; then
        COMPOSE_FILES="$COMPOSE_FILES -f docker-compose.override.yml"
    fi
fi

# 停止已有服务
echo "🛑 停止现有服务..."
docker-compose $COMPOSE_FILES down --remove-orphans

# 启动服务
echo "▶️  启动服务..."
if [[ "$DETACH" == "true" ]]; then
    docker-compose $COMPOSE_FILES up -d
    
    echo "⏳ 等待服务启动..."
    sleep 10
    
    # 检查服务状态
    echo "📊 检查服务状态..."
    docker-compose $COMPOSE_FILES ps
    
    echo ""
    echo "✅ 部署完成！"
    echo ""
    echo "🌐 服务地址:"
    echo "  Web管理界面:    http://localhost:8090"
    echo "  REST API:      http://localhost:8080"
    echo "  Prometheus:    http://localhost:9091"
    echo "  Grafana:       http://localhost:3000"
    echo "  EMQX Dashboard: http://localhost:18083"
    
    if [[ "$ENVIRONMENT" == "dev" ]]; then
        echo ""
        echo "🛠️  开发环境账号:"
        echo "  Grafana:       admin/admin"
        echo "  EMQX:          admin/public"
        echo "  InfluxDB:      admin/password123"
    fi
    
    echo ""
    echo "💡 有用的命令:"
    echo "  查看日志: docker-compose $COMPOSE_FILES logs -f"
    echo "  停止服务: docker-compose $COMPOSE_FILES down"
    echo "  重启服务: docker-compose $COMPOSE_FILES restart"
    
else
    # 前台运行
    docker-compose $COMPOSE_FILES up
fi