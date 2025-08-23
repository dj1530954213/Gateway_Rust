# REST API接口规范文档

## 📋 概述

Gateway_Rust 提供完整的RESTful API接口，支持设备管理、数据点位操作、驱动管理、历史数据查询、报警管理等核心功能。所有API遵循RESTful设计原则，采用统一的响应格式和错误处理机制。

**技术特性**：
- **框架**: Actix-Web 4.x + SQLx
- **文档**: OpenAPI 3.0 (utoipa)
- **认证**: JWT Token (可选)
- **序列化**: JSON (serde)
- **分页**: 统一分页参数
- **错误处理**: 标准化错误码和消息

## 🎯 API设计原则

### 1. RESTful设计规范

**资源命名**：
- 使用名词复数形式：`/devices`, `/tags`, `/alerts`
- 层级关系清晰：`/devices/{id}/tags`
- 版本控制：`/api/v1/...`

**HTTP方法语义**：
```
GET    /api/v1/devices           # 获取设备列表
POST   /api/v1/devices           # 创建新设备
GET    /api/v1/devices/{id}      # 获取设备详情
PUT    /api/v1/devices/{id}      # 更新设备
DELETE /api/v1/devices/{id}      # 删除设备
```

**状态码使用**：
- `200 OK`: 成功获取资源
- `201 Created`: 成功创建资源
- `204 No Content`: 成功删除资源
- `400 Bad Request`: 请求参数错误
- `404 Not Found`: 资源不存在
- `409 Conflict`: 资源冲突（如名称重复）
- `500 Internal Server Error`: 服务器内部错误

### 2. 统一响应格式

**成功响应**：
```json
{
  "success": true,
  "data": { /* 业务数据 */ },
  "meta": {
    "pagination": {
      "page": 1,
      "size": 20,
      "total": 100,
      "pages": 5,
      "has_next": true,
      "has_prev": false
    },
    "performance": {
      "query_time_ms": 12.5,
      "processing_time_ms": 8.3,
      "total_time_ms": 20.8
    },
    "version": "0.1.0"
  },
  "timestamp": "2025-01-18T10:30:00Z",
  "request_id": "req_12345"
}
```

**错误响应**：
```json
{
  "success": false,
  "error": {
    "code": "DEVICE_NOT_FOUND",
    "message": "Device with ID xxx not found",
    "details": {
      "resource": "Device",
      "id": "xxx"
    },
    "trace_id": "trace_12345"
  },
  "timestamp": "2025-01-18T10:30:00Z",
  "request_id": "req_12345"
}
```

## 📱 核心API接口

### 1. 设备管理API

**基础路径**: `/api/v1/devices`

#### 1.1 创建设备

```http
POST /api/v1/devices
Content-Type: application/json

{
  "name": "PLC-001",
  "protocol": "ModbusTcp",
  "location": "车间A",
  "endpoint": "tcp://192.168.1.100:502",
  "config": {
    "unit_id": 1,
    "timeout": 5000,
    "polling_interval": "1s"
  },
  "enabled": true
}
```

**响应 (201 Created)**：
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "PLC-001",
    "protocol": "ModbusTcp",
    "location": "车间A",
    "endpoint": "tcp://192.168.1.100:502",
    "config": {
      "unit_id": 1,
      "timeout": 5000,
      "polling_interval": "1s"
    },
    "enabled": true,
    "created_at": "2025-01-18T10:30:00Z",
    "updated_at": "2025-01-18T10:30:00Z"
  }
}
```

#### 1.2 查询设备列表

```http
GET /api/v1/devices?protocol=ModbusTcp&enabled=true&page=1&size=20
```

**查询参数**：
- `protocol`: 协议类型过滤 (`ModbusTcp`, `OpcUa`, `Mqtt`)
- `enabled`: 启用状态过滤 (`true`, `false`)
- `page`: 页码 (默认: 1)
- `size`: 每页大小 (默认: 20, 最大: 100)

**响应 (200 OK)**：
```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "PLC-001",
        "protocol": "ModbusTcp",
        "location": "车间A",
        "endpoint": "tcp://192.168.1.100:502",
        "config": { /* ... */ },
        "enabled": true,
        "created_at": "2025-01-18T10:30:00Z",
        "updated_at": "2025-01-18T10:30:00Z"
      }
    ],
    "total": 1,
    "page": 1,
    "size": 20,
    "pages": 1
  }
}
```

#### 1.3 获取设备详情

```http
GET /api/v1/devices/{id}
```

**响应 (200 OK)**：
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "PLC-001",
    "protocol": "ModbusTcp",
    "location": "车间A",
    "endpoint": "tcp://192.168.1.100:502",
    "config": {
      "unit_id": 1,
      "timeout": 5000,
      "polling_interval": "1s"
    },
    "enabled": true,
    "created_at": "2025-01-18T10:30:00Z",
    "updated_at": "2025-01-18T10:30:00Z"
  }
}
```

#### 1.4 更新设备

```http
PUT /api/v1/devices/{id}
Content-Type: application/json

{
  "name": "PLC-001-Updated",
  "location": "车间B",
  "config": {
    "unit_id": 2,
    "timeout": 6000
  },
  "enabled": false
}
```

**响应 (200 OK)**：设备更新后的完整信息

#### 1.5 删除设备

```http
DELETE /api/v1/devices/{id}
```

**响应 (204 No Content)**：无响应体

### 2. 数据点位API

**基础路径**: `/api/v1/tags` 或 `/api/v1/datapoints` (别名)

#### 2.1 创建点位

```http
POST /api/v1/tags
Content-Type: application/json

{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "temperature_sensor_1",
  "address": "40001",
  "data_type": "Float",
  "scaling": 0.1,
  "offset": 0.0,
  "unit": "°C",
  "description": "温度传感器1",
  "enabled": true
}
```

**数据类型枚举**：
- `Float`: 浮点数
- `Int`: 整数
- `Bool`: 布尔值
- `String`: 字符串

#### 2.2 查询点位列表

```http
GET /api/v1/tags?device_id={device_id}&data_type=Float&enabled=true&page=1&size=20
```

#### 2.3 更新点位

```http
PUT /api/v1/tags/{id}
Content-Type: application/json

{
  "name": "temperature_sensor_1_updated",
  "scaling": 0.01,
  "description": "更新后的温度传感器1"
}
```

### 3. 驱动管理API

**基础路径**: `/api/v1/drivers`

#### 3.1 获取驱动列表

```http
GET /api/v1/drivers?driver_kind=static&protocol=ModbusTcp&status=loaded&page=1&page_size=20
```

**查询参数**：
- `driver_kind`: 驱动类型 (`static`, `dynamic`)
- `protocol`: 协议过滤
- `status`: 状态过滤 (`loaded`, `failed`, `unloaded`)
- `name_contains`: 名称模糊搜索
- `active_only`: 仅活跃驱动
- `error_only`: 仅错误驱动

**响应**：
```json
{
  "success": true,
  "data": {
    "drivers": [
      {
        "driver_id": "modbus-tcp-static",
        "driver_kind": "static",
        "name": "modbus-tcp",
        "version": "0.1.0",
        "protocol": "modbus_tcp",
        "status": "loaded",
        "description": "Static Modbus-TCP driver",
        "features": ["read"],
        "loaded_at": "2025-01-18T10:00:00Z",
        "file_path": null,
        "stats": {
          "attached_devices": 2,
          "read_count": 1500,
          "write_count": 0,
          "error_count": 5,
          "avg_response_time_ms": 25.6,
          "success_rate": 99.67
        }
      }
    ],
    "total": 1,
    "page": 1,
    "page_size": 20,
    "total_pages": 1
  }
}
```

#### 3.2 上传动态驱动

```http
POST /api/v1/drivers/upload
Content-Type: multipart/form-data

files: [modbus_driver.so, opcua_driver.dll]
```

**响应**：
```json
{
  "success": true,
  "uploaded_files": [
    {
      "filename": "modbus_driver.so",
      "driver_id": "modbus-dynamic-v1.0",
      "file_size": 2048576,
      "status": "loaded",
      "message": "Driver loaded successfully"
    }
  ],
  "message": "1 file(s) uploaded successfully"
}
```

#### 3.3 热重载驱动

```http
POST /api/v1/drivers/{driver_id}/reload
Content-Type: application/json

{
  "force": false
}
```

#### 3.4 卸载驱动

```http
DELETE /api/v1/drivers/{driver_id}
```

### 4. 驱动配置API

**基础路径**: `/api/v1/driver-configs`

#### 4.1 创建驱动配置

```http
POST /api/v1/driver-configs
Content-Type: application/json

{
  "name": "ModbusTCP-Production",
  "description": "生产环境ModbusTCP配置",
  "protocol": "modbus_tcp",
  "connection_type": "ethernet",
  "enabled": true,
  "config": {
    "host": "192.168.1.100",
    "port": 502,
    "unit_id": 1
  },
  "scan_interval": 1000,
  "timeout": 5000,
  "max_concurrent": 10,
  "batch_size": 100,
  "max_retries": 3,
  "retry_interval": 1000,
  "exponential_backoff": true,
  "max_retry_interval": 10000,
  "log_level": "info",
  "enable_request_log": false,
  "enable_response_log": false,
  "max_log_size": 10,
  "enable_ssl": false,
  "verify_certificate": true
}
```

#### 4.2 驱动配置生命周期管理

```http
POST /api/v1/driver-configs/{id}/start    # 启动配置
POST /api/v1/driver-configs/{id}/stop     # 停止配置
POST /api/v1/driver-configs/{id}/restart  # 重启配置
GET /api/v1/driver-configs/{id}/status    # 获取状态
```

**状态响应**：
```json
{
  "success": true,
  "data": {
    "config_id": "config-uuid",
    "config_name": "ModbusTCP-Production",
    "status": {
      "running": true,
      "enabled": true,
      "managed_driver_id": "modbus-config-001",
      "driver_state": "Active",
      "status_message": "运行正常",
      "last_checked": "2025-01-18T10:30:00Z"
    }
  }
}
```

### 5. 历史数据API

**基础路径**: `/api/v1/history`

#### 5.1 查询历史数据

```http
GET /api/v1/history/data?device_id={device_id}&tag_id={tag_id}&start_time=2025-01-18T00:00:00Z&end_time=2025-01-18T23:59:59Z&limit=1000
```

**查询参数**：
- `device_id`: 设备ID (可选)
- `tag_id`: 点位ID (可选)  
- `start_time`: 开始时间 (ISO8601格式)
- `end_time`: 结束时间 (ISO8601格式)
- `aggregation_window`: 聚合窗口 (`1s`, `1m`, `1h`)
- `limit`: 限制条数 (默认: 1000)
- `offset`: 偏移量

**响应**：
```json
{
  "success": true,
  "data": [
    {
      "device_id": "550e8400-e29b-41d4-a716-446655440000",
      "tag_id": "660e8400-e29b-41d4-a716-446655440001",
      "timestamp": "2025-01-18T10:30:00Z",
      "value": 25.6,
      "unit": "°C"
    }
  ]
}
```

#### 5.2 聚合统计查询

```http
GET /api/v1/history/aggregated?device_id={device_id}&start_time=2025-01-18T00:00:00Z&end_time=2025-01-18T23:59:59Z&window=1h&function=mean
```

**聚合函数**：
- `mean`: 平均值
- `min`: 最小值
- `max`: 最大值
- `sum`: 总和
- `count`: 计数

#### 5.3 数据导出

```http
POST /api/v1/history/export
Content-Type: application/json

{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "start_time": "2025-01-18T00:00:00Z",
  "end_time": "2025-01-18T23:59:59Z",
  "include_headers": true,
  "timestamp_format": "ISO8601"
}
```

**时间戳格式**：
- `ISO8601`: ISO8601格式 (默认)
- `UNIX`: Unix时间戳（秒）
- `UNIX_MS`: Unix时间戳（毫秒）
- `FORMATTED`: 自定义格式化

### 6. 报警管理API

**基础路径**: `/api/v1/alerts`

#### 6.1 创建报警规则

```http
POST /api/v1/alerts/rules
Content-Type: application/json

{
  "name": "温度过高报警",
  "description": "当温度超过50°C时触发报警",
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "tag_id": "660e8400-e29b-41d4-a716-446655440001",
  "op": "GT",
  "threshold": 50.0,
  "level": "WARN",
  "eval_every": "10s",
  "eval_for": "30s",
  "enabled": true
}
```

**比较操作符**：
- `GT`: 大于 (>)
- `LT`: 小于 (<)
- `GTE`: 大于等于 (>=)
- `LTE`: 小于等于 (<=)
- `EQ`: 等于 (==)
- `NE`: 不等于 (!=)

**报警级别**：
- `INFO`: 信息
- `WARN`: 警告
- `CRIT`: 严重

#### 6.2 查询报警历史

```http
GET /api/v1/alerts/history?device_id={device_id}&level=WARN&status=firing&from=2025-01-18T00:00:00Z&to=2025-01-18T23:59:59Z&page=1&size=20
```

**响应**：
```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": "alert-event-uuid",
        "rule_id": "rule-uuid",
        "device_id": "device-uuid",
        "tag_id": "tag-uuid",
        "fired_at": "2025-01-18T10:30:00Z",
        "resolved_at": null,
        "value": 55.2,
        "threshold": 50.0,
        "level": "WARN",
        "message": "温度过高: 55.2°C > 50.0°C",
        "status": "firing"
      }
    ],
    "total": 1,
    "page": 1,
    "size": 20,
    "pages": 1
  }
}
```

### 7. 系统管理API

#### 7.1 健康检查

```http
GET /health
```

**响应**：
```json
{
  "status": "healthy",
  "timestamp": "2025-01-18T10:30:00Z",
  "version": "0.1.0",
  "services": {
    "database": "healthy",
    "frame_bus": "healthy",
    "driver_manager": "healthy",
    "websocket": "healthy"
  }
}
```

#### 7.2 系统信息

```http
GET /api/v1/system/info
```

#### 7.3 数据库连接池状态

```http
GET /api/v1/database/pool/status
```

**响应**：
```json
{
  "success": true,
  "data": {
    "max_connections": 100,
    "active_connections": 15,
    "idle_connections": 5,
    "pool_utilization": 0.15,
    "avg_acquire_time_ms": 2.5,
    "total_acquired": 1500,
    "total_errors": 2
  }
}
```

## 🌐 WebSocket实时API

### 连接建立

```javascript
const ws = new WebSocket('ws://localhost:50013/ws/telemetry?client_id=client-001');
```

### 订阅数据

**发送订阅消息**：
```json
{
  "type": "Subscribe",
  "data": {
    "device_ids": ["550e8400-e29b-41d4-a716-446655440000"],
    "tag_ids": ["660e8400-e29b-41d4-a716-446655440001"],
    "alerts": true,
    "sample_interval_ms": 1000
  }
}
```

**接收遥测数据**：
```json
{
  "type": "Telemetry",
  "data": {
    "device_id": "550e8400-e29b-41d4-a716-446655440000",
    "tag_id": "660e8400-e29b-41d4-a716-446655440001",
    "ts": 1705584600000,
    "value": 25.6,
    "unit": "°C"
  }
}
```

**接收报警通知**：
```json
{
  "type": "Alert",
  "data": {
    "event_id": "alert-event-uuid",
    "rule_name": "温度过高报警",
    "device_name": "PLC-001",
    "tag_name": "temperature_sensor_1",
    "level": "WARN",
    "message": "温度过高: 55.2°C > 50.0°C",
    "fired_at": "2025-01-18T10:30:00Z",
    "value": 55.2,
    "threshold": 50.0
  }
}
```

### WebSocket管理API

```http
GET /ws/status                              # 获取WebSocket状态
GET /ws/connections                         # 获取连接详情
POST /ws/connections/{client_id}/disconnect # 断开指定连接
POST /ws/admin/broadcast                    # 广播管理消息
GET /ws/admin/stats                         # 获取管理统计
```

## 🔒 认证与授权

### JWT Token认证 (可选)

**获取Token**：
```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "password"
}
```

**使用Token**：
```http
GET /api/v1/devices
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## 📊 错误码参考

### 标准错误码

| 错误码 | HTTP状态 | 描述 |
|--------|----------|------|
| `INVALID_REQUEST` | 400 | 请求参数无效 |
| `RESOURCE_NOT_FOUND` | 404 | 资源不存在 |
| `RESOURCE_CONFLICT` | 409 | 资源冲突 |
| `UNAUTHORIZED` | 401 | 未授权访问 |
| `FORBIDDEN` | 403 | 禁止访问 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 |

### 业务错误码

| 错误码 | 描述 |
|--------|------|
| `DEVICE_NOT_FOUND` | 设备不存在 |
| `DEVICE_NAME_EXISTS` | 设备名称已存在 |
| `TAG_NOT_FOUND` | 点位不存在 |
| `TAG_ADDRESS_CONFLICT` | 点位地址冲突 |
| `DRIVER_NOT_LOADED` | 驱动未加载 |
| `DRIVER_LOAD_FAILED` | 驱动加载失败 |
| `CONFIG_VALIDATION_ERROR` | 配置验证失败 |
| `DATABASE_CONNECTION_ERROR` | 数据库连接错误 |

## 🚀 性能与限制

### API限制

| 项目 | 限制值 | 说明 |
|------|--------|------|
| 最大分页大小 | 100 | 单次查询最大记录数 |
| 请求超时 | 30秒 | API请求超时时间 |
| 并发连接数 | 1000 | WebSocket最大并发连接 |
| 文件上传大小 | 100MB | 驱动文件最大上传大小 |
| 历史数据查询 | 10000条 | 单次历史数据查询最大记录数 |

### 性能指标

- **API响应时间**: < 100ms (P95)
- **WebSocket延迟**: < 50ms
- **数据库连接池**: < 10ms获取时间
- **内存使用**: < 512MB (无负载状态)
- **CPU使用**: < 10% (正常负载)

## 📝 OpenAPI文档

完整的API文档通过OpenAPI 3.0规范生成，可通过以下地址访问：

- **Swagger UI**: `http://localhost:50013/swagger-ui/`
- **OpenAPI Spec**: `http://localhost:50013/api-docs/openapi.json`
- **ReDoc**: `http://localhost:50013/redoc/`

---

**注意**：本文档基于实际代码实现，所有接口定义、参数说明和响应格式均来自真实的API路由和DTO定义，确保了文档的准确性和实用性。