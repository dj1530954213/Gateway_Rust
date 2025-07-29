-- 种子数据和示例配置
-- 插入示例通知配置
INSERT INTO notification_configs (id, name, type, config, enabled) VALUES
(
    gen_random_uuid(),
    'Default Email',
    'email',
    '{"smtp_host": "localhost", "smtp_port": 587, "from": "gateway@example.com", "to": ["admin@example.com"]}'::jsonb,
    true
),
(
    gen_random_uuid(),
    'Webhook Alerts',
    'webhook',
    '{"url": "http://localhost:3000/webhook/alerts", "timeout": 10}'::jsonb,
    true
);

-- 插入示例设备（仅作演示，实际环境中应通过API创建）
INSERT INTO devices (id, name, protocol, location, endpoint, config, enabled) VALUES
(
    gen_random_uuid(),
    'Demo PLC-01',
    'ModbusTcp',
    '测试车间',
    'tcp://192.168.1.100:502',
    '{"slave_id": 1, "timeout": 5000}'::jsonb,
    false  -- 默认禁用，避免连接错误
);

-- 注意：实际标签和报警规则应通过Web界面创建
-- 这里只是预留表结构