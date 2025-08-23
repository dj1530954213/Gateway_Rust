# 数据库模式设计文档

## 📋 概述

Gateway_Rust 采用 **PostgreSQL** 作为主要数据库，设计了一个支持工业网关核心功能的完整数据库架构。该设计遵循严格的关系型数据库范式，支持设备管理、点位数据、报警系统、用户认证、驱动管理等关键业务场景。

**技术栈**：
- **数据库引擎**：PostgreSQL 14+
- **ORM框架**：SQLx (异步SQL工具包)
- **迁移管理**：SQL迁移文件
- **数据访问**：Repository模式

## 🏗️ 整体架构

### 模块化设计

数据库按功能模块划分为6个核心领域：

```
数据库架构
├── 用户认证模块 (0001_init_auth.sql)
│   ├── users (用户表)
│   ├── roles (角色表) 
│   ├── permissions (权限表)
│   └── 关联表 (user_roles, role_permissions)
├── 设备管理模块 (0003_init_device_tag.sql)
│   ├── devices (设备表)
│   └── tags (点位表)
├── 驱动系统模块 (0004_driver_registry.sql)
│   ├── driver_registry (驱动注册表)
│   └── driver_load_history (驱动加载历史)
├── 报警系统模块 (0005_alerts.sql)
│   ├── alert_rules (报警规则)
│   ├── alert_history (报警历史)
│   ├── notification_configs (通知配置)
│   └── alert_rule_notifications (规则通知关联)
├── 驱动配置模块 (0007_driver_configs.sql)
│   └── driver_configs (用户驱动配置)
└── 基础数据模块 (0006_seed_data.sql)
    └── 初始化数据
```

### 数据类型枚举

系统定义了5个核心枚举类型：

```sql
-- 协议类型 (支持的工业协议)
CREATE TYPE protocol_kind AS ENUM ('ModbusTcp','OpcUa','Mqtt');

-- 数据类型 (点位数据类型)
CREATE TYPE tag_data_type AS ENUM ('Float','Int','Bool','String');

-- 比较操作 (报警条件)
CREATE TYPE compare_op AS ENUM ('GT','LT','GTE','LTE','EQ','NE');

-- 报警级别
CREATE TYPE alert_level AS ENUM ('INFO','WARN','CRIT');

-- 通知类型
CREATE TYPE notification_type AS ENUM ('email','webhook','websocket');
```

## 📊 核心数据表详解

**注意**：以下表结构均基于实际迁移文件分析，反映了项目的真实数据库实现。

### 1. 用户认证系统

**基于**: `/schema/migrations/0001_init_auth.sql`

#### users 表 - 用户主表
```sql
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(100),
    avatar_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_login TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**业务特性**：
- UUID主键确保全局唯一性
- 用户名和邮箱唯一性约束
- 密码哈希存储(安全策略)
- 软删除支持(is_active字段)
- 登录时间跟踪

#### roles 表 - 角色管理
```sql
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

#### permissions 表 - 权限定义
```sql
CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    resource VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### 2. 设备管理系统

**基于**: `/schema/migrations/0003_init_device_tag.sql`

#### devices 表 - 设备主表
```sql
CREATE TABLE devices (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL UNIQUE,
    protocol    protocol_kind NOT NULL,
    location    VARCHAR(128),
    endpoint    TEXT,                    -- 连接地址 tcp://[IP]:[PORT]
    config      JSONB,                   -- 协议特定配置
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**设计特点**：
- 支持多种工业协议(ModbusTcp, OpcUa, Mqtt)
- JSONB配置字段支持灵活的协议参数
- 软启用/禁用机制
- 自动时间戳管理

#### tags 表 - 数据点位表
```sql
CREATE TABLE tags (
    id          UUID PRIMARY KEY,
    device_id   UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    name        VARCHAR(64) NOT NULL,
    address     VARCHAR(32) NOT NULL,     -- 寄存器地址 "40001"
    data_type   tag_data_type NOT NULL,
    scaling     DOUBLE PRECISION,         -- 比例因子
    "offset"    DOUBLE PRECISION,        -- 偏移量 (注意:SQL关键字需转义)
    unit        VARCHAR(16),             -- 单位
    description TEXT,
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(device_id, address)
);
```

**业务逻辑**：
- 强制外键约束确保数据一致性
- 设备内地址唯一性约束
- 数据变换支持(scaling + offset)
- 支持4种基础数据类型

### 3. 报警管理系统

**基于**: `/schema/migrations/0005_alerts.sql`

#### alert_rules 表 - 报警规则
```sql
CREATE TABLE alert_rules (
    id           UUID PRIMARY KEY,
    name         VARCHAR(64) NOT NULL,
    description  TEXT,
    device_id    UUID REFERENCES devices(id) ON DELETE CASCADE,
    tag_id       UUID REFERENCES tags(id) ON DELETE CASCADE,
    op           compare_op  NOT NULL,
    threshold    DOUBLE PRECISION NOT NULL,
    level        alert_level NOT NULL DEFAULT 'WARN',
    eval_every   INTERVAL NOT NULL DEFAULT INTERVAL '10 seconds',
    eval_for     INTERVAL DEFAULT INTERVAL '0 seconds',  -- 持续时间触发
    enabled      BOOLEAN NOT NULL DEFAULT TRUE,
    created_by   VARCHAR(64),             -- 创建用户
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT valid_device_tag CHECK (
        (device_id IS NOT NULL AND tag_id IS NOT NULL) OR 
        (device_id IS NOT NULL AND tag_id IS NULL)    -- 设备级别规则
    )
);
```

**高级特性**：
- 支持设备级和点位级报警规则
- 时间间隔配置(检查频率 + 持续时间)
- 6种比较操作符(GT, LT, GTE, LTE, EQ, NE)
- 3级报警等级(INFO, WARN, CRIT)
- 约束检查确保规则逻辑正确

#### alert_history 表 - 报警历史
```sql
CREATE TABLE alert_history (
    id           UUID PRIMARY KEY,
    rule_id      UUID NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    device_id    UUID,
    tag_id       UUID,
    fired_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at  TIMESTAMPTZ,
    value        DOUBLE PRECISION,
    threshold    DOUBLE PRECISION,
    level        alert_level NOT NULL,
    message      TEXT NOT NULL,
    status       VARCHAR(16) NOT NULL DEFAULT 'firing' CHECK (status IN ('firing', 'resolved')),
    duration     INTERVAL GENERATED ALWAYS AS (resolved_at - fired_at) STORED
);
```

**时序特性**：
- 计算列自动计算报警持续时间
- 状态枚举约束(firing/resolved)
- 完整的报警生命周期跟踪

### 4. 驱动管理系统

**基于**: `/schema/migrations/0004_driver_registry.sql`

#### driver_registry 表 - 驱动注册表
```sql
CREATE TABLE driver_registry (
    protocol    protocol_kind PRIMARY KEY,
    version     VARCHAR(32) NOT NULL,
    file_path   TEXT NOT NULL,           -- .so 文件路径
    file_hash   VARCHAR(64) NOT NULL,    -- 文件SHA256校验
    api_version SMALLINT NOT NULL DEFAULT 1,
    metadata    JSONB,                   -- 驱动元数据
    loaded_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    status      VARCHAR(16) NOT NULL DEFAULT 'loaded' CHECK (status IN ('loaded', 'failed', 'unloaded'))
);
```

**安全机制**：
- 文件完整性校验(SHA256)
- API版本兼容性管理
- 驱动状态跟踪
- 元数据扩展支持

#### driver_configs 表 - 用户驱动配置
```sql
CREATE TABLE driver_configs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(100) NOT NULL UNIQUE,
    description     TEXT,
    protocol        VARCHAR(50) NOT NULL,  -- 'modbus_tcp', 'modbus_rtu', 'opcua', 'mqtt', 'ethernet_ip', 'bacnet'
    connection_type VARCHAR(20) NOT NULL,  -- 'ethernet', 'serial'
    enabled         BOOLEAN NOT NULL DEFAULT true,
    
    -- 协议特定配置 (JSON)
    config          JSONB NOT NULL DEFAULT '{}',
    
    -- 性能设置
    scan_interval   INTEGER NOT NULL DEFAULT 1000,      -- ms
    timeout         INTEGER NOT NULL DEFAULT 5000,      -- ms
    max_concurrent  INTEGER NOT NULL DEFAULT 10,
    batch_size      INTEGER NOT NULL DEFAULT 100,
    
    -- 重连策略
    max_retries           INTEGER NOT NULL DEFAULT 3,
    retry_interval        INTEGER NOT NULL DEFAULT 1000,    -- ms
    exponential_backoff   BOOLEAN NOT NULL DEFAULT true,
    max_retry_interval    INTEGER NOT NULL DEFAULT 10000,   -- ms
    
    -- 日志设置
    log_level             VARCHAR(10) NOT NULL DEFAULT 'info',
    enable_request_log    BOOLEAN NOT NULL DEFAULT false,
    enable_response_log   BOOLEAN NOT NULL DEFAULT false,
    max_log_size         INTEGER NOT NULL DEFAULT 10,       -- MB
    
    -- 安全配置
    enable_ssl           BOOLEAN NOT NULL DEFAULT false,
    verify_certificate   BOOLEAN NOT NULL DEFAULT true,
    client_cert_path     TEXT,
    client_key_path      TEXT,
    
    -- 元数据
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**企业级特性**：
- 完整的性能调优参数
- 自适应重连策略(指数退避)
- 细粒度日志控制
- SSL/TLS安全配置
- 证书路径管理

## 🔗 Rust数据模型映射

### SQLx集成

项目使用SQLx提供类型安全的数据库访问：

```rust
use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Device {
    pub id: Uuid,
    pub name: String,
    pub protocol: DbProtocolKind,
    pub location: Option<String>,
    pub endpoint: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 枚举类型映射

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "protocol_kind", rename_all = "PascalCase")]
pub enum DbProtocolKind {
    ModbusTcp,
    OpcUa,
    Mqtt,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "tag_data_type", rename_all = "PascalCase")]
pub enum DbTagDataType {
    Float,
    Int,
    Bool,
    String,
}
```

### CRUD操作模式

每个核心实体提供三种数据结构：

1. **主模型** - 完整数据读取
2. **创建模型** - 新增数据(New*)
3. **更新模型** - 部分字段更新(*Update)

```rust
// 主模型 - 查询使用
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Tag { /* 完整字段 */ }

// 创建模型 - 插入使用
#[derive(Debug, Serialize, Deserialize)]
pub struct NewTag { /* 必需字段 */ }

// 更新模型 - 更新使用
#[derive(Debug, Serialize, Deserialize)]
pub struct TagUpdate { /* Option<T>字段 */ }
```

## 🚀 性能优化设计

### 索引策略

**查询频率优化**：
```sql
-- 设备相关索引
CREATE INDEX idx_devices_protocol ON devices(protocol);
CREATE INDEX idx_devices_enabled ON devices(enabled);

-- 点位查询优化
CREATE INDEX idx_tags_device ON tags(device_id);
CREATE INDEX idx_tags_enabled ON tags(enabled);

-- 报警历史查询优化
CREATE INDEX idx_alert_history_fired_at ON alert_history(fired_at);
CREATE INDEX idx_alert_history_device_tag ON alert_history(device_id, tag_id);

-- 驱动配置查询优化
CREATE INDEX idx_driver_configs_protocol ON driver_configs(protocol);
CREATE INDEX idx_driver_configs_enabled ON driver_configs(enabled);
```

### 触发器自动化

**时间戳自动更新**：
```sql
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 应用到所有需要updated_at的表
CREATE TRIGGER update_devices_updated_at BEFORE UPDATE ON devices
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

### 约束设计

**数据完整性保障**：
- 外键级联删除防止孤立数据
- 唯一性约束防止重复配置
- 检查约束确保枚举值有效性
- 复合约束确保业务逻辑正确性

## 📊 扩展性考虑

### JSON配置扩展

JSONB字段支持灵活的配置扩展：
```sql
-- 设备配置示例
"config": {
  "modbus": {
    "unit_id": 1,
    "holding_registers": "40001-40100",
    "timeout_ms": 5000
  }
}

-- 驱动配置示例  
"config": {
  "connection": {
    "host": "192.168.1.100",
    "port": 502,
    "max_connections": 5
  },
  "advanced": {
    "byte_order": "big_endian",
    "word_order": "high_low"
  }
}
```

### 读写分离支持

数据访问层使用Repository模式，天然支持读写分离：
- 读操作：连接只读副本
- 写操作：连接主数据库
- 事务操作：强制主数据库

## 🔒 安全设计

### 访问控制

- **RBAC模型**：用户-角色-权限三层架构
- **资源级权限**：细化到具体操作(resource.action)
- **系统角色保护**：is_system字段防止误删关键角色

### 数据安全

- **密码安全**：bcrypt哈希存储，不存明文
- **软删除**：关键数据逻辑删除(is_active)
- **审计跟踪**：完整的创建/更新时间记录

### 输入验证

- **长度限制**：所有varchar字段定义明确长度
- **类型约束**：使用枚举类型限制输入范围
- **业务约束**：CHECK约束确保数据逻辑正确

## 🎯 部署与维护

### 迁移管理

按功能模块分离迁移文件：
```
schema/migrations/
├── 0001_init_auth.sql      # 用户认证系统
├── 0002_init_settings.sql  # 系统设置(可选)
├── 0003_init_device_tag.sql # 设备点位管理
├── 0004_driver_registry.sql # 驱动注册系统
├── 0005_alerts.sql         # 报警管理系统
├── 0006_seed_data.sql      # 基础数据
└── 0007_driver_configs.sql # 用户驱动配置
```

### 数据备份策略

```bash
# 完整备份
pg_dump -h localhost -U gateway_user gateway_db > backup_full.sql

# 仅数据备份
pg_dump -h localhost -U gateway_user --data-only gateway_db > backup_data.sql

# 仅结构备份
pg_dump -h localhost -U gateway_user --schema-only gateway_db > backup_schema.sql
```

---

**注意**：本文档基于实际代码分析，反映了Gateway_Rust项目的真实数据库设计。所有表结构、约束和索引都是基于现有迁移文件和Rust模型定义进行的准确描述。

