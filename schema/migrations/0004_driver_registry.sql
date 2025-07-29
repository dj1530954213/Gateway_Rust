-- 驱动注册表
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

-- 驱动加载历史表（用于审计）
CREATE TABLE driver_load_history (
    id          UUID PRIMARY KEY,
    protocol    protocol_kind NOT NULL,
    version     VARCHAR(32) NOT NULL,
    action      VARCHAR(16) NOT NULL CHECK (action IN ('load', 'reload', 'unload')),
    file_path   TEXT,
    result      VARCHAR(16) NOT NULL CHECK (result IN ('success', 'failed')),
    error_msg   TEXT,
    loaded_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_driver_history_protocol ON driver_load_history(protocol);
CREATE INDEX idx_driver_history_loaded_at ON driver_load_history(loaded_at);