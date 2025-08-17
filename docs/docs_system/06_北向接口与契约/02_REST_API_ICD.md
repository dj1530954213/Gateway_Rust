# REST API 接口控制文档 (ICD)

## 概述

Gateway_Rust系统提供完整的RESTful API接口，基于OpenAPI 3.0规范，支持设备管理、数据查询、报警配置等核心功能。API采用Actix-Web框架实现，提供高性能的HTTP服务。

## API基础信息

| 属性 | 值 |
|------|---|
| **Base URL** | `http://localhost:50013/api/v1` |
| **协议** | HTTP/1.1, HTTP/2 |
| **数据格式** | JSON |
| **编码** | UTF-8 |
| **API版本** | v1.0 |
| **OpenAPI版本** | 3.0.3 |

## 认证授权

### JWT Token 认证

```http
Authorization: Bearer <jwt_token>
```

### 获取Token

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "secure_password"
}
```

**响应**:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "def502003..."
}
```

## 核心API端点

### 1. 设备管理 API

#### 1.1 获取设备列表

```http
GET /api/v1/devices?page=1&limit=20&protocol=ModbusTcp
```

**查询参数**:
- `page` (integer, optional): 页码，默认1
- `limit` (integer, optional): 每页数量，默认20，最大100
- `protocol` (string, optional): 协议过滤
- `enabled` (boolean, optional): 启用状态过滤

**响应**:
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "PLC-001",
      "protocol": "ModbusTcp",
      "location": "Workshop A",
      "endpoint": "192.168.1.100:502",
      "enabled": true,
      "created_at": "2025-01-17T10:00:00Z",
      "updated_at": "2025-01-17T10:00:00Z",
      "config": {
        "slave_id": 1,
        "timeout": 5000,
        "retry_count": 3
      }
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 156,
    "total_pages": 8
  }
}
```

#### 1.2 创建设备

```http
POST /api/v1/devices
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "PLC-002",
  "protocol": "ModbusTcp",
  "location": "Workshop B",
  "endpoint": "192.168.1.101:502",
  "config": {
    "slave_id": 2,
    "timeout": 5000,
    "retry_count": 3
  },
  "enabled": true
}
```

**响应** (201 Created):
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440001",
  "name": "PLC-002",
  "protocol": "ModbusTcp",
  "location": "Workshop B",
  "endpoint": "192.168.1.101:502",
  "enabled": true,
  "created_at": "2025-01-17T11:00:00Z",
  "updated_at": "2025-01-17T11:00:00Z",
  "config": {
    "slave_id": 2,
    "timeout": 5000,
    "retry_count": 3
  }
}
```

#### 1.3 获取单个设备

```http
GET /api/v1/devices/{device_id}
```

#### 1.4 更新设备

```http
PUT /api/v1/devices/{device_id}
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "PLC-002-Updated",
  "location": "Workshop B-2",
  "enabled": false
}
```

#### 1.5 删除设备

```http
DELETE /api/v1/devices/{device_id}
Authorization: Bearer <token>
```

### 2. 标签管理 API

#### 2.1 获取标签列表

```http
GET /api/v1/devices/{device_id}/tags?data_type=Float
```

**响应**:
```json
{
  "data": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440002",
      "device_id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "temperature_1",
      "address": "40001",
      "data_type": "Float",
      "scaling": 0.1,
      "tag_offset": -40.0,
      "unit": "°C",
      "description": "Workshop temperature sensor 1",
      "enabled": true,
      "created_at": "2025-01-17T10:00:00Z"
    }
  ]
}
```

#### 2.2 创建标签

```http
POST /api/v1/devices/{device_id}/tags
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "pressure_1",
  "address": "40002",
  "data_type": "Float",
  "scaling": 0.01,
  "tag_offset": 0.0,
  "unit": "bar",
  "description": "Hydraulic pressure sensor",
  "enabled": true
}
```

### 3. 历史数据查询 API

#### 3.1 查询时序数据

```http
GET /api/v1/history/data?tags=temperature_1,pressure_1&start=2025-01-17T00:00:00Z&end=2025-01-17T23:59:59Z&interval=1m
```

**查询参数**:
- `tags` (string, required): 标签名称列表，逗号分隔
- `start` (string, required): 开始时间 (ISO 8601)
- `end` (string, required): 结束时间 (ISO 8601)
- `interval` (string, optional): 数据聚合间隔 (1s, 1m, 1h, 1d)
- `aggregation` (string, optional): 聚合函数 (mean, max, min, sum)

**响应**:
```json
{
  "data": [
    {
      "tag": "temperature_1",
      "device_name": "PLC-001",
      "unit": "°C",
      "points": [
        {
          "timestamp": "2025-01-17T10:00:00Z",
          "value": 25.6,
          "quality": 1
        },
        {
          "timestamp": "2025-01-17T10:01:00Z",
          "value": 25.8,
          "quality": 1
        }
      ]
    }
  ],
  "metadata": {
    "total_points": 1440,
    "tags_count": 2,
    "time_range": {
      "start": "2025-01-17T00:00:00Z",
      "end": "2025-01-17T23:59:59Z"
    }
  }
}
```

### 4. 实时数据 API

#### 4.1 获取实时数据

```http
GET /api/v1/realtime/current?devices=PLC-001,PLC-002
```

**响应**:
```json
{
  "data": [
    {
      "device_id": "550e8400-e29b-41d4-a716-446655440000",
      "device_name": "PLC-001",
      "tags": [
        {
          "tag_name": "temperature_1",
          "value": 25.6,
          "unit": "°C",
          "quality": 1,
          "timestamp": "2025-01-17T15:30:00Z"
        }
      ]
    }
  ],
  "timestamp": "2025-01-17T15:30:00Z"
}
```

### 5. 系统管理 API

#### 5.1 系统健康检查

```http
GET /api/v1/health
```

**响应**:
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": "2025-01-17T15:30:00Z",
  "components": {
    "database": {
      "status": "healthy",
      "connection_pool": {
        "active": 5,
        "idle": 15,
        "max": 100
      }
    },
    "frame_bus": {
      "status": "healthy",
      "latency_p99": 0.8,
      "throughput": 1200
    },
    "influxdb": {
      "status": "healthy",
      "write_latency": 2.1
    }
  }
}
```

#### 5.2 系统指标

```http
GET /api/v1/system/metrics
```

**响应**:
```json
{
  "system": {
    "cpu_usage": 15.2,
    "memory_usage": 89.5,
    "disk_usage": 45.8,
    "uptime": 86400
  },
  "frame_bus": {
    "messages_per_second": 1200,
    "latency_p95": 0.6,
    "latency_p99": 0.8,
    "buffer_usage": 12.5,
    "backpressure_active": false
  },
  "drivers": {
    "active_count": 3,
    "error_rate": 0.01,
    "total_devices": 15
  }
}
```

## WebSocket 实时通信

### 连接建立

```javascript
const ws = new WebSocket('ws://localhost:50013/ws/telemetry');

ws.onopen = function(event) {
    console.log('WebSocket connected');
    
    // 订阅实时数据
    ws.send(JSON.stringify({
        type: 'subscribe',
        filters: {
            devices: ['PLC-001'],
            tags: ['temperature_1', 'pressure_1']
        }
    }));
};

ws.onmessage = function(event) {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
};
```

### 消息格式

**订阅消息**:
```json
{
  "type": "subscribe",
  "filters": {
    "devices": ["PLC-001"],
    "tags": ["temperature_1"],
    "data_types": ["Float"]
  }
}
```

**数据推送**:
```json
{
  "type": "data",
  "timestamp": "2025-01-17T15:30:00Z",
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "device_name": "PLC-001",
  "tag_name": "temperature_1",
  "value": 25.6,
  "unit": "°C",
  "quality": 1
}
```

## 错误处理

### 标准错误响应

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request data",
    "details": [
      {
        "field": "name",
        "message": "Device name already exists"
      }
    ],
    "trace_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2025-01-17T15:30:00Z"
  }
}
```

### 错误码定义

| HTTP状态码 | 错误码 | 说明 |
|-----------|--------|------|
| 400 | `VALIDATION_ERROR` | 请求参数验证失败 |
| 401 | `UNAUTHORIZED` | 未授权访问 |
| 403 | `FORBIDDEN` | 权限不足 |
| 404 | `NOT_FOUND` | 资源不存在 |
| 409 | `CONFLICT` | 资源冲突 |
| 422 | `UNPROCESSABLE_ENTITY` | 业务逻辑错误 |
| 429 | `RATE_LIMITED` | 请求频率限制 |
| 500 | `INTERNAL_ERROR` | 内部服务错误 |
| 503 | `SERVICE_UNAVAILABLE` | 服务暂时不可用 |

## 性能限制

### 速率限制

| 端点类型 | 限制 | 窗口期 |
|---------|------|--------|
| **认证** | 10次/IP | 1分钟 |
| **查询API** | 100次/token | 1分钟 |
| **写入API** | 50次/token | 1分钟 |
| **WebSocket** | 5连接/IP | - |

### 数据限制

| 参数 | 限制 |
|------|------|
| **查询时间范围** | 最大7天 |
| **批量操作** | 最大1000条记录 |
| **历史数据点** | 每次查询最大10000点 |
| **WebSocket消息** | 最大1MB |
| **JSON载荷** | 最大10MB |

## SDK 示例

### JavaScript/Node.js

```javascript
class GatewayClient {
    constructor(baseURL, token) {
        this.baseURL = baseURL;
        this.token = token;
    }

    async getDevices(options = {}) {
        const params = new URLSearchParams(options);
        const response = await fetch(`${this.baseURL}/devices?${params}`, {
            headers: {
                'Authorization': `Bearer ${this.token}`,
                'Content-Type': 'application/json'
            }
        });
        return response.json();
    }

    async createDevice(device) {
        const response = await fetch(`${this.baseURL}/devices`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${this.token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(device)
        });
        return response.json();
    }
}

// 使用示例
const client = new GatewayClient('http://localhost:50013/api/v1', 'your-token');

const devices = await client.getDevices({
    protocol: 'ModbusTcp',
    limit: 50
});

console.log('设备列表:', devices);
```

### Python

```python
import requests
import json

class GatewayClient:
    def __init__(self, base_url, token):
        self.base_url = base_url
        self.token = token
        self.headers = {
            'Authorization': f'Bearer {token}',
            'Content-Type': 'application/json'
        }

    def get_devices(self, **params):
        response = requests.get(
            f'{self.base_url}/devices',
            headers=self.headers,
            params=params
        )
        response.raise_for_status()
        return response.json()

    def create_device(self, device):
        response = requests.post(
            f'{self.base_url}/devices',
            headers=self.headers,
            json=device
        )
        response.raise_for_status()
        return response.json()

# 使用示例
client = GatewayClient('http://localhost:50013/api/v1', 'your-token')

devices = client.get_devices(protocol='ModbusTcp', limit=50)
print(f'找到 {len(devices["data"])} 个设备')
```

## OpenAPI 规范

完整的OpenAPI 3.0规范文档可通过以下地址访问：

- **Swagger UI**: `http://localhost:50013/swagger-ui/`
- **OpenAPI JSON**: `http://localhost:50013/api/v1/openapi.json`
- **ReDoc**: `http://localhost:50013/redoc/`

## 最佳实践

### 1. 性能优化

- 使用适当的分页参数避免大量数据传输
- 历史数据查询使用合理的时间范围和聚合间隔
- WebSocket连接复用，避免频繁建立连接
- 使用压缩减少网络传输量

### 2. 错误处理

- 始终检查HTTP状态码和响应错误信息
- 实现指数退避的重试机制
- 记录trace_id用于问题追踪
- 对临时性错误进行适当重试

### 3. 安全考虑

- Token定期刷新，避免长期使用
- 敏感信息不要记录在日志中
- 使用HTTPS进行生产环境通信
- 实现适当的输入验证和sanitization

---

**API版本**: v1.0  
**最后更新**: 2025-01-17  
**OpenAPI版本**: 3.0.3  
**维护团队**: Gateway API Team