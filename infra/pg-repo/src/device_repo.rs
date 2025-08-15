//! device_repo.rs —— 设备仓储实现
//!
//! 实现设备相关的数据库操作：
//! - CRUD操作：创建、读取、更新、删除设备
//! - 查询过滤：按协议、状态等条件查询
//! - 分页支持：列表查询支持分页
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::error::RepoResult;
use crate::models::{Device, DeviceFilter, DeviceUpdate, NewDevice};
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use tracing::{debug, warn};

/// 设备仓储接口
#[async_trait]
pub trait DeviceRepo: Send + Sync {
    /// 创建设备
    async fn create(&self, device: NewDevice) -> RepoResult<Device>;
    
    /// 根据ID获取设备
    async fn get_by_id(&self, id: Uuid) -> RepoResult<Option<Device>>;
    
    /// 根据名称获取设备
    async fn get_by_name(&self, name: &str) -> RepoResult<Option<Device>>;
    
    /// 更新设备
    async fn update(&self, id: Uuid, update: DeviceUpdate) -> RepoResult<Option<Device>>;
    
    /// 删除设备
    async fn delete(&self, id: Uuid) -> RepoResult<bool>;
    
    /// 列表查询（带过滤和分页）
    async fn list(&self, filter: DeviceFilter) -> RepoResult<Vec<Device>>;
    
    /// 计算总数（用于分页）
    async fn count(&self, filter: DeviceFilter) -> RepoResult<i64>;
    
    /// 检查设备是否存在
    async fn exists(&self, id: Uuid) -> RepoResult<bool>;
    
    /// 检查设备名称是否已存在
    async fn name_exists(&self, name: &str, exclude_id: Option<Uuid>) -> RepoResult<bool>;
}

/// 设备仓储PostgreSQL实现
pub struct DeviceRepoImpl {
    pool: Pool<Postgres>,
}

impl DeviceRepoImpl {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
    
    /// 执行带监控的查询
    #[inline]
    async fn execute_monitored<T, F, Fut>(&self, 
        query_name: &'static str,
        query_fn: F
    ) -> RepoResult<T>
    where
        F: FnOnce(&Pool<Postgres>) -> Fut,
        Fut: std::future::Future<Output = Result<T, sqlx::Error>>,
    {
        let start = std::time::Instant::now();
        let result = query_fn(&self.pool).await;
        let elapsed = start.elapsed();
        
        // 记录性能指标
        tracing::debug!("Query {} executed in {:?}", query_name, elapsed);
        
        // 记录慢查询
        if elapsed.as_millis() > 1000 {
            tracing::warn!("Slow query detected: {} took {:?}", query_name, elapsed);
        }
        
        result.map_err(Into::into)
    }
}

#[async_trait]
impl DeviceRepo for DeviceRepoImpl {
    async fn create(&self, device: NewDevice) -> RepoResult<Device> {
        self.execute_monitored("device_create", |pool| async move {
            sqlx::query_as::<_, Device>(
                r#"
                INSERT INTO devices (id, name, protocol, location, endpoint, config, enabled)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, name, protocol, location, endpoint, config, enabled, created_at, updated_at
                "#
            )
            .bind(device.id)
            .bind(device.name)
            .bind(device.protocol)
            .bind(device.location)
            .bind(device.endpoint)
            .bind(device.config)
            .bind(device.enabled)
            .fetch_one(pool)
            .await
        }).await
    }
    
    async fn get_by_id(&self, id: Uuid) -> RepoResult<Option<Device>> {
        self.execute_monitored("device_get_by_id", |pool| async move {
            sqlx::query_as::<_, Device>(
                r#"
                SELECT id, name, protocol, location, endpoint, config, enabled, created_at, updated_at
                FROM devices 
                WHERE id = $1
                "#
            )
            .bind(id)
            .fetch_optional(pool)
            .await
        }).await
    }
    
    async fn get_by_name(&self, name: &str) -> RepoResult<Option<Device>> {
        let result = sqlx::query_as::<_, Device>(
            r#"
            SELECT id, name, protocol, location, endpoint, config, enabled, created_at, updated_at
            FROM devices 
            WHERE name = $1
            "#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn update(&self, id: Uuid, update: DeviceUpdate) -> RepoResult<Option<Device>> {
        let result = sqlx::query_as::<_, Device>(
            r#"
            UPDATE devices 
            SET 
                name = COALESCE($2, name),
                location = COALESCE($3, location), 
                endpoint = COALESCE($4, endpoint),
                config = COALESCE($5, config),
                enabled = COALESCE($6, enabled),
                updated_at = now()
            WHERE id = $1
            RETURNING id, name, protocol, location, endpoint, config, enabled, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(update.name)
        .bind(update.location)
        .bind(update.endpoint)
        .bind(update.config)
        .bind(update.enabled)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn delete(&self, id: Uuid) -> RepoResult<bool> {
        let result = sqlx::query("DELETE FROM devices WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn list(&self, filter: DeviceFilter) -> RepoResult<Vec<Device>> {
        self.execute_monitored("device_list", |pool| async move {
            let mut query_builder = sqlx::QueryBuilder::new(
                r#"
                SELECT id, name, protocol, location, endpoint, config, enabled, created_at, updated_at
                FROM devices 
                WHERE 1=1
                "#
            );
            
            if let Some(protocol) = filter.protocol {
                query_builder.push(" AND protocol = ").push_bind(protocol);
            }
            
            if let Some(enabled) = filter.enabled {
                query_builder.push(" AND enabled = ").push_bind(enabled);
            }
            
            query_builder.push(" ORDER BY created_at DESC");
            
            if let Some(limit) = filter.limit {
                query_builder.push(" LIMIT ").push_bind(limit);
            }
            
            if let Some(offset) = filter.offset {
                query_builder.push(" OFFSET ").push_bind(offset);
            }
            
            query_builder
                .build_query_as::<Device>()
                .fetch_all(pool)
                .await
        }).await
    }
    
    async fn count(&self, filter: DeviceFilter) -> RepoResult<i64> {
        let mut query_builder = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM devices WHERE 1=1");
        
        if let Some(protocol) = filter.protocol {
            query_builder.push(" AND protocol = ").push_bind(protocol);
        }
        
        if let Some(enabled) = filter.enabled {
            query_builder.push(" AND enabled = ").push_bind(enabled);
        }
        
        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(&self.pool)
            .await?;
        
        Ok(count.0)
    }
    
    async fn exists(&self, id: Uuid) -> RepoResult<bool> {
        let result = sqlx::query("SELECT 1 FROM devices WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(result.is_some())
    }
    
    async fn name_exists(&self, name: &str, exclude_id: Option<Uuid>) -> RepoResult<bool> {
        let result = if let Some(exclude_id) = exclude_id {
            sqlx::query("SELECT 1 FROM devices WHERE name = $1 AND id != $2")
                .bind(name)
                .bind(exclude_id)
                .fetch_optional(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT 1 FROM devices WHERE name = $1")
                .bind(name)
                .fetch_optional(&self.pool)
                .await?
        };
        
        Ok(result.is_some())
    }
}