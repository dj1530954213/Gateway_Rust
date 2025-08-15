//! driver_repo.rs —— 驱动仓储实现
//!
//! 实现驱动注册表相关的数据库操作：
//! - 驱动注册与状态管理
//! - 加载历史记录
//! - 协议查询与版本比较
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::error::RepoResult;
use crate::models::{
    DbProtocolKind, DriverLoadHistory, DriverRegistry, 
    NewDriverLoadHistory, NewDriverRegistry
};
use async_trait::async_trait;
use sqlx::{Pool, Postgres};

/// 驱动仓储接口
#[async_trait]
pub trait DriverRepo: Send + Sync {
    // 驱动注册表相关
    async fn register_driver(&self, driver: NewDriverRegistry) -> RepoResult<DriverRegistry>;
    async fn get_driver(&self, protocol: DbProtocolKind) -> RepoResult<Option<DriverRegistry>>;
    async fn update_driver_status(&self, protocol: DbProtocolKind, status: &str) -> RepoResult<bool>;
    async fn unregister_driver(&self, protocol: DbProtocolKind) -> RepoResult<bool>;
    async fn list_drivers(&self) -> RepoResult<Vec<DriverRegistry>>;
    
    // 加载历史相关
    async fn add_load_history(&self, history: NewDriverLoadHistory) -> RepoResult<DriverLoadHistory>;
    async fn get_load_history(&self, protocol: Option<DbProtocolKind>, limit: Option<i64>) -> RepoResult<Vec<DriverLoadHistory>>;
    
    // 工具方法
    async fn is_driver_loaded(&self, protocol: DbProtocolKind) -> RepoResult<bool>;
    async fn get_driver_version(&self, protocol: DbProtocolKind) -> RepoResult<Option<String>>;
}

/// 驱动仓储PostgreSQL实现
pub struct DriverRepoImpl {
    pool: Pool<Postgres>,
}

impl DriverRepoImpl {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriverRepo for DriverRepoImpl {
    async fn register_driver(&self, driver: NewDriverRegistry) -> RepoResult<DriverRegistry> {
        let result = sqlx::query_as::<_, DriverRegistry>(
            r#"
            INSERT INTO driver_registry (protocol, version, file_path, file_hash, api_version, metadata, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (protocol) 
            DO UPDATE SET 
                version = EXCLUDED.version,
                file_path = EXCLUDED.file_path,
                file_hash = EXCLUDED.file_hash,
                api_version = EXCLUDED.api_version,
                metadata = EXCLUDED.metadata,
                status = EXCLUDED.status,
                loaded_at = now()
            RETURNING protocol, version, file_path, file_hash, api_version, metadata, loaded_at, status
            "#
        )
        .bind(driver.protocol)
        .bind(driver.version)
        .bind(driver.file_path)
        .bind(driver.file_hash)
        .bind(driver.api_version)
        .bind(driver.metadata)
        .bind(driver.status)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn get_driver(&self, protocol: DbProtocolKind) -> RepoResult<Option<DriverRegistry>> {
        let result = sqlx::query_as::<_, DriverRegistry>(
            r#"
            SELECT protocol, version, file_path, file_hash, api_version, metadata, loaded_at, status
            FROM driver_registry 
            WHERE protocol = $1
            "#
        )
        .bind(protocol)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn update_driver_status(&self, protocol: DbProtocolKind, status: &str) -> RepoResult<bool> {
        let result = sqlx::query(
            "UPDATE driver_registry SET status = $1, loaded_at = now() WHERE protocol = $2"
        )
        .bind(status)
        .bind(protocol)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn unregister_driver(&self, protocol: DbProtocolKind) -> RepoResult<bool> {
        let result = sqlx::query(
            "DELETE FROM driver_registry WHERE protocol = $1"
        )
        .bind(protocol)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn list_drivers(&self) -> RepoResult<Vec<DriverRegistry>> {
        let result = sqlx::query_as::<_, DriverRegistry>(
            r#"
            SELECT protocol, version, file_path, file_hash, api_version, metadata, loaded_at, status
            FROM driver_registry 
            ORDER BY loaded_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn add_load_history(&self, history: NewDriverLoadHistory) -> RepoResult<DriverLoadHistory> {
        let result = sqlx::query_as::<_, DriverLoadHistory>(
            r#"
            INSERT INTO driver_load_history (id, protocol, version, action, file_path, result, error_msg)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, protocol, version, action, file_path, result, error_msg, loaded_at
            "#
        )
        .bind(history.id)
        .bind(history.protocol)
        .bind(history.version)
        .bind(history.action)
        .bind(history.file_path)
        .bind(history.result)
        .bind(history.error_msg)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn get_load_history(&self, protocol: Option<DbProtocolKind>, limit: Option<i64>) -> RepoResult<Vec<DriverLoadHistory>> {
        let limit = limit.unwrap_or(50);
        
        let result = if let Some(protocol) = protocol {
            sqlx::query_as::<_, DriverLoadHistory>(
                r#"
                SELECT id, protocol, version, action, file_path, result, error_msg, loaded_at
                FROM driver_load_history 
                WHERE protocol = $1
                ORDER BY loaded_at DESC
                LIMIT $2
                "#
            )
            .bind(protocol)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, DriverLoadHistory>(
                r#"
                SELECT id, protocol, version, action, file_path, result, error_msg, loaded_at
                FROM driver_load_history 
                ORDER BY loaded_at DESC
                LIMIT $1
                "#
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };
        
        Ok(result)
    }
    
    async fn is_driver_loaded(&self, protocol: DbProtocolKind) -> RepoResult<bool> {
        let result = sqlx::query_scalar::<_, i32>(
            "SELECT 1 FROM driver_registry WHERE protocol = $1 AND status = 'loaded'"
        )
        .bind(protocol)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result.is_some())
    }
    
    async fn get_driver_version(&self, protocol: DbProtocolKind) -> RepoResult<Option<String>> {
        let result = sqlx::query_scalar::<_, String>(
            "SELECT version FROM driver_registry WHERE protocol = $1"
        )
        .bind(protocol)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
}