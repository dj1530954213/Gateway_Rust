-- 设备与点位表结构
-- 创建枚举类型
CREATE TYPE protocol_kind AS ENUM ('ModbusTcp','OpcUa','Mqtt');
CREATE TYPE tag_data_type AS ENUM ('Float','Int','Bool','String');

-- 设备表
CREATE TABLE devices (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL UNIQUE,
    protocol    protocol_kind NOT NULL,
    location    VARCHAR(128),
    endpoint    TEXT,                    -- 连接地址，如 tcp://192.168.1.100:502
    config      JSONB,                   -- 协议特定配置
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 点位表
CREATE TABLE tags (
    id          UUID PRIMARY KEY,
    device_id   UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    name        VARCHAR(64) NOT NULL,
    address     VARCHAR(32) NOT NULL,    -- 寄存器地址，如 "40001"
    data_type   tag_data_type NOT NULL,
    scaling     DOUBLE PRECISION,        -- 比例因子
    offset      DOUBLE PRECISION,        -- 偏移量
    unit        VARCHAR(16),             -- 单位
    description TEXT,                    -- 描述
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(device_id, address)
);

-- 索引
CREATE INDEX idx_tags_device ON tags(device_id);
CREATE INDEX idx_devices_protocol ON devices(protocol);
CREATE INDEX idx_devices_enabled ON devices(enabled);
CREATE INDEX idx_tags_enabled ON tags(enabled);

-- 触发器：自动更新 updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_devices_updated_at BEFORE UPDATE ON devices
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();