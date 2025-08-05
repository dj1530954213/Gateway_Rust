-- 系统设置表
CREATE TABLE IF NOT EXISTS system_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(100) UNIQUE NOT NULL,
    value JSONB,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_system_settings_key ON system_settings(key);

-- 插入默认系统设置
INSERT INTO system_settings (key, value, description) VALUES 
('system.name', '"IoT Gateway"', '系统名称'),
('system.version', '"1.0.0"', '系统版本'),
('data_retention_days', '90', '数据保留天数'),
('alert_cleanup_days', '30', '告警记录清理天数')
ON CONFLICT (key) DO NOTHING;