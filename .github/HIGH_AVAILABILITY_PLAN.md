# 🏗️ 高可用性架构改进计划

## 当前可用性评估 ⚠️ 不满足99.9%要求

### 关键风险点识别
1. **单点故障** - 所有组件单实例部署
2. **无故障转移** - 缺乏自动failover机制
3. **数据丢失风险** - 备份策略不完整
4. **恢复时间长** - RTO/RPO目标未达标

## 立即实施的高可用性改进

### 1. 数据库高可用架构 (本周实施)

#### PostgreSQL主从配置
```yaml
# kubernetes/postgresql-ha.yaml
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: postgres-cluster
spec:
  instances: 3
  
  postgresql:
    parameters:
      max_connections: "200"
      shared_buffers: "256MB"
      effective_cache_size: "1GB"
      
  bootstrap:
    initdb:
      database: iot
      owner: gateway_user
      
  storage:
    size: 100Gi
    storageClass: fast-ssd
    
  monitoring:
    enabled: true
    
  backup:
    retentionPolicy: "30d"
    barmanObjectStore:
      destinationPath: "s3://gateway-backups/postgres"
      s3Credentials:
        accessKeyId:
          name: backup-credentials
          key: ACCESS_KEY_ID
        secretAccessKey:
          name: backup-credentials  
          key: SECRET_ACCESS_KEY
```

#### 读写分离配置
```rust
// infra/pg-repo/src/pool.rs
pub struct DatabasePool {
    write_pool: Pool<Postgres>,
    read_pools: Vec<Pool<Postgres>>,
    current_read_index: AtomicUsize,
}

impl DatabasePool {
    pub async fn get_write_connection(&self) -> Result<PoolConnection<Postgres>> {
        self.write_pool.acquire().await
    }
    
    pub async fn get_read_connection(&self) -> Result<PoolConnection<Postgres>> {
        let index = self.current_read_index.fetch_add(1, Ordering::Relaxed);
        let pool = &self.read_pools[index % self.read_pools.len()];
        pool.acquire().await
    }
}
```

### 2. InfluxDB集群部署

#### 分布式InfluxDB配置
```yaml
# kubernetes/influxdb-cluster.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: influxdb-cluster
spec:
  serviceName: influxdb-headless
  replicas: 3
  template:
    spec:
      containers:
      - name: influxdb
        image: influxdb:2.7
        env:
        - name: INFLUXDB_CLUSTERING_ENABLED
          value: "true"
        - name: INFLUXDB_META_DIR
          value: /var/lib/influxdb/meta
        - name: INFLUXDB_DATA_DIR
          value: /var/lib/influxdb/data
        volumeMounts:
        - name: influxdb-storage
          mountPath: /var/lib/influxdb
  volumeClaimTemplates:
  - metadata:
      name: influxdb-storage
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 200Gi
```

### 3. 应用层高可用设计

#### 多副本部署策略
```yaml
# kubernetes/gateway-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: edge-gateway
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
      
  template:
    spec:
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchLabels:
                app: edge-gateway
            topologyKey: kubernetes.io/hostname
            
      containers:
      - name: gateway
        image: edge-gateway:latest
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
            
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 9090
          initialDelaySeconds: 10
          periodSeconds: 5
          failureThreshold: 3
          
        livenessProbe:
          httpGet:
            path: /health/live
            port: 9090
          initialDelaySeconds: 30
          periodSeconds: 10
          failureThreshold: 3
```

### 4. 负载均衡和服务发现

#### Ingress配置
```yaml
# kubernetes/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: gateway-ingress
  annotations:
    nginx.ingress.kubernetes.io/upstream-hash-by: "$request_uri"
    nginx.ingress.kubernetes.io/load-balancing: "round_robin"
    nginx.ingress.kubernetes.io/proxy-connect-timeout: "5"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "300"
spec:
  rules:
  - host: gateway.industrial.local
    http:
      paths:
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: edge-gateway-service
            port:
              number: 50013
```

### 5. 故障检测和自愈机制

#### 健康检查改进
```rust
// core/web-gw-api/src/health.rs
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub services: HashMap<String, ServiceHealth>,
    pub uptime: Duration,
}

#[derive(Debug, Serialize)]
pub struct ServiceHealth {
    pub status: String,
    pub response_time: Duration,
    pub last_error: Option<String>,
}

pub async fn comprehensive_health_check() -> Result<HealthStatus> {
    let mut services = HashMap::new();
    
    // 数据库健康检查
    services.insert("database".to_string(), check_database_health().await);
    
    // InfluxDB健康检查
    services.insert("influxdb".to_string(), check_influxdb_health().await);
    
    // MQTT连接检查
    services.insert("mqtt".to_string(), check_mqtt_health().await);
    
    // 外部依赖检查
    services.insert("external_apis".to_string(), check_external_deps().await);
    
    Ok(HealthStatus {
        status: determine_overall_status(&services),
        timestamp: Utc::now(),
        services,
        uptime: get_uptime(),
    })
}
```

### 6. 数据备份和恢复策略

#### 自动化备份脚本
```bash
#!/bin/bash
# backup-automation.sh

# PostgreSQL全量备份
kubectl exec -n industrial-gateway postgres-cluster-1 -- \
  pg_dump -U gateway_user -d iot | \
  gzip > "postgres-backup-$(date +%Y%m%d-%H%M%S).sql.gz"

# InfluxDB备份
kubectl exec -n industrial-gateway influxdb-0 -- \
  influx backup -portable /backup/influxdb-$(date +%Y%m%d)

# 配置文件备份
kubectl get secrets,configmaps -n industrial-gateway -o yaml > \
  "config-backup-$(date +%Y%m%d).yaml"

# 上传到对象存储
aws s3 cp postgres-backup-*.sql.gz s3://gateway-backups/postgres/
aws s3 cp /backup/influxdb-* s3://gateway-backups/influxdb/ --recursive
aws s3 cp config-backup-*.yaml s3://gateway-backups/configs/
```

### 7. 灾难恢复计划

#### RTO/RPO目标设定
- **RTO (恢复时间目标)**: 15分钟
- **RPO (恢复点目标)**: 5分钟
- **可用性目标**: 99.95%

#### 多区域部署策略
```yaml
# 主区域 (us-west-2a)
- 3个应用副本
- 主数据库 + 2个读副本
- Redis主节点

# 备用区域 (us-west-2b)  
- 2个应用副本
- 数据库只读副本
- Redis从节点

# 灾备区域 (us-east-1)
- 1个应用副本
- 数据库备份恢复实例
- 冷备份存储
```

## 实施优先级和时间表

| 优先级 | 改进项目 | 完成时间 | 影响 |
|-------|---------|---------|------|
| 🔴 P0 | PostgreSQL主从配置 | 本周 | 消除数据库单点故障 |
| 🔴 P0 | 应用多副本部署 | 本周 | 提升服务可用性 |
| 🟡 P1 | InfluxDB集群 | 下周 | 时序数据高可用 |
| 🟡 P1 | 负载均衡配置 | 下周 | 流量分发优化 |
| 🟢 P2 | 自动化备份 | 两周内 | 数据保护增强 |
| 🟢 P2 | 灾难恢复演练 | 三周内 | 验证恢复能力 |

## 监控和告警配置

### Prometheus告警规则
```yaml
# alerts.yml
groups:
- name: gateway.rules
  rules:
  - alert: GatewayDown
    expr: up{job="edge-gateway"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Gateway服务不可用"
      
  - alert: DatabaseConnectionHigh
    expr: pg_stat_database_numbackends > 150
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "数据库连接数过高"
```

此改进计划实施后，预计可达到**99.95%可用性**，满足工业级部署要求。