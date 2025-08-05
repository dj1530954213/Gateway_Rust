# Edge Gateway 配置指南

本文档详细介绍工控物联网边缘网关的配置选项和最佳实践。

## 目录
- [配置文件结构](#配置文件结构)
- [核心配置](#核心配置)
- [驱动配置](#驱动配置)
- [连接器配置](#连接器配置)
- [高级功能配置](#高级功能配置)
- [环境变量](#环境变量)
- [配置模板](#配置模板)
- [配置验证](#配置验证)

## 配置文件结构

Edge Gateway使用YAML格式的配置文件，支持多级配置和环境变量覆盖。

### 主配置文件位置
- 默认配置: `config/gateway.yml`
- Docker环境: `config/docker-gateway.yml`
- 开发环境: `config/dev-gateway.yml`
- 生产环境: `config/prod-gateway.yml`

### 配置文件加载顺序
1. 默认配置文件
2. 环境特定配置
3. 环境变量覆盖
4. 命令行参数

## 核心配置

### 基础设置

```yaml
# 网关基础信息
gateway:
  id: "edge-gateway-001"
  name: "工控边缘网关"
  location: "生产车间A"
  description: "主要处理生产线数据采集和上云"
  
  # 系统配置
  system:
    log_level: "info"           # trace, debug, info, warn, error
    max_memory_mb: 1024        # 最大内存使用(MB)
    max_connections: 1000      # 最大并发连接数
    thread_pool_size: 8        # 线程池大小
    enable_metrics: true       # 启用Prometheus指标
    
  # 数据目录
  paths:
    data_dir: "./data"
    log_dir: "./logs"
    config_dir: "./config"
    temp_dir: "./temp"
    models_dir: "./models"
```

### 服务端口配置

```yaml
# 服务端口配置
services:
  # REST API服务
  api:
    enabled: true
    host: "0.0.0.0"
    port: 8080
    request_timeout: "30s"
    max_payload_size: "10MB"
    enable_compression: true
    enable_cors: true
    cors_origins: ["*"]
    
  # Web管理界面
  web:
    enabled: true
    host: "0.0.0.0"
    port: 8090
    static_path: "./web"
    enable_websocket: true
    websocket_path: "/ws"
    
  # Prometheus指标
  metrics:
    enabled: true
    host: "0.0.0.0"
    port: 9090
    path: "/metrics"
    
  # 健康检查
  health:
    enabled: true
    path: "/health"
    interval: "30s"
```

## 驱动配置

### Modbus TCP 驱动

```yaml
drivers:
  # Modbus TCP驱动示例
  modbus_line1:
    enabled: true
    type: "modbus-tcp"
    name: "生产线PLC1"
    endpoint: "tcp://[REAL_PLC_IP]:502"
    
    config:
      unit_id: 1
      polling_interval: "2s"     # 轮询间隔
      timeout: "5s"              # 连接超时
      retry_count: 3             # 重试次数
      retry_delay: "1s"          # 重试延迟
      max_batch_size: 100        # 批量读取大小
      
    # 数据点配置
    datapoints:
      - id: "temp_001"
        name: "温度传感器1"
        address: "40001"
        data_type: "float32"
        unit: "°C"
        scale: 0.1
        offset: 0
        min_value: -50.0
        max_value: 150.0
        tags:
          - "temperature"
          - "sensor"
          - "line1"
          
      - id: "pressure_001"
        name: "压力传感器1"
        address: "40002"
        data_type: "uint16"
        unit: "Pa"
        scale: 100
        offset: 0
        min_value: 0
        max_value: 10000
        tags:
          - "pressure"
          - "sensor"
          - "line1"
```

### Modbus RTU 驱动

```yaml
  modbus_line2:
    enabled: true
    type: "modbus-rtu"
    name: "生产线PLC2"
    endpoint: "serial:///dev/ttyUSB0"
    
    config:
      unit_id: 2
      baud_rate: 9600
      data_bits: 8
      stop_bits: 1
      parity: "none"
      polling_interval: "3s"
      timeout: "10s"
      
    datapoints:
      - id: "motor_speed_001"
        name: "电机转速1"
        address: "30001"
        data_type: "uint32"
        unit: "rpm"
        byte_order: "big_endian"
```

### OPC UA 驱动

```yaml
  opcua_scada:
    enabled: true
    type: "opcua"
    name: "SCADA系统"
    endpoint: "opc.tcp://[REAL_OPC_IP]:4840"
    
    config:
      security_policy: "None"
      security_mode: "None"
      session_timeout: "60s"
      subscription_interval: "1s"
      
      # 认证配置
      auth:
        type: "username"
        username: "opcuser"
        password: "${OPC_PASSWORD}"
        
    datapoints:
      - id: "production_count"
        name: "生产计数"
        node_id: "ns=2;s=ProductionCounter"
        data_type: "uint32"
```

## 连接器配置

### MQTT连接器

```yaml
connectors:
  mqtt:
    enabled: true
    type: "mqtt5"
    name: "云端MQTT"
    
    # 连接配置
    connection:
      broker: "emqx:1883"
      client_id: "edge-gateway-001"
      username: "gateway"
      password: "${MQTT_PASSWORD}"
      clean_session: true
      keep_alive: 60
      
    # TLS配置
    tls:
      enabled: false
      ca_cert: "./certs/ca.crt"
      client_cert: "./certs/client.crt"
      client_key: "./certs/client.key"
      insecure_skip_verify: false
      
    # 发布配置
    publish:
      topic_prefix: "industrial/gateway001"
      qos: 1
      retain: false
      batch_size: 100
      batch_timeout: "5s"
      compression: "gzip"
      
    # 数据映射
    data_mapping:
      # 数据点映射
      datapoints:
        topic_template: "data/{driver_id}/{point_id}"
        payload_format: "json"
        include_metadata: true
        
      # 告警映射
      alerts:
        topic: "alerts"
        payload_format: "json"
        
      # 系统状态映射
      system_status:
        topic: "status"
        interval: "30s"
```

### HTTP连接器

```yaml
  http_api:
    enabled: true
    type: "http"
    name: "企业API"
    
    config:
      base_url: "https://api.company.com/v1"
      timeout: "30s"
      max_retries: 3
      retry_delay: "5s"
      
      # 认证
      auth:
        type: "bearer"
        token: "${API_TOKEN}"
        
      # 请求头
      headers:
        Content-Type: "application/json"
        User-Agent: "EdgeGateway/1.0"
        
    # 数据发送配置
    endpoints:
      datapoints: "/datapoints"
      alerts: "/alerts"
      status: "/gateways/{gateway_id}/status"
```

## 高级功能配置

### 机器学习引擎

```yaml
ml_engine:
  enabled: true
  
  # 模型配置
  models:
    anomaly_detection:
      name: "异常检测模型"
      type: "onnx"
      path: "./models/anomaly_v1.onnx"
      enabled: true
      
      # 输入配置
      inputs:
        - datapoint: "temp_001"
          feature: "temperature"
        - datapoint: "pressure_001"
          feature: "pressure"
          
      # 输出配置
      outputs:
        - name: "anomaly_score"
          threshold: 0.8
          action: "alert"
          
      # 运行配置
      inference:
        interval: "10s"
        batch_size: 1
        
    predictive_maintenance:
      name: "预测性维护"
      type: "tensorflow"
      path: "./models/maintenance_v2.pb"
      enabled: false
```

### 数据分析

```yaml
analytics:
  enabled: true
  
  # 时间序列分析
  time_series:
    window_size: "1h"
    aggregation_functions:
      - "mean"
      - "max"
      - "min"
      - "std"
      
  # 趋势分析
  trend_analysis:
    enabled: true
    sensitivity: 0.1
    min_samples: 100
    
  # 异常检测
  anomaly_detection:
    enabled: true
    method: "isolation_forest"
    contamination: 0.1
```

### 数据存储

```yaml
storage:
  # WAL(Write-Ahead Log)配置
  wal:
    enabled: true
    path: "./data/wal"
    max_file_size: "100MB"
    max_files: 10
    sync_interval: "1s"
    
  # 时间序列数据库
  timeseries:
    enabled: true
    type: "influxdb"
    
    config:
      url: "http://influxdb:8086"
      database: "gateway_data"
      username: "gateway"
      password: "${INFLUX_PASSWORD}"
      retention_policy: "30d"
      batch_size: 1000
      flush_interval: "10s"
      
  # 本地缓存
  cache:
    enabled: true
    type: "redis"
    url: "redis://redis:6379"
    max_memory: "256MB"
    ttl: "1h"
```

## 环境变量

### 必需的环境变量

```bash
# 数据库密码
export INFLUX_PASSWORD="your-secure-password"
export REDIS_PASSWORD="your-secure-password"

# MQTT认证
export MQTT_PASSWORD="mqtt-password"

# API认证
export API_TOKEN="your-api-token"
export JWT_SECRET="your-jwt-secret"

# OPC UA认证
export OPC_PASSWORD="opc-password"

# 加密密钥
export ENCRYPTION_KEY="32-byte-encryption-key"
```

### 可选的环境变量

```bash
# 日志级别覆盖
export RUST_LOG="info"
export GATEWAY_LOG_LEVEL="debug"

# 性能调优
export GATEWAY_MAX_MEMORY="2048"
export GATEWAY_THREAD_POOL_SIZE="16"

# 功能开关
export ENABLE_ML_ENGINE="true"
export ENABLE_ANALYTICS="true"
export ENABLE_WEB_UI="true"

# 开发模式
export DEVELOPMENT_MODE="false"
export DEBUG_MODE="false"
```

## 配置模板

### 开发环境模板

```yaml
# config/templates/dev-gateway.yml
gateway:
  id: "dev-gateway"
  name: "开发环境网关"
  
  system:
    log_level: "debug"
    max_memory_mb: 512
    enable_metrics: true

services:
  api:
    host: "0.0.0.0"
    port: 8080
    enable_cors: true
    
  web:
    host: "0.0.0.0"
    port: 8090
    
  metrics:
    host: "0.0.0.0"
    port: 9090

# 简化的驱动配置用于开发测试
drivers:
  simulator:
    enabled: true
    type: "simulator"
    name: "数据模拟器"
    
    config:
      data_rate: "1s"
      point_count: 10

connectors:
  mqtt:
    enabled: true
    connection:
      broker: "localhost:1883"
      client_id: "dev-gateway"
      
ml_engine:
  enabled: false
  
analytics:
  enabled: false
```

### 生产环境模板

```yaml
# config/templates/prod-gateway.yml
gateway:
  id: "${GATEWAY_ID}"
  name: "${GATEWAY_NAME}"
  location: "${GATEWAY_LOCATION}"
  
  system:
    log_level: "info"
    max_memory_mb: 2048
    max_connections: 2000
    enable_metrics: true

services:
  api:
    host: "0.0.0.0"
    port: 8080
    enable_cors: false
    cors_origins: ["https://dashboard.company.com"]
    
  web:
    host: "127.0.0.1"  # 仅本地访问
    port: 8090
    
  metrics:
    host: "127.0.0.1"  # 仅本地访问
    port: 9090

# 完整的驱动配置
drivers: !include drivers.yml

# 生产级连接器配置
connectors: !include connectors.yml

# 启用所有高级功能
ml_engine: !include ml-config.yml
analytics: !include analytics-config.yml
storage: !include storage-config.yml

# 安全配置
security:
  jwt_secret: "${JWT_SECRET}"
  encryption_key: "${ENCRYPTION_KEY}"
  enable_rbac: true
  
# 监控配置
monitoring:
  enabled: true
  alerts: !include alerts-config.yml
```

### 边缘计算模板

```yaml
# config/templates/edge-gateway.yml
gateway:
  id: "edge-${DEVICE_ID}"
  name: "边缘计算网关"
  
  system:
    log_level: "warn"
    max_memory_mb: 256  # 资源受限环境
    max_connections: 100
    
services:
  api:
    port: 8080
    
  web:
    enabled: false  # 禁用Web界面节省资源
    
  metrics:
    enabled: true
    port: 9090

# 精简的驱动配置
drivers:
  local_modbus:
    enabled: true
    type: "modbus-tcp"
    endpoint: "tcp://localhost:502"
    
    config:
      polling_interval: "5s"  # 降低采集频率
      max_batch_size: 50

connectors:
  # 仅启用轻量级连接器
  http_api:
    enabled: true
    type: "http"
    config:
      base_url: "${CLOUD_API_URL}"
      batch_size: 50
      batch_timeout: "30s"

# 启用边缘ML推理
ml_engine:
  enabled: true
  models:
    lightweight_anomaly:
      type: "onnx"
      path: "./models/light_anomaly.onnx"

# 禁用资源密集型功能
analytics:
  enabled: false
  
storage:
  # 仅启用WAL，不使用外部数据库
  wal:
    enabled: true
    max_file_size: "10MB"
    max_files: 5
    
  timeseries:
    enabled: false
```

## 配置验证

### 配置语法检查

```bash
# 验证配置文件语法
edge-gateway --config config/gateway.yml --validate

# 检查配置完整性
edge-gateway --config config/gateway.yml --check-config

# 生成配置文档
edge-gateway --generate-config-docs > config-reference.md
```

### 配置测试工具

```bash
# 测试驱动连接
edge-gateway --config config/gateway.yml --test-drivers

# 测试连接器
edge-gateway --config config/gateway.yml --test-connectors

# 验证权限配置
edge-gateway --config config/gateway.yml --test-auth
```

### 常见配置错误

1. **端口冲突**
```yaml
# 错误: 多个服务使用相同端口
services:
  api:
    port: 8080
  web:
    port: 8080  # 冲突!
```

2. **无效的数据类型**
```yaml
# 错误: 不支持的数据类型
datapoints:
  - data_type: "invalid_type"  # 错误!
```

3. **缺少必需参数**
```yaml
# 错误: 缺少endpoint
drivers:
  modbus1:
    type: "modbus-tcp"
    # endpoint: "tcp://..." # 必需!
```

### 配置最佳实践

1. **使用环境变量存储敏感信息**
```yaml
# 好的做法
password: "${DB_PASSWORD}"

# 避免硬编码
password: "hardcoded_password"  # 不安全!
```

2. **使用包含文件组织大型配置**
```yaml
# 主配置文件
drivers: !include drivers.yml
connectors: !include connectors.yml
```

3. **设置合理的资源限制**
```yaml
system:
  max_memory_mb: 1024      # 根据实际环境调整
  max_connections: 1000    # 避免资源耗尽
  thread_pool_size: 8      # 根据CPU核心数调整
```

4. **启用监控和日志**
```yaml
system:
  log_level: "info"        # 生产环境使用info
  enable_metrics: true     # 启用Prometheus指标

monitoring:
  enabled: true
  alerts:
    enabled: true
```

## 相关文档

- [API文档](./API.md)
- [Docker部署指南](./DOCKER_DEPLOYMENT.md)
- [故障排除指南](./TROUBLESHOOTING.md)
- [性能优化指南](./PERFORMANCE.md)