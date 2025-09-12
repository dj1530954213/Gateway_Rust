# 多阶段构建Dockerfile for Edge Gateway

# 构建阶段
FROM rust:1.85-bullseye as builder

# 安装依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    build-essential \
    clang \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制项目文件
COPY Cargo.toml Cargo.lock ./
COPY core/ ./core/
COPY drivers/ ./drivers/
COPY connectors/ ./connectors/
COPY infra/ ./infra/
COPY edge-gateway/ ./edge-gateway/
COPY schema/ ./schema/

# 构建发布版本
RUN cargo build --release --bin edge-gateway

# 运行时阶段
FROM debian:bookworm-slim as runtime

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建应用用户
RUN useradd -r -s /bin/false gateway

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/edge-gateway /usr/local/bin/edge-gateway

# 创建配置和数据目录
RUN mkdir -p /app/config /app/data /app/logs && \
    chown -R gateway:gateway /app

# 复制配置文件（兼容 .yml 与 .yaml）
COPY config/*.yml /app/config/
COPY config/*.yaml /app/config/
COPY examples/*.yml /app/config/
COPY examples/*.yaml /app/config/

# 创建模型目录
RUN mkdir -p /app/models

# 切换到应用用户
USER gateway

# 暴露端口
EXPOSE 8080 8090 9090

# 健康检查（与 REST /healthz 对齐）
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/healthz || exit 1

# 启动命令（默认使用 dev.yaml，可通过 --config 覆盖）
CMD ["edge-gateway", "--config", "/app/config/dev.yaml"]