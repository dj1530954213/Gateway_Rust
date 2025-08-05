# Edge Gateway 部署指南

本文档提供工控物联网边缘网关的完整部署指南，涵盖不同环境和场景的部署方案。

## 目录
- [部署概述](#部署概述)
- [系统要求](#系统要求)
- [部署方式](#部署方式)
- [部署步骤](#部署步骤)
- [生产环境部署](#生产环境部署)
- [集群部署](#集群部署)
- [安全配置](#安全配置)
- [监控和维护](#监控和维护)
- [故障排除](#故障排除)

## 部署概述

Edge Gateway支持多种部署方式，适应不同的工业环境需求：

### 部署架构选择

| 部署方式 | 适用场景 | 优势 | 限制 |
|---------|---------|------|------|
| **单机部署** | 小型车间、设备较少 | 简单、成本低 | 单点故障风险 |
| **Docker容器** | 标准化环境、易扩展 | 部署快速、环境隔离 | 需要Docker知识 |
| **Kubernetes** | 大规模、高可用 | 自动扩展、故障恢复 | 复杂度高 |
| **边缘设备** | 远程站点、资源受限 | 本地处理、低延迟 | 资源限制 |

## 系统要求

### 硬件要求

#### 最小配置（开发/测试）
- **CPU**: 2核心 2.0GHz
- **内存**: 4GB RAM
- **存储**: 20GB可用空间
- **网络**: 100Mbps以太网

#### 推荐配置（生产环境）
- **CPU**: 4核心 2.5GHz或更高
- **内存**: 8GB RAM或更高
- **存储**: 100GB SSD存储
- **网络**: 1Gbps以太网
- **电源**: UPS不间断电源

#### 高性能配置（大规模部署）
- **CPU**: 8核心 3.0GHz或更高
- **内存**: 16GB RAM或更高
- **存储**: 500GB NVMe SSD
- **网络**: 双网卡绑定
- **存储**: RAID 1磁盘阵列

### 软件要求

#### 操作系统支持
- **Linux**: Ubuntu 20.04+, CentOS 8+, RHEL 8+
- **Windows**: Windows Server 2019+, Windows 10 Pro
- **容器**: Docker 20.0+, Kubernetes 1.20+

#### 依赖软件
```bash
# 必需组件
- Rust 1.76+ (仅源码编译时需要)
- OpenSSL 1.1+
- Protocol Buffers 3.0+

# 可选组件
- Docker & Docker Compose (容器部署)
- Nginx (反向代理)
- Prometheus (监控)
- Grafana (可视化)
```

## 部署方式

### 1. 二进制直接部署

#### 下载预编译版本
```bash
# 下载最新版本
wget https://github.com/yourorg/gateway-rust/releases/download/v1.0.0/edge-gateway-linux-x64.tar.gz

# 解压
tar -xzf edge-gateway-linux-x64.tar.gz
cd edge-gateway

# 设置权限
chmod +x edge-gateway
```

#### 从源码编译
```bash
# 克隆源码
git clone https://github.com/yourorg/gateway-rust.git
cd gateway-rust

# 安装Rust工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 编译发布版本
cargo build --release

# 安装到系统
sudo cp target/release/edge-gateway /usr/local/bin/
```

### 2. Docker容器部署

#### 单容器部署
```bash
# 拉取镜像
docker pull edge-gateway:latest

# 运行容器
docker run -d \
  --name edge-gateway \
  -p 8080:8080 \
  -p 8090:8090 \
  -p 9090:9090 \
  -v ./config:/app/config \
  -v ./data:/app/data \
  -v ./logs:/app/logs \
  edge-gateway:latest
```

#### Docker Compose部署
```bash
# 使用提供的compose文件
docker-compose up -d

# 或自定义compose
cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  edge-gateway:
    image: edge-gateway:latest
    container_name: edge-gateway
    restart: unless-stopped
    ports:
      - "8080:8080"
      - "8090:8090"
      - "9090:9090"
    volumes:
      - ./config:/app/config
      - ./data:/app/data
      - ./logs:/app/logs
    environment:
      - RUST_LOG=info
      - GATEWAY_CONFIG=/app/config/gateway.yml
    networks:
      - gateway-net

networks:
  gateway-net:
    driver: bridge
EOF

docker-compose up -d
```

### 3. 系统服务部署

#### 创建systemd服务（Linux）
```bash
# 创建服务用户
sudo useradd -r -s /bin/false gateway

# 创建服务目录
sudo mkdir -p /opt/edge-gateway/{bin,config,data,logs}
sudo chown -R gateway:gateway /opt/edge-gateway

# 复制二进制文件
sudo cp edge-gateway /opt/edge-gateway/bin/

# 创建systemd服务文件
sudo tee /etc/systemd/system/edge-gateway.service << 'EOF'
[Unit]
Description=Edge Gateway Service
After=network.target
Wants=network.target

[Service]
Type=simple
User=gateway
Group=gateway
WorkingDirectory=/opt/edge-gateway
ExecStart=/opt/edge-gateway/bin/edge-gateway --config /opt/edge-gateway/config/gateway.yml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=edge-gateway

# 环境变量
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

# 安全设置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/edge-gateway/data /opt/edge-gateway/logs

[Install]
WantedBy=multi-user.target
EOF

# 启用并启动服务
sudo systemctl daemon-reload
sudo systemctl enable edge-gateway
sudo systemctl start edge-gateway
```

#### 创建Windows服务
```powershell
# 使用NSSM创建Windows服务
# 1. 下载NSSM
# 2. 安装服务
nssm install EdgeGateway "C:\EdgeGateway\edge-gateway.exe"
nssm set EdgeGateway Parameters "--config C:\EdgeGateway\config\gateway.yml"
nssm set EdgeGateway DisplayName "Edge Gateway Service"
nssm set EdgeGateway Description "Industrial IoT Edge Gateway"
nssm set EdgeGateway Start SERVICE_AUTO_START

# 启动服务
nssm start EdgeGateway
```

## 部署步骤

### 步骤1: 环境准备

```bash
# 1. 创建部署目录
sudo mkdir -p /opt/edge-gateway/{bin,config,data,logs,models}

# 2. 创建服务用户
sudo useradd -r -s /bin/false -d /opt/edge-gateway gateway

# 3. 设置目录权限
sudo chown -R gateway:gateway /opt/edge-gateway

# 4. 安装系统依赖
# Ubuntu/Debian
sudo apt update
sudo apt install -y curl wget unzip openssl ca-certificates

# CentOS/RHEL
sudo yum install -y curl wget unzip openssl ca-certificates
```

### 步骤2: 安装网关

```bash
# 下载并解压
cd /tmp
wget https://github.com/yourorg/gateway-rust/releases/download/v1.0.0/edge-gateway-linux-x64.tar.gz
tar -xzf edge-gateway-linux-x64.tar.gz

# 安装二进制文件
sudo cp edge-gateway /opt/edge-gateway/bin/
sudo chmod +x /opt/edge-gateway/bin/edge-gateway

# 复制配置模板
sudo cp -r config/* /opt/edge-gateway/config/
sudo cp -r web /opt/edge-gateway/

# 设置权限
sudo chown -R gateway:gateway /opt/edge-gateway
```

### 步骤3: 配置网关

```bash
# 复制配置模板
sudo cp /opt/edge-gateway/config/templates/prod-gateway.yml /opt/edge-gateway/config/gateway.yml

# 编辑配置文件
sudo nano /opt/edge-gateway/config/gateway.yml
```

#### 基础配置示例
```yaml
gateway:
  id: "gateway-${HOSTNAME}"
  name: "生产线网关"
  location: "车间A"

services:
  api:
    host: "0.0.0.0"
    port: 8080
  web:
    host: "0.0.0.0"
    port: 8090
  metrics:
    host: "0.0.0.0"
    port: 9090

drivers:
  modbus_plc1:
    enabled: true
    type: "modbus-tcp"
    endpoint: "tcp://[REAL_PLC_IP]:502"
    config:
      unit_id: 1
      polling_interval: "2s"

connectors:
  mqtt:
    enabled: true
    connection:
      broker: "mqtt.company.com:1883"
      client_id: "gateway-${HOSTNAME}"
```

### 步骤4: 配置环境变量

```bash
# 创建环境变量文件
sudo tee /opt/edge-gateway/.env << 'EOF'
# 数据库配置
INFLUX_PASSWORD=your-secure-password
REDIS_PASSWORD=your-secure-password

# MQTT配置
MQTT_USERNAME=gateway
MQTT_PASSWORD=mqtt-password

# 安全配置
JWT_SECRET=your-32-byte-jwt-secret-key
ENCRYPTION_KEY=your-32-byte-encryption-key

# 可选配置
RUST_LOG=info
GATEWAY_MAX_MEMORY=2048
EOF

# 设置环境变量文件权限
sudo chmod 600 /opt/edge-gateway/.env
sudo chown gateway:gateway /opt/edge-gateway/.env
```

### 步骤5: 启动和验证

```bash
# 验证配置
sudo -u gateway /opt/edge-gateway/bin/edge-gateway --config /opt/edge-gateway/config/gateway.yml --validate

# 创建systemd服务（参见上文）
sudo systemctl start edge-gateway

# 检查服务状态
sudo systemctl status edge-gateway

# 检查日志
sudo journalctl -u edge-gateway -f

# 验证服务端点
curl http://localhost:8090/health
curl http://localhost:9090/metrics
```

## 生产环境部署

### 高可用架构

```mermaid
graph TB
    LB[负载均衡器] --> GW1[网关实例1]
    LB --> GW2[网关实例2]
    LB --> GW3[网关实例3]
    
    GW1 --> DB[(时序数据库集群)]
    GW2 --> DB
    GW3 --> DB
    
    GW1 --> MQ[MQTT集群]
    GW2 --> MQ
    GW3 --> MQ
    
    GW1 --> REDIS[(Redis集群)]
    GW2 --> REDIS
    GW3 --> REDIS
```

### 负载均衡配置

#### Nginx配置
```nginx
# /etc/nginx/sites-available/edge-gateway
upstream edge_gateway {
    least_conn;
    server [REAL_SERVER1_IP]:8080 max_fails=3 fail_timeout=30s;
    server [REAL_SERVER2_IP]:8080 max_fails=3 fail_timeout=30s;
    server [REAL_SERVER3_IP]:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    server_name gateway.company.com;
    
    # API代理
    location /api/ {
        proxy_pass http://edge_gateway;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # 健康检查
        proxy_next_upstream error timeout invalid_header http_500 http_502 http_503 http_504;
        proxy_connect_timeout 5s;
        proxy_send_timeout 10s;
        proxy_read_timeout 10s;
    }
    
    # WebSocket代理
    location /ws {
        proxy_pass http://edge_gateway;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
    
    # 健康检查端点
    location /health {
        access_log off;
        proxy_pass http://edge_gateway;
    }
}
```

### 数据库集群配置

#### InfluxDB集群
```yaml
# influxdb集群配置
version: '3.8'
services:
  influxdb-1:
    image: influxdb:2.7
    environment:
      - INFLUXDB_DB=gateway_data
      - INFLUXDB_ADMIN_USER=admin
      - INFLUXDB_ADMIN_PASSWORD=${INFLUX_PASSWORD}
    volumes:
      - influxdb1-data:/var/lib/influxdb
    ports:
      - "8086:8086"
      
  influxdb-2:
    image: influxdb:2.7
    environment:
      - INFLUXDB_DB=gateway_data
      - INFLUXDB_ADMIN_USER=admin
      - INFLUXDB_ADMIN_PASSWORD=${INFLUX_PASSWORD}
    volumes:
      - influxdb2-data:/var/lib/influxdb
    ports:
      - "8087:8086"

volumes:
  influxdb1-data:
  influxdb2-data:
```

## 集群部署

### Kubernetes部署

#### 命名空间
```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: edge-gateway
  labels:
    name: edge-gateway
```

#### ConfigMap
```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: gateway-config
  namespace: edge-gateway
data:
  gateway.yml: |
    gateway:
      id: "k8s-gateway"
      name: "Kubernetes网关"
    services:
      api:
        host: "0.0.0.0"
        port: 8080
      web:
        host: "0.0.0.0"
        port: 8090
      metrics:
        host: "0.0.0.0"
        port: 9090
    # ... 其他配置
```

#### Secret
```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: gateway-secrets
  namespace: edge-gateway
type: Opaque
data:
  jwt-secret: eW91ci0zMi1ieXRlLWp3dC1zZWNyZXQta2V5  # base64编码
  encryption-key: eW91ci0zMi1ieXRlLWVuY3J5cHRpb24ta2V5
  mqtt-password: bXF0dC1wYXNzd29yZA==
```

#### Deployment
```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: edge-gateway
  namespace: edge-gateway
spec:
  replicas: 3
  selector:
    matchLabels:
      app: edge-gateway
  template:
    metadata:
      labels:
        app: edge-gateway
    spec:
      containers:
      - name: edge-gateway
        image: edge-gateway:latest
        ports:
        - containerPort: 8080
        - containerPort: 8090
        - containerPort: 9090
        env:
        - name: RUST_LOG
          value: "info"
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: gateway-secrets
              key: jwt-secret
        volumeMounts:
        - name: config
          mountPath: /app/config
        - name: data
          mountPath: /app/data
        livenessProbe:
          httpGet:
            path: /health
            port: 8090
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8090
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
      volumes:
      - name: config
        configMap:
          name: gateway-config
      - name: data
        persistentVolumeClaim:
          claimName: gateway-data
```

#### Service
```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: edge-gateway-service
  namespace: edge-gateway
spec:
  selector:
    app: edge-gateway
  ports:
  - name: api
    port: 8080
    targetPort: 8080
  - name: web
    port: 8090
    targetPort: 8090
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP
```

#### Ingress
```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: edge-gateway-ingress
  namespace: edge-gateway
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: gateway.company.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: edge-gateway-service
            port:
              number: 8090
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: edge-gateway-service
            port:
              number: 8080
```

### 部署到Kubernetes
```bash
# 创建命名空间
kubectl apply -f namespace.yaml

# 创建ConfigMap和Secret
kubectl apply -f configmap.yaml
kubectl apply -f secret.yaml

# 创建PVC
kubectl apply -f pvc.yaml

# 部署应用
kubectl apply -f deployment.yaml

# 创建服务
kubectl apply -f service.yaml

# 创建Ingress
kubectl apply -f ingress.yaml

# 验证部署
kubectl get pods -n edge-gateway
kubectl get svc -n edge-gateway
```

## 安全配置

### TLS/SSL配置

#### 生成自签名证书
```bash
# 创建证书目录
mkdir -p /opt/edge-gateway/certs

# 生成私钥
openssl genrsa -out /opt/edge-gateway/certs/server.key 2048

# 生成证书签名请求
openssl req -new -key /opt/edge-gateway/certs/server.key -out /opt/edge-gateway/certs/server.csr

# 生成自签名证书
openssl x509 -req -days 365 -in /opt/edge-gateway/certs/server.csr -signkey /opt/edge-gateway/certs/server.key -out /opt/edge-gateway/certs/server.crt
```

#### 配置HTTPS
```yaml
# 在gateway.yml中配置TLS
services:
  api:
    host: "0.0.0.0"
    port: 8443
    tls:
      enabled: true
      cert_file: "/opt/edge-gateway/certs/server.crt"
      key_file: "/opt/edge-gateway/certs/server.key"
      
  web:
    host: "0.0.0.0"
    port: 8443
    tls:
      enabled: true
      cert_file: "/opt/edge-gateway/certs/server.crt"
      key_file: "/opt/edge-gateway/certs/server.key"
```

### 防火墙配置

#### UFW配置（Ubuntu）
```bash
# 启用UFW
sudo ufw enable

# 允许SSH
sudo ufw allow ssh

# 允许网关端口
sudo ufw allow 8080/tcp  # API
sudo ufw allow 8090/tcp  # Web UI
sudo ufw allow 9090/tcp  # Metrics

# 允许Modbus端口（如果需要）
sudo ufw allow 502/tcp

# 查看状态
sudo ufw status
```

#### iptables配置（CentOS/RHEL）
```bash
# 允许网关端口
sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 8090 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 9090 -j ACCEPT

# 保存规则
sudo iptables-save > /etc/iptables/rules.v4
```

### 访问控制

#### 配置RBAC
```yaml
# 在gateway.yml中配置基于角色的访问控制
security:
  rbac:
    enabled: true
    roles:
      admin:
        permissions:
          - "system:*"
          - "drivers:*"
          - "users:*"
      operator:
        permissions:
          - "drivers:read"
          - "datapoints:read"
          - "commands:execute"
      viewer:
        permissions:
          - "drivers:read"
          - "datapoints:read"
```

## 监控和维护

### Prometheus监控配置

#### prometheus.yml
```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'edge-gateway'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 5s
    metrics_path: /metrics
    
  - job_name: 'edge-gateway-cluster'
    static_configs:
      - targets:
        - '[REAL_SERVER1_IP]:9090'
        - '[REAL_SERVER2_IP]:9090'
        - '[REAL_SERVER3_IP]:9090'
```

### Grafana仪表板

#### 自动配置仪表板
```bash
# 下载预配置仪表板
wget https://raw.githubusercontent.com/yourorg/gateway-rust/main/monitoring/grafana/dashboard.json

# 导入到Grafana
curl -X POST \
  http://admin:admin@localhost:3000/api/dashboards/db \
  -H 'Content-Type: application/json' \
  -d @dashboard.json
```

### 日志管理

#### Logrotate配置
```bash
# 创建logrotate配置
sudo tee /etc/logrotate.d/edge-gateway << 'EOF'
/opt/edge-gateway/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 gateway gateway
    postrotate
        systemctl reload edge-gateway
    endscript
}
EOF
```

### 备份策略

#### 自动备份脚本
```bash
#!/bin/bash
# backup-gateway.sh

BACKUP_DIR="/backup/edge-gateway"
DATE=$(date +%Y%m%d_%H%M%S)

# 创建备份目录
mkdir -p "$BACKUP_DIR/$DATE"

# 备份配置
tar -czf "$BACKUP_DIR/$DATE/config.tar.gz" -C /opt/edge-gateway config/

# 备份数据
tar -czf "$BACKUP_DIR/$DATE/data.tar.gz" -C /opt/edge-gateway data/

# 备份数据库（如果使用本地数据库）
if command -v influx &> /dev/null; then
    influx backup "$BACKUP_DIR/$DATE/influxdb"
fi

# 清理旧备份（保留30天）
find "$BACKUP_DIR" -type d -mtime +30 -exec rm -rf {} +

echo "备份完成: $BACKUP_DIR/$DATE"
```

#### 设置定时备份
```bash
# 添加到crontab
echo "0 2 * * * /opt/edge-gateway/scripts/backup-gateway.sh" | crontab -
```

## 故障排除

### 常见问题和解决方案

#### 1. 服务启动失败
```bash
# 检查配置文件语法
edge-gateway --config /opt/edge-gateway/config/gateway.yml --validate

# 检查权限
ls -la /opt/edge-gateway/
sudo chown -R gateway:gateway /opt/edge-gateway/

# 检查端口占用
netstat -tulpn | grep :8080
```

#### 2. 驱动连接失败
```bash
# 测试网络连接
telnet [REAL_PLC_IP] 502

# 检查防火墙
sudo ufw status
sudo iptables -L

# 查看驱动日志
journalctl -u edge-gateway | grep -i modbus
```

#### 3. 内存使用过高
```bash
# 监控内存使用
top -p $(pgrep edge-gateway)

# 调整配置
vim /opt/edge-gateway/config/gateway.yml
# 修改 max_memory_mb 和 max_connections
```

#### 4. 数据库连接问题
```bash
# 测试数据库连接
curl http://localhost:8086/ping

# 检查数据库状态
docker ps | grep influx

# 重启数据库服务
docker-compose restart influxdb
```

### 性能调优

#### 系统级优化
```bash
# 增加文件描述符限制
echo "gateway soft nofile 65536" >> /etc/security/limits.conf
echo "gateway hard nofile 65536" >> /etc/security/limits.conf

# 调整内核参数
echo "net.core.somaxconn = 65535" >> /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 65535" >> /etc/sysctl.conf
sysctl -p
```

#### 应用级优化
```yaml
# 在gateway.yml中调整性能参数
gateway:
  system:
    thread_pool_size: 16        # 根据CPU核心数调整
    max_connections: 2000       # 根据内存容量调整
    max_memory_mb: 4096         # 根据可用内存调整

drivers:
  modbus_plc1:
    config:
      max_batch_size: 200       # 增加批量读取大小
      polling_interval: "1s"    # 调整轮询频率
      
connectors:
  mqtt:
    publish:
      batch_size: 500           # 增加批量发送大小
      batch_timeout: "2s"       # 调整批量超时
```

### 诊断工具

#### 内置诊断命令
```bash
# 检查系统状态
edge-gateway --status

# 测试驱动连接
edge-gateway --test-drivers

# 验证配置
edge-gateway --validate-config

# 生成诊断报告
edge-gateway --diagnose > gateway-diagnose.txt
```

#### 健康检查脚本
```bash
#!/bin/bash
# health-check.sh

# 检查服务状态
if ! systemctl is-active --quiet edge-gateway; then
    echo "ERROR: Edge Gateway service is not running"
    exit 1
fi

# 检查API端点
if ! curl -f -s http://localhost:8090/health > /dev/null; then
    echo "ERROR: Health endpoint is not responding"
    exit 1
fi

# 检查磁盘空间
DISK_USAGE=$(df /opt/edge-gateway | awk 'NR==2 {print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 80 ]; then
    echo "WARNING: Disk usage is ${DISK_USAGE}%"
fi

# 检查内存使用
MEMORY_USAGE=$(free | awk 'NR==2{printf "%.2f\n", $3*100/$2}')
if (( $(echo "$MEMORY_USAGE > 80" | bc -l) )); then
    echo "WARNING: Memory usage is ${MEMORY_USAGE}%"
fi

echo "All checks passed"
exit 0
```

## 相关文档

- [配置指南](./CONFIGURATION.md)
- [Docker部署指南](./DOCKER_DEPLOYMENT.md)
- [API文档](./API.md)
- [故障排除指南](./TROUBLESHOOTING.md)
- [性能优化指南](./PERFORMANCE.md)