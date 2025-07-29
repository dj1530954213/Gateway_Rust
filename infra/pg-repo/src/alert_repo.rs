//! alert_repo.rs —— 报警仓储实现
//!
//! 实现报警相关的数据库操作：
//! - 报警规则CRUD
//! - 报警历史记录
//! - 查询过滤与分页
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::error::{RepoError, RepoResult};
use crate::models::{
    AlertHistory, AlertHistoryFilter, AlertRule, AlertRuleUpdate, 
    NewAlertHistory, NewAlertRule
};
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// 报警仓储接口
#[async_trait]
pub trait AlertRepo: Send + Sync {
    // 报警规则相关
    async fn create_rule(&self, rule: NewAlertRule) -> RepoResult<AlertRule>;
    async fn get_rule_by_id(&self, id: Uuid) -> RepoResult<Option<AlertRule>>;
    async fn update_rule(&self, id: Uuid, update: AlertRuleUpdate) -> RepoResult<Option<AlertRule>>;
    async fn delete_rule(&self, id: Uuid) -> RepoResult<bool>;
    async fn list_rules(&self, enabled_only: Option<bool>) -> RepoResult<Vec<AlertRule>>;
    
    // 报警历史相关
    async fn create_history(&self, history: NewAlertHistory) -> RepoResult<AlertHistory>;
    async fn get_history_by_id(&self, id: Uuid) -> RepoResult<Option<AlertHistory>>;
    async fn list_history(&self, filter: AlertHistoryFilter) -> RepoResult<Vec<AlertHistory>>;
    async fn count_history(&self, filter: AlertHistoryFilter) -> RepoResult<i64>;
    async fn resolve_alert(&self, history_id: Uuid) -> RepoResult<bool>;
}

/// 报警仓储PostgreSQL实现
pub struct AlertRepoImpl {
    pool: Pool<Postgres>,
}

impl AlertRepoImpl {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AlertRepo for AlertRepoImpl {
    async fn create_rule(&self, rule: NewAlertRule) -> RepoResult<AlertRule> {
        let result = sqlx::query_as::<_, AlertRule>(
            r#"
            INSERT INTO alert_rules (id, name, description, device_id, tag_id, op, threshold, level, eval_every, eval_for, enabled, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, name, description, device_id, tag_id, op, threshold, level, eval_every, eval_for, enabled, created_by, created_at, updated_at
            "#
        )
        .bind(rule.id)
        .bind(rule.name)
        .bind(rule.description)
        .bind(rule.device_id)
        .bind(rule.tag_id)
        .bind(rule.op)
        .bind(rule.threshold)
        .bind(rule.level)
        .bind(rule.eval_every)
        .bind(rule.eval_for)
        .bind(rule.enabled)
        .bind(rule.created_by)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn get_rule_by_id(&self, id: Uuid) -> RepoResult<Option<AlertRule>> {
        let result = sqlx::query_as::<_, AlertRule>(
            r#"
            SELECT id, name, description, device_id, tag_id, op, threshold, level, eval_every, eval_for, enabled, created_by, created_at, updated_at
            FROM alert_rules 
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn update_rule(&self, id: Uuid, update: AlertRuleUpdate) -> RepoResult<Option<AlertRule>> {
        // 简化实现：只更新基本字段
        if update.name.is_none() && update.description.is_none() && update.op.is_none()
            && update.threshold.is_none() && update.level.is_none() && update.enabled.is_none() {
            return self.get_rule_by_id(id).await;
        }
        
        let mut query_builder = sqlx::QueryBuilder::new("UPDATE alert_rules SET updated_at = now()");
        
        if let Some(name) = &update.name {
            query_builder.push(", name = ").push_bind(name);
        }
        if let Some(description) = &update.description {
            query_builder.push(", description = ").push_bind(description);
        }
        if let Some(op) = &update.op {
            query_builder.push(", op = ").push_bind(op);
        }
        if let Some(threshold) = update.threshold {
            query_builder.push(", threshold = ").push_bind(threshold);
        }
        if let Some(level) = &update.level {
            query_builder.push(", level = ").push_bind(level);
        }
        if let Some(enabled) = update.enabled {
            query_builder.push(", enabled = ").push_bind(enabled);
        }
        
        query_builder.push(" WHERE id = ").push_bind(id);
        query_builder.push(" RETURNING id, name, description, device_id, tag_id, op, threshold, level, eval_every, eval_for, enabled, created_by, created_at, updated_at");
        
        let result = query_builder
            .build_query_as::<AlertRule>()
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(result)
    }
    
    async fn delete_rule(&self, id: Uuid) -> RepoResult<bool> {
        let result = sqlx::query(
            "DELETE FROM alert_rules WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn list_rules(&self, enabled_only: Option<bool>) -> RepoResult<Vec<AlertRule>> {
        let result = if let Some(enabled) = enabled_only {
            sqlx::query_as::<_, AlertRule>(
                r#"
                SELECT id, name, description, device_id, tag_id, 
                       op, threshold, level, 
                       eval_every, eval_for, enabled, created_by, created_at, updated_at
                FROM alert_rules 
                WHERE enabled = $1
                ORDER BY created_at DESC
                "#
            )
            .bind(enabled)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, AlertRule>(
                r#"
                SELECT id, name, description, device_id, tag_id, 
                       op, threshold, level, 
                       eval_every, eval_for, enabled, created_by, created_at, updated_at
                FROM alert_rules 
                ORDER BY created_at DESC
                "#
            )
            .fetch_all(&self.pool)
            .await?
        };
        
        Ok(result)
    }
    
    async fn create_history(&self, history: NewAlertHistory) -> RepoResult<AlertHistory> {
        let result = sqlx::query_as::<_, AlertHistory>(
            r#"
            INSERT INTO alert_history (id, rule_id, device_id, tag_id, value, threshold, level, message, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, rule_id, device_id, tag_id, fired_at, resolved_at, 
                     value, threshold, level, message, status, duration
            "#
        )
        .bind(history.id)
        .bind(history.rule_id)
        .bind(history.device_id)
        .bind(history.tag_id)
        .bind(history.value)
        .bind(history.threshold)
        .bind(history.level)
        .bind(history.message)
        .bind(history.status)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn get_history_by_id(&self, id: Uuid) -> RepoResult<Option<AlertHistory>> {
        let result = sqlx::query_as::<_, AlertHistory>(
            r#"
            SELECT id, rule_id, device_id, tag_id, fired_at, resolved_at, 
                   value, threshold, level, message, status, duration
            FROM alert_history 
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn list_history(&self, filter: AlertHistoryFilter) -> RepoResult<Vec<AlertHistory>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            SELECT id, rule_id, device_id, tag_id, fired_at, resolved_at, 
                   value, threshold, level, message, status, duration
            FROM alert_history 
            WHERE 1=1
            "#
        );
        
        if let Some(device_id) = filter.device_id {
            query_builder.push(" AND device_id = ").push_bind(device_id);
        }
        
        if let Some(tag_id) = filter.tag_id {
            query_builder.push(" AND tag_id = ").push_bind(tag_id);
        }
        
        if let Some(level) = filter.level {
            query_builder.push(" AND level = ").push_bind(level);
        }
        
        if let Some(status) = &filter.status {
            query_builder.push(" AND status = ").push_bind(status);
        }
        
        if let Some(from) = filter.from {
            query_builder.push(" AND fired_at >= ").push_bind(from);
        }
        
        if let Some(to) = filter.to {
            query_builder.push(" AND fired_at <= ").push_bind(to);
        }
        
        query_builder.push(" ORDER BY fired_at DESC");
        
        if let Some(limit) = filter.limit {
            query_builder.push(" LIMIT ").push_bind(limit);
        }
        
        if let Some(offset) = filter.offset {
            query_builder.push(" OFFSET ").push_bind(offset);
        }
        
        let history = query_builder
            .build_query_as::<AlertHistory>()
            .fetch_all(&self.pool)
            .await?;
        
        Ok(history)
    }
    
    async fn count_history(&self, filter: AlertHistoryFilter) -> RepoResult<i64> {
        let mut query_builder = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM alert_history WHERE 1=1");
        
        if let Some(device_id) = filter.device_id {
            query_builder.push(" AND device_id = ").push_bind(device_id);
        }
        
        if let Some(tag_id) = filter.tag_id {
            query_builder.push(" AND tag_id = ").push_bind(tag_id);
        }
        
        if let Some(level) = filter.level {
            query_builder.push(" AND level = ").push_bind(level);
        }
        
        if let Some(status) = &filter.status {
            query_builder.push(" AND status = ").push_bind(status);
        }
        
        if let Some(from) = filter.from {
            query_builder.push(" AND fired_at >= ").push_bind(from);
        }
        
        if let Some(to) = filter.to {
            query_builder.push(" AND fired_at <= ").push_bind(to);
        }
        
        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(&self.pool)
            .await?;
        
        Ok(count.0)
    }
    
    async fn resolve_alert(&self, history_id: Uuid) -> RepoResult<bool> {
        let result = sqlx::query(
            "UPDATE alert_history SET resolved_at = now(), status = 'resolved' WHERE id = $1 AND status = 'firing'"
        )
        .bind(history_id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
}