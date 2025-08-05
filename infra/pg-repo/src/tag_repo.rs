//! tag_repo.rs —— 点位仓储实现
//!
//! 实现点位相关的数据库操作：
//! - CRUD操作：创建、读取、更新、删除点位
//! - 关联查询：按设备ID查询点位
//! - 地址唯一性：同设备内地址不重复
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::error::{RepoError, RepoResult};
use crate::models::{NewTag, Tag, TagFilter, TagUpdate};
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// 点位仓储接口
#[async_trait]
pub trait TagRepo: Send + Sync {
    /// 创建点位
    async fn create(&self, tag: NewTag) -> RepoResult<Tag>;
    
    /// 根据ID获取点位
    async fn get_by_id(&self, id: Uuid) -> RepoResult<Option<Tag>>;
    
    /// 更新点位
    async fn update(&self, id: Uuid, update: TagUpdate) -> RepoResult<Option<Tag>>;
    
    /// 删除点位
    async fn delete(&self, id: Uuid) -> RepoResult<bool>;
    
    /// 列表查询（带过滤和分页）
    async fn list(&self, filter: TagFilter) -> RepoResult<Vec<Tag>>;
    
    /// 按设备ID获取所有点位
    async fn list_by_device(&self, device_id: Uuid) -> RepoResult<Vec<Tag>>;
    
    /// 计算总数（用于分页）
    async fn count(&self, filter: TagFilter) -> RepoResult<i64>;
    
    /// 检查点位是否存在
    async fn exists(&self, id: Uuid) -> RepoResult<bool>;
    
    /// 检查设备内地址是否已存在
    async fn address_exists(&self, device_id: Uuid, address: &str, exclude_id: Option<Uuid>) -> RepoResult<bool>;
    
    /// 批量删除设备的所有点位
    async fn delete_by_device(&self, device_id: Uuid) -> RepoResult<u64>;
}

/// 点位仓储PostgreSQL实现
pub struct TagRepoImpl {
    pool: Pool<Postgres>,
}

impl TagRepoImpl {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagRepo for TagRepoImpl {
    async fn create(&self, tag: NewTag) -> RepoResult<Tag> {
        let result = sqlx::query_as::<_, Tag>(
            r#"
            INSERT INTO tags (id, device_id, name, address, data_type, scaling, "offset", unit, description, enabled)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, device_id, name, address, data_type, scaling, "offset", unit, description, enabled, created_at
            "#
        )
        .bind(tag.id)
        .bind(tag.device_id)
        .bind(tag.name)
        .bind(tag.address)
        .bind(tag.data_type)
        .bind(tag.scaling)
        .bind(tag.offset)
        .bind(tag.unit)
        .bind(tag.description)
        .bind(tag.enabled)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn get_by_id(&self, id: Uuid) -> RepoResult<Option<Tag>> {
        let result = sqlx::query_as::<_, Tag>(
            r#"
            SELECT id, device_id, name, address, data_type, scaling, "offset", unit, description, enabled, created_at
            FROM tags 
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn update(&self, id: Uuid, update: TagUpdate) -> RepoResult<Option<Tag>> {
        let result = sqlx::query_as::<_, Tag>(
            r#"
            UPDATE tags 
            SET 
                name = COALESCE($2, name),
                address = COALESCE($3, address),
                data_type = COALESCE($4, data_type),
                scaling = COALESCE($5, scaling),
                "offset" = COALESCE($6, "offset"),
                unit = COALESCE($7, unit),
                description = COALESCE($8, description),
                enabled = COALESCE($9, enabled)
            WHERE id = $1
            RETURNING id, device_id, name, address, data_type, scaling, "offset", unit, description, enabled, created_at
            "#
        )
        .bind(id)
        .bind(update.name)
        .bind(update.address)
        .bind(update.data_type)
        .bind(update.scaling)
        .bind(update.offset)
        .bind(update.unit)
        .bind(update.description)
        .bind(update.enabled)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn delete(&self, id: Uuid) -> RepoResult<bool> {
        let result = sqlx::query("DELETE FROM tags WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn list(&self, filter: TagFilter) -> RepoResult<Vec<Tag>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            SELECT id, device_id, name, address, data_type, scaling, "offset", unit, description, enabled, created_at
            FROM tags 
            WHERE 1=1
            "#
        );
        
        if let Some(device_id) = filter.device_id {
            query_builder.push(" AND device_id = ").push_bind(device_id);
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
        
        let tags = query_builder
            .build_query_as::<Tag>()
            .fetch_all(&self.pool)
            .await?;
        
        Ok(tags)
    }
    
    async fn list_by_device(&self, device_id: Uuid) -> RepoResult<Vec<Tag>> {
        let tags = sqlx::query_as::<_, Tag>(
            r#"
            SELECT id, device_id, name, address, data_type, scaling, "offset", unit, description, enabled, created_at
            FROM tags 
            WHERE device_id = $1 
            ORDER BY name
            "#
        )
        .bind(device_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(tags)
    }
    
    async fn count(&self, filter: TagFilter) -> RepoResult<i64> {
        let mut query_builder = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM tags WHERE 1=1");
        
        if let Some(device_id) = filter.device_id {
            query_builder.push(" AND device_id = ").push_bind(device_id);
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
        let result = sqlx::query("SELECT 1 FROM tags WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(result.is_some())
    }
    
    async fn address_exists(&self, device_id: Uuid, address: &str, exclude_id: Option<Uuid>) -> RepoResult<bool> {
        let result = if let Some(exclude_id) = exclude_id {
            sqlx::query("SELECT 1 FROM tags WHERE device_id = $1 AND address = $2 AND id != $3")
                .bind(device_id)
                .bind(address)
                .bind(exclude_id)
                .fetch_optional(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT 1 FROM tags WHERE device_id = $1 AND address = $2")
                .bind(device_id)
                .bind(address)
                .fetch_optional(&self.pool)
                .await?
        };
        
        Ok(result.is_some())
    }
    
    async fn delete_by_device(&self, device_id: Uuid) -> RepoResult<u64> {
        let result = sqlx::query("DELETE FROM tags WHERE device_id = $1")
            .bind(device_id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected())
    }
}