# Gateway Rust 项目启动指南

## 前置条件

确保您的系统已安装：
- Docker Desktop 或 Docker Engine
- Docker Compose v2.0+
- Node.js 18+ (仅开发模式)
- Rust 1.70+ (仅开发模式)
- Git

## 1. 清理现有Docker环境

首先删除所有之前的镜像和容器：

```bash
# 停止所有容器
docker stop $(docker ps -aq)

# 删除所有容器
docker rm $(docker ps -aq)

# 删除所有镜像
docker rmi $(docker images -q) -f

# 删除所有volumes
docker volume prune -f

# 删除所有网络
docker network prune -f

# 完整清理系统
docker system prune -a --volumes -f
```

## 2. 项目启动方式

### 方式一：生产环境启动 (推荐)

```bash
# 进入项目目录
cd "C:\Program Files\Git\code\Gateway_Rust"

# 创建环境变量文件
cp .env.example .env.prod

# 编辑.env.prod文件，设置必要的密码
# INFLUXDB_ADMIN_PASSWORD=your_secure_password
# GRAFANA_ADMIN_PASSWORD=your_grafana_password
# GRAFANA_SECRET_KEY=your_secret_key

# 启动生产环境 (使用InfluxDB 3.2)
docker-compose -f docker-compose.prod.yml up -d
```

### 方式二：开发环境启动

```bash
# 启动基础服务 (使用InfluxDB 3.2)
docker-compose up -d

# 编译并运行后端服务 (另开终端)
cargo run -p edge-gateway

# 运行前端开发服务器 (另开终端)
cd web-ui
npm install
npm run dev
```

## 3. 服务端口说明

### 生产环境端口：
- **Edge Gateway API**: http://localhost:8080
- **Web管理界面**: http://localhost:8090
- **Grafana仪表板**: http://localhost:3000 (admin/admin_2024)
- **InfluxDB 3.2-core**: http://localhost:8086 (admin/influx_2024)
- **Prometheus**: http://localhost:9091
- **EMQX MQTT**: mqtt://localhost:1883
- **EMQX Dashboard**: http://localhost:18083 (admin/emqx_admin_2024)

### 开发环境端口：
- **Edge Gateway API**: http://localhost:20009
- **Web管理界面**: http://localhost:20010
- **InfluxDB 3.2-core**: http://localhost:20012 (admin/password123)
- **Grafana仪表板**: http://localhost:20008 (admin/admin)
- **Prometheus**: http://localhost:20007
- **EMQX MQTT**: mqtt://localhost:20002
- **EMQX Dashboard**: http://localhost:20006 (admin/public)
- **前端开发服务器**: http://localhost:5173 (仅开发模式)

## 4. 验证服务状态

```bash
# 检查所有容器状态
docker-compose ps

# 查看服务日志
docker-compose logs -f

# 检查特定服务日志
docker-compose logs influxdb
docker-compose logs edge-gateway
docker-compose logs grafana
```

## 5. 初始化配置

### InfluxDB 3.2 配置：
1. 访问 http://localhost:8086 (生产) 或 http://localhost:20012 (开发)
2. 使用管理员账户登录
3. 创建组织: `edge-gateway`
4. 创建存储桶: `iot-data`
5. 生成API Token用于服务连接

### Grafana 配置：
1. 访问 http://localhost:3000 (生产) 或 http://localhost:20008 (开发)
2. 首次登录使用 admin/admin 会要求修改密码
3. 添加InfluxDB数据源：
   - URL: http://influxdb:8086
   - Organization: edge-gateway
   - Token: (从InfluxDB获取)
   - Default Bucket: iot-data

### EMQX 配置：
1. 访问 http://localhost:18083 (生产) 或 http://localhost:20006 (开发)
2. 使用管理员账户登录
3. 配置MQTT客户端认证和授权规则

## 6. 故障排除

### 常见问题：

1. **端口冲突**：
   ```bash
   # 检查端口占用
   netstat -ano | findstr :8080
   # 停止占用端口的进程
   taskkill /PID <PID> /F
   ```

2. **InfluxDB启动失败**：
   ```bash
   # 删除InfluxDB数据卷重新初始化
   docker volume rm gateway_rust_influxdb-data
   docker-compose up -d influxdb
   ```

3. **内存不足**：
   ```bash
   # 检查Docker资源限制
   docker system df
   # 适当调整compose文件中的资源限制
   ```

4. **网络问题**：
   ```bash
   # 重建Docker网络
   docker network prune -f
   docker-compose down
   docker-compose up -d
   ```

## 7. 停止服务

```bash
# 停止所有服务
docker-compose down

# 停止并删除volumes
docker-compose down -v
```

## 8. 数据备份

```bash
# 备份InfluxDB数据
docker exec influxdb influxd backup /tmp/backup
docker cp influxdb:/tmp/backup ./backup/

# 备份Grafana配置
docker cp grafana:/var/lib/grafana ./backup/grafana/
```

## 关键配置文件

- `docker-compose.yml`: 开发环境配置
- `docker-compose.prod.yml`: 生产环境配置 (使用InfluxDB 3.2)
- `docker-compose.override.yml`: 开发环境覆盖配置
- `.env.prod`: 生产环境变量
- `config/`: 网关配置文件目录

## 注意事项

1. **InfluxDB版本**: 项目已升级为InfluxDB 3.2，具有更好的性能和新特性
2. **数据持久化**: 所有服务数据都通过Docker volumes持久化存储
3. **安全配置**: 生产环境请务必修改默认密码和密钥
4. **资源限制**: 生产环境已配置合理的CPU和内存限制
5. **健康检查**: 生产环境启用了服务健康检查

启动成功后，您可以通过Web管理界面 http://localhost:8090 (生产) 或 http://localhost:20010 (开发) 访问系统。