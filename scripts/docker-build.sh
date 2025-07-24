#!/bin/bash
# Docker构建脚本

set -e

# 脚本目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# 默认参数
TAG="latest"
BUILD_TYPE="release"
PUSH=false

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--tag)
            TAG="$2"
            shift 2
            ;;
        --debug)
            BUILD_TYPE="debug"
            shift
            ;;
        --push)
            PUSH=true
            shift
            ;;
        -h|--help)
            echo "用法: $0 [选项]"
            echo "选项:"
            echo "  -t, --tag TAG     设置镜像标签 (默认: latest)"
            echo "  --debug           构建调试版本"
            echo "  --push            构建后推送到仓库"
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

echo "🔨 开始构建Edge Gateway Docker镜像..."
echo "📁 项目目录: $PROJECT_DIR"
echo "🏷️  镜像标签: edge-gateway:$TAG"
echo "🔧 构建类型: $BUILD_TYPE"

# 确保必要的目录存在
mkdir -p data logs config

# 构建Docker镜像
echo "🚀 执行Docker构建..."
if [[ "$BUILD_TYPE" == "debug" ]]; then
    # 调试版本的Dockerfile
    cat > Dockerfile.debug << 'EOF'
# 调试版本Dockerfile
FROM rust:1.76-slim as builder

# 安装依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# 构建调试版本
RUN cargo build --bin edge-gateway

# 运行时
FROM debian:bookworm-slim as runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false gateway
WORKDIR /app

# 复制调试版本
COPY --from=builder /app/target/debug/edge-gateway /usr/local/bin/edge-gateway
RUN mkdir -p /app/config /app/data /app/logs /app/models && \
    chown -R gateway:gateway /app

COPY config/*.yml /app/config/
COPY examples/*.yml /app/config/
COPY web/ /app/web/

USER gateway
EXPOSE 8080 8090 9090

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8090/health || exit 1

CMD ["edge-gateway", "--config", "/app/config/docker-gateway.yml"]
EOF
    docker build -f Dockerfile.debug -t edge-gateway:$TAG .
    rm Dockerfile.debug
else
    docker build -t edge-gateway:$TAG .
fi

echo "✅ Docker镜像构建完成: edge-gateway:$TAG"

# 显示镜像信息
echo "📊 镜像信息:"
docker images edge-gateway:$TAG

# 推送镜像(如果指定)
if [[ "$PUSH" == "true" ]]; then
    echo "📤 推送镜像到仓库..."
    docker push edge-gateway:$TAG
    echo "✅ 镜像推送完成"
fi

echo "🎉 构建完成！"
echo ""
echo "💡 使用方法:"
echo "  开发环境: docker-compose up"
echo "  生产环境: docker-compose -f docker-compose.prod.yml up"
echo "  自定义配置: docker run -v ./config:/app/config edge-gateway:$TAG"