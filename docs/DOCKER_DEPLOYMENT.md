# Edge Gateway Docker 部署指南

本文档介绍如何使用Docker容器化部署工控物联网边缘网关。

## 快速开始

### 1. 环境要求

- Docker Engine 20.0+
- Docker Compose 2.0+
- 至少 4GB 内存
- 至少 10GB 磁盘空间

### 2. 克隆项目

```bash
git clone <your-repo-url>
cd Gateway_Rust
```

### 3. 开发环境部署

```bash
# 使用开发环境配置启动
docker-compose up -d

# 或使用部署脚本
./scripts/docker-deploy.sh --env dev
```

### 4. 生产环境部署

```bash
# 设置环境变量
export GRAFANA_ADMIN_PASSWORD="your-secure-password"
export INFLUXDB_ADMIN_PASSWORD="your-secure-password"

# 使用生产环境配置启动
docker-compose -f docker-compose.prod.yml up -d

# 或使用部署脚本
./scripts/docker-deploy.sh --env prod
```

## 服务架构

部署包含以下服务：

### 核心服务

- **edge-gateway**: 主网关服务
  - REST API: `http://localhost:8080`
  - Web管理界面: `http://localhost:8090`
  - Prometheus指标: `http://localhost:9090`

### 基础设施服务

- **emqx**: MQTT代理
  - MQTT端口: `1883`
  - WebSocket: `8083`
  - 管理界面: `http://localhost:18083`

- **prometheus**: 监控数据收集
  - 访问地址: `http://localhost:9091`

- **grafana**: 可视化仪表板
  - 访问地址: `http://localhost:3000`

- **influxdb**: 时序数据库 (可选)
  - 访问地址: `http://localhost:8086`

### 模拟器服务

- **plc-simulator**: PLC模拟器
  - Modbus TCP端口: `502`

## 配置文件

### 开发环境配置

使用 `config/docker-gateway.yml` 作为主配置文件：

- 简化的功能配置
- 调试级别日志
- 适度的资源限制
- 连接到容器化的依赖服务

### 生产环境配置

使用 `config/gateway.yml` 作为主配置文件：

- 完整的功能配置
- 生产级别日志
- 严格的资源限制
- 安全性配置

## 构建镜像

### 使用构建脚本

```bash
# 构建发布版本
./scripts/docker-build.sh

# 构建调试版本
./scripts/docker-build.sh --debug

# 构建并推送
./scripts/docker-build.sh --tag v1.0.0 --push
```

### 手动构建

```bash
# 发布版本
docker build -t edge-gateway:latest .

# 调试版本
docker build -f Dockerfile.debug -t edge-gateway:debug .
```

## 部署脚本

### docker-deploy.sh

智能部署脚本，支持多种环境：

```bash
# 开发环境
./scripts/docker-deploy.sh --env dev

# 生产环境
./scripts/docker-deploy.sh --env prod

# 重新构建并部署
./scripts/docker-deploy.sh --build

# 前台运行
./scripts/docker-deploy.sh --no-detach
```

## 数据持久化

### 开发环境

数据存储在命名卷中：
- `gateway-data`: 应用数据
- `gateway-logs`: 日志文件
- `emqx-data`: MQTT代理数据
- `prometheus-data`: 监控数据
- `grafana-data`: 仪表板配置

### 生产环境

推荐使用主机挂载：

```yaml
volumes:
  - /opt/edge-gateway/data:/app/data
  - /opt/edge-gateway/logs:/app/logs
  - /opt/edge-gateway/config:/app/config:ro
```

## 环境变量配置

### 网关服务

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `RUST_LOG` | `info` | 日志级别 |
| `GATEWAY_CONFIG` | `/app/config/docker-gateway.yml` | 配置文件路径 |
| `RUST_BACKTRACE` | `0` | 错误堆栈跟踪 |

### Grafana

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `GRAFANA_ADMIN_PASSWORD` | `admin` | 管理员密码 |
| `GRAFANA_SECRET_KEY` | - | 安全密钥 |

### InfluxDB

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `INFLUXDB_ADMIN_PASSWORD` | `password123` | 管理员密码 |

## 网络配置

所有服务运行在自定义网络 `gateway-net` 中：
- 子网: `172.20.0.0/16`
- 驱动: bridge
- 服务间可通过服务名互相访问

## 健康检查

### 网关服务健康检查

```bash
# 检查HTTP健康端点
curl -f http://localhost:8090/health

# 检查容器健康状态
docker-compose ps
```

### 服务状态监控

```bash
# 查看所有服务状态
docker-compose ps

# 查看服务日志
docker-compose logs -f edge-gateway

# 查看资源使用情况
docker stats
```

## 故障排除

### 常见问题

1. **端口冲突**
   ```bash
   # 检查端口占用
   netstat -tulpn | grep :8080
   
   # 修改端口映射
   vim docker-compose.yml
   ```

2. **权限问题**
   ```bash
   # 检查目录权限
   ls -la data/ logs/ config/
   
   # 修复权限
   sudo chown -R $USER:$USER data/ logs/
   ```

3. **内存不足**
   ```bash
   # 检查内存使用
   docker stats
   
   # 调整资源限制
   vim docker-compose.yml
   ```

### 日志调试

```bash
# 查看网关日志
docker-compose logs edge-gateway

# 实时跟踪日志
docker-compose logs -f edge-gateway

# 查看所有服务日志
docker-compose logs
```

### 性能调优

1. **资源限制**
   ```yaml
   deploy:
     resources:
       limits:
         memory: 512M
         cpus: '1.0'
   ```

2. **JVM调优** (如果使用Java组件)
   ```bash
   environment:
     - JAVA_OPTS=-Xmx512m -Xms256m
   ```

## 安全考虑

### 生产部署安全检查表

- [ ] 更改默认密码
- [ ] 启用TLS/SSL
- [ ] 配置防火墙规则
- [ ] 定期更新镜像
- [ ] 监控安全日志
- [ ] 备份重要数据

### 网络安全

```yaml
# 生产环境网络配置
networks:
  gateway-net:
    driver: bridge
    internal: true  # 隔离外部网络
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

### 访问控制

```yaml
# 限制容器权限
security_opt:
  - no-new-privileges:true
user: "1000:1000"
read_only: true
```

## 监控和维护

### Prometheus监控

监控指标包括：
- 系统资源使用率
- 应用性能指标
- 业务数据指标
- 错误率和响应时间

### 日志管理

```bash
# 配置日志轮转
docker-compose exec edge-gateway logrotate /etc/logrotate.conf

# 清理旧日志
find logs/ -name "*.log" -mtime +30 -delete
```

### 备份策略

```bash
# 备份配置
tar -czf backup/config-$(date +%Y%m%d).tar.gz config/

# 备份数据
tar -czf backup/data-$(date +%Y%m%d).tar.gz data/

# 备份数据库
docker-compose exec influxdb influx backup /backup/
```

## 附录

### A. 完整的docker-compose命令

```bash
# 启动所有服务
docker-compose up -d

# 停止所有服务
docker-compose down

# 重启特定服务
docker-compose restart edge-gateway

# 查看服务状态
docker-compose ps

# 扩展服务实例
docker-compose up -d --scale edge-gateway=3

# 更新服务
docker-compose pull && docker-compose up -d
```

### B. 配置文件模板

详细的配置文件模板请参考：
- `config/gateway.yml` - 完整配置模板
- `config/docker-gateway.yml` - Docker环境配置
- `examples/` - 各组件配置示例

### C. 相关文档

- [API文档](./API.md)
- [性能测试](./PERFORMANCE.md)
- [配置指南](./CONFIGURATION.md)