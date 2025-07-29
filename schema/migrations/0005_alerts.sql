-- 报警系统表结构
-- 创建枚举类型
CREATE TYPE compare_op AS ENUM ('GT','LT','GTE','LTE','EQ','NE');
CREATE TYPE alert_level AS ENUM ('INFO','WARN','CRIT');
CREATE TYPE notification_type AS ENUM ('email','webhook','websocket');

-- 报警规则表
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

-- 报警历史表
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

-- 通知配置表
CREATE TABLE notification_configs (
    id           UUID PRIMARY KEY,
    name         VARCHAR(64) NOT NULL UNIQUE,
    type         notification_type NOT NULL,
    config       JSONB NOT NULL,          -- 通知配置(邮件地址、webhook URL等)
    enabled      BOOLEAN NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 规则与通知配置关联表
CREATE TABLE alert_rule_notifications (
    rule_id      UUID NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    config_id    UUID NOT NULL REFERENCES notification_configs(id) ON DELETE CASCADE,
    PRIMARY KEY (rule_id, config_id)
);

-- 索引
CREATE INDEX idx_alert_rules_device ON alert_rules(device_id);
CREATE INDEX idx_alert_rules_tag ON alert_rules(tag_id);
CREATE INDEX idx_alert_rules_enabled ON alert_rules(enabled);
CREATE INDEX idx_alert_history_rule ON alert_history(rule_id);
CREATE INDEX idx_alert_history_fired_at ON alert_history(fired_at);
CREATE INDEX idx_alert_history_status ON alert_history(status);
CREATE INDEX idx_alert_history_device_tag ON alert_history(device_id, tag_id);

-- 触发器：自动更新 updated_at
CREATE TRIGGER update_alert_rules_updated_at BEFORE UPDATE ON alert_rules
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();