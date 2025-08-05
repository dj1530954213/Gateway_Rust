//! driver_config_repo.rs —— 驱动配置仓储实现
//!
//! 实现驱动配置相关的数据库操作：
//! - 驱动配置CRUD操作
//! - 驱动配置查询与过滤
//! - 驱动配置状态管理
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::error::{RepoError, RepoResult};
use crate::models::{
    DriverConfig, NewDriverConfig, DriverConfigUpdate, DriverConfigFilter
};
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// 驱动配置仓储接口
#[async_trait]
pub trait DriverConfigRepo: Send + Sync {
    // CRUD 操作
    async fn create_driver_config(&self, config: NewDriverConfig) -> RepoResult<DriverConfig>;
    async fn get_driver_config(&self, id: Uuid) -> RepoResult<Option<DriverConfig>>;
    async fn get_driver_config_by_name(&self, name: &str) -> RepoResult<Option<DriverConfig>>;
    async fn update_driver_config(&self, id: Uuid, update: DriverConfigUpdate) -> RepoResult<Option<DriverConfig>>;
    async fn delete_driver_config(&self, id: Uuid) -> RepoResult<bool>;
    
    // 查询操作
    async fn list_driver_configs(&self, filter: DriverConfigFilter) -> RepoResult<Vec<DriverConfig>>;
    async fn count_driver_configs(&self, filter: DriverConfigFilter) -> RepoResult<i64>;
    
    // 状态操作
    async fn enable_driver_config(&self, id: Uuid, enabled: bool) -> RepoResult<bool>;
    async fn is_driver_name_exists(&self, name: &str, exclude_id: Option<Uuid>) -> RepoResult<bool>;
}

/// 驱动配置仓储PostgreSQL实现
pub struct DriverConfigRepoImpl {
    pool: Pool<Postgres>,
}

impl DriverConfigRepoImpl {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriverConfigRepo for DriverConfigRepoImpl {
    async fn create_driver_config(&self, config: NewDriverConfig) -> RepoResult<DriverConfig> {
        let result = sqlx::query_as::<_, DriverConfig>(
            r#"
            INSERT INTO driver_configs (
                name, description, protocol, connection_type, enabled, config,
                scan_interval, timeout, max_concurrent, batch_size,
                max_retries, retry_interval, exponential_backoff, max_retry_interval,
                log_level, enable_request_log, enable_response_log, max_log_size,
                enable_ssl, verify_certificate, client_cert_path, client_key_path
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            RETURNING *
            "#
        )
        .bind(&config.name)
        .bind(&config.description)
        .bind(&config.protocol)
        .bind(&config.connection_type)
        .bind(config.enabled)
        .bind(&config.config)
        .bind(config.scan_interval)
        .bind(config.timeout)
        .bind(config.max_concurrent)
        .bind(config.batch_size)
        .bind(config.max_retries)
        .bind(config.retry_interval)
        .bind(config.exponential_backoff)
        .bind(config.max_retry_interval)
        .bind(&config.log_level)
        .bind(config.enable_request_log)
        .bind(config.enable_response_log)
        .bind(config.max_log_size)
        .bind(config.enable_ssl)
        .bind(config.verify_certificate)
        .bind(&config.client_cert_path)
        .bind(&config.client_key_path)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn get_driver_config(&self, id: Uuid) -> RepoResult<Option<DriverConfig>> {
        let result = sqlx::query_as::<_, DriverConfig>(
            "SELECT * FROM driver_configs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn get_driver_config_by_name(&self, name: &str) -> RepoResult<Option<DriverConfig>> {
        let result = sqlx::query_as::<_, DriverConfig>(
            "SELECT * FROM driver_configs WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn update_driver_config(&self, id: Uuid, update: DriverConfigUpdate) -> RepoResult<Option<DriverConfig>> {
        let mut query = sqlx::QueryBuilder::new("UPDATE driver_configs SET ");
        let mut has_update = false;
        
        if let Some(name) = &update.name {
            if has_update { query.push(", "); }
            query.push("name = ");
            query.push_bind(name);
            has_update = true;
        }
        
        if let Some(description) = &update.description {
            if has_update { query.push(", "); }
            query.push("description = ");
            query.push_bind(description);
            has_update = true;
        }
        
        if let Some(protocol) = &update.protocol {
            if has_update { query.push(", "); }
            query.push("protocol = ");
            query.push_bind(protocol);
            has_update = true;
        }
        
        if let Some(connection_type) = &update.connection_type {
            if has_update { query.push(", "); }
            query.push("connection_type = ");
            query.push_bind(connection_type);
            has_update = true;
        }
        
        if let Some(enabled) = update.enabled {
            if has_update { query.push(", "); }
            query.push("enabled = ");
            query.push_bind(enabled);
            has_update = true;
        }
        
        if let Some(config) = &update.config {
            if has_update { query.push(", "); }
            query.push("config = ");
            query.push_bind(config);
            has_update = true;
        }
        
        // 添加其他字段的更新逻辑...
        if let Some(scan_interval) = update.scan_interval {
            if has_update { query.push(", "); }
            query.push("scan_interval = ");
            query.push_bind(scan_interval);
            has_update = true;
        }
        
        if let Some(timeout) = update.timeout {
            if has_update { query.push(", "); }
            query.push("timeout = ");
            query.push_bind(timeout);
            has_update = true;
        }
        
        if !has_update {
            return self.get_driver_config(id).await;
        }
        
        query.push(", updated_at = now() WHERE id = ");
        query.push_bind(id);
        query.push(" RETURNING *");
        
        let result = query
            .build_query_as::<DriverConfig>()
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(result)
    }
    
    async fn delete_driver_config(&self, id: Uuid) -> RepoResult<bool> {
        let result = sqlx::query("DELETE FROM driver_configs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn list_driver_configs(&self, filter: DriverConfigFilter) -> RepoResult<Vec<DriverConfig>> {
        let mut query = sqlx::QueryBuilder::new("SELECT * FROM driver_configs WHERE 1=1 ");
        
        if let Some(protocol) = &filter.protocol {
            query.push(" AND protocol = ");
            query.push_bind(protocol);
        }
        
        if let Some(enabled) = filter.enabled {
            query.push(" AND enabled = ");
            query.push_bind(enabled);
        }
        
        if let Some(name_contains) = &filter.name_contains {
            query.push(" AND name ILIKE ");
            query.push_bind(format!("%{}%", name_contains));
        }
        
        query.push(" ORDER BY created_at DESC");
        
        if let Some(limit) = filter.limit {
            query.push(" LIMIT ");
            query.push_bind(limit);
        }
        
        if let Some(offset) = filter.offset {
            query.push(" OFFSET ");
            query.push_bind(offset);
        }
        
        let result = query
            .build_query_as::<DriverConfig>()
            .fetch_all(&self.pool)
            .await?;
        
        Ok(result)
    }
    
    async fn count_driver_configs(&self, filter: DriverConfigFilter) -> RepoResult<i64> {
        let mut query = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM driver_configs WHERE 1=1 ");
        
        if let Some(protocol) = &filter.protocol {
            query.push(" AND protocol = ");
            query.push_bind(protocol);
        }
        
        if let Some(enabled) = filter.enabled {
            query.push(" AND enabled = ");
            query.push_bind(enabled);
        }
        
        if let Some(name_contains) = &filter.name_contains {
            query.push(" AND name ILIKE ");
            query.push_bind(format!("%{}%", name_contains));
        }
        
        let result: i64 = query
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await?;
        
        Ok(result)
    }
    
    async fn enable_driver_config(&self, id: Uuid, enabled: bool) -> RepoResult<bool> {
        let result = sqlx::query(
            "UPDATE driver_configs SET enabled = $1, updated_at = now() WHERE id = $2"
        )
        .bind(enabled)
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn is_driver_name_exists(&self, name: &str, exclude_id: Option<Uuid>) -> RepoResult<bool> {
        let result = if let Some(exclude_id) = exclude_id {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM driver_configs WHERE name = $1 AND id != $2"
            )
            .bind(name)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM driver_configs WHERE name = $1"
            )
            .bind(name)
            .fetch_one(&self.pool)
            .await?
        };
        
        Ok(result > 0)
    }
}