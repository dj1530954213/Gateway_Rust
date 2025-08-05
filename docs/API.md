# Edge Gateway API 完整文档

工控物联网边缘网关 REST API 详细说明文档。

## 目录
- [认证](#认证)
- [REST API](#rest-api)
- [WebSocket API](#websocket-api) 
- [Prometheus指标API](#prometheus指标api)
- [健康检查API](#健康检查api)
- [错误处理](#错误处理)
- [SDK和示例](#sdk和示例)

## 认证

### JWT令牌认证

所有API请求需要在Header中包含JWT令牌：

```http
Authorization: Bearer <jwt_token>
```

### 获取令牌

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "password"
}
```

**响应：**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "user": {
    "id": "admin",
    "name": "Administrator", 
    "roles": ["admin"]
  }
}
```

## REST API

### 基础信息

- **基础URL**: `http://localhost:8080/api/v1`
- **内容类型**: `application/json`
- **认证方式**: JWT Bearer Token

### 1. 系统管理

#### 获取系统状态
```http
GET /api/v1/system/status
Authorization: Bearer <token>
```

**响应：**
```json
{
  "gateway": {
    "status": "running",
    "uptime": "2h 15m",
    "version": "1.0.0",
    "build": "2024-01-20T10:30:00Z"
  },
  "drivers": {
    "total": 3,
    "active": 2,
    "failed": 0
  },
  "connectors": {
    "mqtt": {
      "status": "connected",
      "broker": "emqx:1883",
      "last_ping": "2024-01-20T12:45:30Z"
    }
  },
  "statistics": {
    "data_points_today": 12547,
    "commands_executed": 23,
    "success_rate": 98.9
  }
}
```

#### 获取系统配置
```http
GET /api/v1/system/config
Authorization: Bearer <token>
```

#### 更新系统配置
```http
PUT /api/v1/system/config
Authorization: Bearer <token>
Content-Type: application/json

{
  "log_level": "info",
  "max_connections": 1000,
  "enable_metrics": true
}
```

### 2. 驱动管理

#### 获取所有驱动
```http
GET /api/v1/drivers
Authorization: Bearer <token>
```

**响应：**
```json
{
  "drivers": [
    {
      "id": "modbus_line1",
      "name": "生产线PLC1",
      "type": "modbus-tcp",
      "status": "running",
      "endpoint": "tcp://[REAL_PLC_IP]:502",
      "enabled": true,
      "statistics": {
        "points_read": 1250,
        "read_rate": 125.5,
        "error_rate": 0.1,
        "last_read": "2024-01-20T12:45:30Z"
      }
    }
  ]
}
```

#### 获取单个驱动详情
```http
GET /api/v1/drivers/{driver_id}
Authorization: Bearer <token>
```

#### 创建新驱动
```http
POST /api/v1/drivers
Authorization: Bearer <token>
Content-Type: application/json

{
  "id": "modbus_new",
  "name": "新设备",
  "type": "modbus-tcp", 
  "endpoint": "tcp://[REAL_PLC2_IP]:502",
  "config": {
    "unit_id": 1,
    "polling_interval": "2s",
    "timeout": "5s"
  }
}
```

#### 更新驱动配置
```http
PUT /api/v1/drivers/{driver_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "enabled": true,
  "config": {
    "polling_interval": "1s"
  }
}
```

#### 删除驱动
```http
DELETE /api/v1/drivers/{driver_id}
Authorization: Bearer <token>
```

#### 启动/停止驱动
```http
POST /api/v1/drivers/{driver_id}/start
POST /api/v1/drivers/{driver_id}/stop
Authorization: Bearer <token>
```

### 3. 数据点管理

#### 获取数据点列表
```http
GET /api/v1/datapoints?driver_id=modbus_line1&limit=100
Authorization: Bearer <token>
```

**响应：**
```json
{
  "datapoints": [
    {
      "id": "temp_001",
      "name": "温度传感器1",
      "driver_id": "modbus_line1", 
      "address": "40001",
      "data_type": "float32",
      "unit": "°C",
      "current_value": 23.5,
      "quality": "good",
      "timestamp": "2024-01-20T12:45:30Z"
    }
  ],
  "total": 150,
  "page": 1,
  "per_page": 100
}
```

#### 获取数据点历史数据
```http
GET /api/v1/datapoints/{point_id}/history?start=2024-01-20T00:00:00Z&end=2024-01-20T23:59:59Z
Authorization: Bearer <token>
```

#### 写入数据点
```http
POST /api/v1/datapoints/{point_id}/write
Authorization: Bearer <token>
Content-Type: application/json

{
  "value": 25.0,
  "timestamp": "2024-01-20T12:45:30Z"
}
```

### 4. 命令执行

#### 执行设备命令
```http
POST /api/v1/commands/execute
Authorization: Bearer <token>
Content-Type: application/json

{
  "driver_id": "modbus_line1",
  "command": "write_register",
  "parameters": {
    "address": "40001",
    "value": 100,
    "data_type": "uint16"
  },
  "timeout": 30
}
```

**响应：**
```json
{
  "command_id": "cmd_12345",
  "status": "executed",
  "result": {
    "success": true,
    "value": 100,
    "execution_time": 125
  },
  "timestamp": "2024-01-20T12:45:30Z"
}
```

#### 获取命令历史
```http
GET /api/v1/commands/history?limit=50
Authorization: Bearer <token>
```

### 5. 告警管理

#### 获取告警列表
```http
GET /api/v1/alerts?severity=high&status=active
Authorization: Bearer <token>
```

**响应：**
```json
{
  "alerts": [
    {
      "id": "alert_001",
      "severity": "high",
      "title": "设备连接丢失",
      "message": "Modbus设备 [REAL_PLC_IP] 连接超时",
      "source": "modbus_line1",
      "status": "active",
      "created_at": "2024-01-20T12:40:00Z",
      "acknowledged": false
    }
  ]
}
```

#### 确认告警
```http
POST /api/v1/alerts/{alert_id}/acknowledge
Authorization: Bearer <token>
Content-Type: application/json

{
  "message": "已查看，正在处理"
}
```

### 6. 用户和权限管理

#### 获取用户列表
```http
GET /api/v1/users
Authorization: Bearer <token>
```

#### 创建用户
```http
POST /api/v1/users
Authorization: Bearer <token>
Content-Type: application/json

{
  "username": "operator1",
  "name": "操作员1",
  "email": "operator1@company.com",
  "password": "secure_password",
  "roles": ["operator"]
}
```

#### 更新用户权限
```http
PUT /api/v1/users/{user_id}/permissions
Authorization: Bearer <token>
Content-Type: application/json

{
  "permissions": [
    "read:drivers",
    "write:datapoints",
    "execute:commands"
  ]
}
```

### 7. 高级功能API

#### 机器学习模型管理
```http
GET /api/v1/ml/models
POST /api/v1/ml/models
GET /api/v1/ml/models/{model_id}
DELETE /api/v1/ml/models/{model_id}
```

#### 执行推理
```http
POST /api/v1/ml/inference
Authorization: Bearer <token>
Content-Type: application/json

{
  "model_id": "anomaly_detection_v1",
  "input_data": {
    "temperature": 85.5,
    "pressure": 2.1,
    "vibration": 0.05
  }
}
```

#### 数据分析任务
```http
POST /api/v1/analytics/tasks
Authorization: Bearer <token>
Content-Type: application/json

{
  "type": "trend_analysis",
  "datapoints": ["temp_001", "pressure_001"],
  "time_range": {
    "start": "2024-01-20T00:00:00Z",
    "end": "2024-01-20T23:59:59Z"
  },
  "parameters": {
    "window_size": "1h",
    "aggregation": "avg"
  }
}
```

## WebSocket API

### 实时数据订阅

**连接地址**: `ws://localhost:8090/ws`

#### 订阅数据点
```javascript
const ws = new WebSocket('ws://localhost:8090/ws');

// 发送订阅请求
ws.send(JSON.stringify({
  action: 'subscribe',
  type: 'datapoints',
  filters: {
    driver_id: 'modbus_line1',
    tags: ['temperature', 'pressure']
  }
}));

// 接收实时数据
ws.onmessage = function(event) {
  const data = JSON.parse(event.data);
  console.log('实时数据:', data);
};
```

#### 订阅告警
```javascript
ws.send(JSON.stringify({
  action: 'subscribe',
  type: 'alerts',
  filters: {
    severity: ['high', 'critical']
  }
}));
```

#### 订阅系统状态
```javascript
ws.send(JSON.stringify({
  action: 'subscribe', 
  type: 'system_status'
}));
```

## Prometheus指标API

### 端点
- **URL**: `http://localhost:9090/metrics`
- **方法**: GET
- **格式**: Prometheus文本格式

### 核心指标

#### Frame Bus指标
```
# 总线发布的总帧数
frame_publish_total

# 总线丢弃的帧数(缓冲区满)
frame_drop_total

# 环形缓冲区当前使用量
frame_ring_used

# WAL写入的总帧数
frame_wal_write_total

# WAL恢复的帧数
frame_wal_recover_total
```

#### Modbus驱动指标
```
# Modbus PDU总请求数
modbus_pdu_total

# 读取的总点位数
modbus_point_total

# 点位读取延迟分布(毫秒)
modbus_point_latency_ms_bucket

# 重连总次数
modbus_reconnect_total

# Modbus异常总数
modbus_exception_total
```

#### MQTT连接器指标
```
# MQTT连接总数
mqtt_connect_total

# MQTT断连总数  
mqtt_disconnect_total

# MQTT重连总数
mqtt_reconnect_total

# 发布的消息总数
mqtt_publish_total

# 发布错误总数
mqtt_publish_error_total

# 消息发布延迟分布(毫秒)
mqtt_publish_latency_ms_bucket

# 消息大小分布(字节)
mqtt_message_size_bytes_bucket

# 缓冲区使用量
mqtt_buffer_used

# 压缩比例分布
mqtt_compression_ratio_bucket

# 批量大小分布
mqtt_batch_size_bucket
```

### 示例查询

#### 获取所有指标
```bash
curl http://localhost:9090/metrics
```

#### Prometheus查询示例
```promql
# 每秒帧发布速率
rate(frame_publish_total[5m])

# 环形缓冲区使用率
frame_ring_used / frame_ring_capacity * 100

# Modbus点位平均延迟
histogram_quantile(0.95, rate(modbus_point_latency_ms_bucket[5m]))

# MQTT发布成功率
rate(mqtt_publish_total[5m]) / (rate(mqtt_publish_total[5m]) + rate(mqtt_publish_error_total[5m])) * 100
```

## 健康检查API

### 端点
- **URL**: `http://localhost:9090/health`
- **方法**: GET
- **格式**: JSON

### 响应示例
```json
{
  "status": "ok",
  "service": "edge-gateway"
}
```

## Grafana仪表板

### 预配置仪表板

#### Edge Gateway Overview
- **URL**: http://localhost:3000/d/edge-gateway-overview
- **内容**: 系统总览，关键指标监控

#### Driver Performance  
- **URL**: http://localhost:3000/d/driver-performance
- **内容**: 驱动性能详细监控

#### MQTT Connector
- **URL**: http://localhost:3000/d/mqtt-connector
- **内容**: MQTT连接器状态和性能

#### System Resources
- **URL**: http://localhost:3000/d/system-resources
- **内容**: 系统资源使用情况

### 自定义查询

#### 关键面板查询

1. **数据采集速率**
```promql
sum(rate(modbus_point_total[5m])) by (driver)
```

2. **上云成功率**
```promql
(
  sum(rate(mqtt_publish_total[5m])) / 
  (sum(rate(mqtt_publish_total[5m])) + sum(rate(mqtt_publish_error_total[5m])))
) * 100
```

3. **系统延迟分布**
```promql
histogram_quantile(0.99, sum(rate(modbus_point_latency_ms_bucket[5m])) by (le))
```

4. **内存使用趋势**
```promql
process_resident_memory_bytes
```

## EMQX管理API

### 控制台
- **URL**: http://localhost:18083
- **用户名**: admin
- **密码**: public

### REST API示例

#### 获取客户端列表
```bash
curl -u admin:public \
  http://localhost:18083/api/v5/clients
```

#### 获取主题统计
```bash
curl -u admin:public \
  http://localhost:18083/api/v5/stats
```

## 错误处理

### 标准错误响应格式

所有API错误都遵循统一的响应格式：

```json
{
  "error": {
    "code": "DRIVER_NOT_FOUND",
    "message": "指定的驱动不存在",
    "details": "驱动ID 'modbus_invalid' 未找到",
    "timestamp": "2024-01-20T12:45:30Z",
    "request_id": "req_12345"
  }
}
```

### 常见错误代码

| 错误代码 | HTTP状态 | 说明 |
|---------|---------|------|
| `UNAUTHORIZED` | 401 | 未提供有效的认证令牌 |
| `FORBIDDEN` | 403 | 权限不足 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `VALIDATION_ERROR` | 400 | 请求参数验证失败 |
| `DRIVER_ERROR` | 500 | 驱动操作失败 |
| `DEVICE_TIMEOUT` | 504 | 设备通信超时 |
| `RATE_LIMIT_EXCEEDED` | 429 | 请求频率超限 |

### 重试策略

对于临时性错误（如网络超时），建议使用指数退避重试：

```javascript
async function apiCallWithRetry(apiCall, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await apiCall();
    } catch (error) {
      if (error.status >= 500 && i < maxRetries - 1) {
        await new Promise(resolve => 
          setTimeout(resolve, Math.pow(2, i) * 1000)
        );
        continue;
      }
      throw error;
    }
  }
}
```

## SDK和示例

### JavaScript/TypeScript SDK

#### 安装
```bash
npm install @edge-gateway/client
```

#### 基础使用
```javascript
import { EdgeGatewayClient } from '@edge-gateway/client';

const client = new EdgeGatewayClient({
  baseURL: 'http://localhost:8080/api/v1',
  auth: {
    username: 'admin',
    password: 'password'
  }
});

// 获取系统状态
const status = await client.system.getStatus();
console.log('网关状态:', status);

// 获取驱动列表
const drivers = await client.drivers.list();
console.log('驱动列表:', drivers);

// 订阅实时数据
client.realtime.subscribe('datapoints', {
  driver_id: 'modbus_line1'
}, (data) => {
  console.log('实时数据:', data);
});
```

### Python SDK

#### 安装
```bash
pip install edge-gateway-client
```

#### 基础使用
```python
from edge_gateway import EdgeGatewayClient

client = EdgeGatewayClient(
    base_url='http://localhost:8080/api/v1',
    username='admin',
    password='password'
)

# 获取系统状态
status = client.system.get_status()
print(f"网关状态: {status}")

# 创建驱动
driver = client.drivers.create({
    'id': 'new_modbus',
    'name': '新Modbus设备',
    'type': 'modbus-tcp',
    'endpoint': 'tcp://[REAL_PLC3_IP]:502'
})

# 执行命令
result = client.commands.execute({
    'driver_id': 'modbus_line1',
    'command': 'write_register',
    'parameters': {
        'address': '40001',
        'value': 100
    }
})
```

### cURL示例集合

#### 获取认证令牌
```bash
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"password"}' \
  | jq -r '.token')
```

#### 获取系统状态
```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/system/status | jq
```

#### 创建驱动
```bash
curl -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test_modbus",
    "name": "测试设备",
    "type": "modbus-tcp",
    "endpoint": "tcp://localhost:502",
    "config": {
      "unit_id": 1,
      "polling_interval": "2s"
    }
  }' \
  http://localhost:8080/api/v1/drivers
```

#### 执行写命令
```bash
curl -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "driver_id": "modbus_line1",
    "command": "write_register",
    "parameters": {
      "address": "40001",
      "value": 100,
      "data_type": "uint16"
    }
  }' \
  http://localhost:8080/api/v1/commands/execute
```

### Postman集合

下载预配置的Postman集合：
- [Edge Gateway API.postman_collection.json](./postman/Edge_Gateway_API.postman_collection.json)
- [Environment.postman_environment.json](./postman/Environment.postman_environment.json)

### 实时数据WebSocket示例

#### 完整的Web客户端示例
```html
<!DOCTYPE html>
<html>
<head>
    <title>Edge Gateway 实时监控</title>
</head>
<body>
    <div id="status"></div>
    <div id="data"></div>
    
    <script>
        class GatewayMonitor {
            constructor(wsUrl, apiUrl) {
                this.wsUrl = wsUrl;
                this.apiUrl = apiUrl;
                this.token = null;
                this.ws = null;
            }
            
            async authenticate(username, password) {
                const response = await fetch(`${this.apiUrl}/auth/login`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ username, password })
                });
                const data = await response.json();
                this.token = data.token;
                return this.token;
            }
            
            connectWebSocket() {
                this.ws = new WebSocket(this.wsUrl);
                
                this.ws.onopen = () => {
                    console.log('WebSocket连接已建立');
                    // 订阅实时数据
                    this.ws.send(JSON.stringify({
                        action: 'subscribe',
                        type: 'datapoints',
                        auth: this.token
                    }));
                };
                
                this.ws.onmessage = (event) => {
                    const data = JSON.parse(event.data);
                    this.handleRealtimeData(data);
                };
                
                this.ws.onerror = (error) => {
                    console.error('WebSocket错误:', error);
                };
                
                this.ws.onclose = () => {
                    console.log('WebSocket连接已关闭');
                    // 重连逻辑
                    setTimeout(() => this.connectWebSocket(), 5000);
                };
            }
            
            handleRealtimeData(data) {
                document.getElementById('data').innerHTML = 
                    `<pre>${JSON.stringify(data, null, 2)}</pre>`;
            }
        }
        
        // 使用示例
        const monitor = new GatewayMonitor(
            'ws://localhost:8090/ws',
            'http://localhost:8080/api/v1'
        );
        
        monitor.authenticate('admin', 'password').then(() => {
            monitor.connectWebSocket();
        });
    </script>
</body>
</html>
```

## 配置热重载

### 触发重载
配置文件修改会自动触发热重载，无需重启服务。

### 监控配置变更
```bash
# 查看配置重载日志
docker logs edge-gateway | grep "reloaded"
```

### 手动重载API
```http
POST /api/v1/system/reload
Authorization: Bearer <token>
```

## 版本信息

### API版本控制

当前API版本: `v1`

**向后兼容策略:**
- 新字段添加不增加版本号
- 字段删除或重命名会增加版本号
- 行为变更会增加版本号

### 版本查询
```http
GET /api/version
```

**响应:**
```json
{
  "api_version": "v1",
  "gateway_version": "1.0.0",
  "build_date": "2024-01-20T10:30:00Z",
  "git_commit": "abc123def",
  "supported_protocols": ["modbus-tcp", "modbus-rtu"],
  "features": ["ml_engine", "analytics", "edge_compute"]
}
```

---

## 附录

### A. 完整的API端点列表

| 方法 | 端点 | 说明 |
|------|------|------|
| POST | `/api/v1/auth/login` | 用户登录 |
| POST | `/api/v1/auth/logout` | 用户登出 |
| GET | `/api/v1/system/status` | 获取系统状态 |
| GET | `/api/v1/system/config` | 获取系统配置 |
| PUT | `/api/v1/system/config` | 更新系统配置 |
| POST | `/api/v1/system/reload` | 重载配置 |
| GET | `/api/v1/drivers` | 获取驱动列表 |
| POST | `/api/v1/drivers` | 创建驱动 |
| GET | `/api/v1/drivers/{id}` | 获取驱动详情 |
| PUT | `/api/v1/drivers/{id}` | 更新驱动 |
| DELETE | `/api/v1/drivers/{id}` | 删除驱动 |
| POST | `/api/v1/drivers/{id}/start` | 启动驱动 |
| POST | `/api/v1/drivers/{id}/stop` | 停止驱动 |
| GET | `/api/v1/datapoints` | 获取数据点列表 |
| GET | `/api/v1/datapoints/{id}` | 获取数据点详情 |
| GET | `/api/v1/datapoints/{id}/history` | 获取历史数据 |
| POST | `/api/v1/datapoints/{id}/write` | 写入数据点 |
| POST | `/api/v1/commands/execute` | 执行命令 |
| GET | `/api/v1/commands/history` | 获取命令历史 |
| GET | `/api/v1/alerts` | 获取告警列表 |
| POST | `/api/v1/alerts/{id}/acknowledge` | 确认告警 |
| GET | `/api/v1/users` | 获取用户列表 |
| POST | `/api/v1/users` | 创建用户 |
| PUT | `/api/v1/users/{id}` | 更新用户 |
| DELETE | `/api/v1/users/{id}` | 删除用户 |
| GET | `/api/v1/ml/models` | 获取ML模型 |
| POST | `/api/v1/ml/inference` | 执行推理 |
| POST | `/api/v1/analytics/tasks` | 创建分析任务 |

### B. 数据类型定义

详细的数据模型和类型定义请参考：
- [数据模型文档](./MODELS.md)
- [OpenAPI规范文件](./openapi.yaml)

### C. 相关文档

- [快速开始指南](../README.md)
- [Docker部署指南](./DOCKER_DEPLOYMENT.md)
- [配置指南](./CONFIGURATION.md)
- [故障排除](./TROUBLESHOOTING.md)