-- 驱动配置表 - 存储用户通过表单创建的驱动配置
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

-- 索引
CREATE INDEX idx_driver_configs_protocol ON driver_configs(protocol);
CREATE INDEX idx_driver_configs_enabled ON driver_configs(enabled);
CREATE INDEX idx_driver_configs_created_at ON driver_configs(created_at);

-- 更新时间触发器
CREATE OR REPLACE FUNCTION update_driver_configs_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_driver_configs_updated_at
    BEFORE UPDATE ON driver_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_driver_configs_updated_at();