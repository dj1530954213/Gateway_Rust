//! database_pool.rs —— PostgreSQL 连接池管理器（最小实现）
//!
//! 提供 DatabasePoolManager 封装 sqlx::Pool<Postgres>，用于健康检查与统计。

use crate::config::DatabasePoolConfig;
use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;

#[derive(Clone)]
pub struct DatabasePoolManager {
    pool: Pool<Postgres>,
    cfg: DatabasePoolConfig,
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub utilization_percent: f64,
    pub active_connections: u32,
}

impl DatabasePoolManager {
    /// 创建新的连接池管理器
    pub async fn new(dsn: &str, cfg: DatabasePoolConfig) -> Result<Self> {
        // 尽量使用 lazy 以避免启动时强依赖数据库可达
        // 注意：acquire_timeout/idle_timeout/max_lifetime 在不同版本签名可能不同，这里仅设置基础项，保证编译稳定
        let pool = PgPoolOptions::new()
            .max_connections(cfg.max_connections)
            .min_connections(cfg.min_connections)
            // .acquire_timeout(Duration::from_secs(cfg.acquire_timeout_secs))
            .connect_lazy(dsn)?;

        Ok(Self { pool, cfg })
    }

    /// 获取底层连接池引用
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// 连接池健康检查（最小实现：尝试获取一次连接）
    pub async fn health_check(&self) -> Result<bool> {
        // 对于 lazy pool，这里会真正触达数据库；失败则返回 false
        match self.pool.acquire().await {
            Ok(_conn) => Ok(true),
            Err(_e) => Ok(false),
        }
    }

    /// 获取连接池统计（最小实现：返回占位值以避免耦合 sqlx 内部指标）
    pub async fn get_pool_stats(&self) -> PoolStats {
        PoolStats {
            utilization_percent: 0.0,
            active_connections: 0,
        }
    }
}

/// 配置合法性校验（最小实现）
pub fn validate_pool_config(cfg: &DatabasePoolConfig) -> Result<()> {
    if cfg.max_connections == 0 {
        anyhow::bail!("max_connections must be > 0");
    }
    if cfg.min_connections > cfg.max_connections {
        anyhow::bail!("min_connections must be <= max_connections");
    }
    // 其他字段保持宽松，避免在迁移期造成不必要阻塞
    Ok(())
}
