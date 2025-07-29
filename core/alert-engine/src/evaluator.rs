//! evaluator.rs —— 报警规则评估器
//!
//! 核心职责：
//! - 接收遥测数据帧
//! - 加载并缓存活跃规则
//! - 执行规则评估逻辑
//! - 生成报警事件
//! - 管理报警状态（触发/解决）
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::{AlertError, AlertResult};
use crate::models::{
    AlertRule, AlertEvent, TelemetryFrame, EvaluationContext, 
    AlertEventStatus, CompareOperator, AlertLevel
};
use chrono::{DateTime, Utc, Duration};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use metrics::{counter, histogram, gauge};

/// 规则评估器
pub struct RuleEvaluator {
    db_pool: PgPool,
    /// 缓存的活跃规则 (rule_id -> rule)
    active_rules: Arc<RwLock<HashMap<Uuid, AlertRule>>>,
    /// 正在触发的事件 (rule_id -> event)
    firing_events: Arc<RwLock<HashMap<Uuid, AlertEvent>>>,
    /// 规则最后评估时间
    last_evaluation: Arc<RwLock<HashMap<Uuid, DateTime<Utc>>>>,
    /// 数据历史缓存 (device_id:tag_id -> values)
    value_history: Arc<RwLock<HashMap<String, HistoryBuffer>>>,
}

/// 历史数据缓冲区
#[derive(Debug, Clone)]
struct HistoryBuffer {
    values: Vec<(DateTime<Utc>, f64)>,
    max_size: usize,
}

impl HistoryBuffer {
    fn new(max_size: usize) -> Self {
        Self {
            values: Vec::with_capacity(max_size),
            max_size,
        }
    }
    
    fn push(&mut self, timestamp: DateTime<Utc>, value: f64) {
        self.values.push((timestamp, value));
        
        // 保持缓冲区大小限制
        if self.values.len() > self.max_size {
            self.values.remove(0);
        }
        
        // 按时间排序
        self.values.sort_by_key(|(ts, _)| *ts);
    }
    
    fn get_recent_values(&self, duration: Duration) -> Vec<f64> {
        let cutoff = Utc::now() - duration;
        self.values
            .iter()
            .filter(|(ts, _)| *ts >= cutoff)
            .map(|(_, v)| *v)
            .collect()
    }
    
    fn latest_value(&self) -> Option<f64> {
        self.values.last().map(|(_, v)| *v)
    }
}

impl RuleEvaluator {
    /// 创建新的规则评估器
    pub async fn new(db_pool: PgPool) -> AlertResult<Self> {
        let evaluator = Self {
            db_pool,
            active_rules: Arc::new(RwLock::new(HashMap::new())),
            firing_events: Arc::new(RwLock::new(HashMap::new())),
            last_evaluation: Arc::new(RwLock::new(HashMap::new())),
            value_history: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // 初始化时加载活跃规则
        evaluator.reload_rules().await?;
        
        Ok(evaluator)
    }
    
    /// 重新加载活跃规则
    pub async fn reload_rules(&self) -> AlertResult<()> {
        info!("Reloading active alert rules...");
        
        let start_time = std::time::Instant::now();
        
        // 从数据库加载活跃规则
        let rules = sqlx::query_as!(
            AlertRule,
            r#"
            SELECT 
                id, name, description, device_id, tag_id,
                operator as "operator: CompareOperator",
                threshold, level as "level: AlertLevel",
                eval_every, eval_for, enabled,
                notification_channels, silence_duration,
                created_by, created_at, updated_at,
                last_fired_at, fire_count
            FROM alert_rules 
            WHERE enabled = true
            "#
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to load alert rules: {}", e)))?;
        
        // 更新缓存
        let mut active_rules = self.active_rules.write().await;
        active_rules.clear();
        
        for rule in rules {
            active_rules.insert(rule.id, rule);
        }
        
        let load_duration = start_time.elapsed();
        histogram!("alert_rule_reload_duration_seconds").record(load_duration.as_secs_f64());
        gauge!("alert_active_rules_total").set(active_rules.len() as f64);
        
        info!("Loaded {} active alert rules in {:?}", active_rules.len(), load_duration);
        
        Ok(())
    }
    
    /// 处理遥测数据帧
    pub async fn process_telemetry_frame(&self, frame: TelemetryFrame) -> AlertResult<Vec<AlertEvent>> {
        debug!(
            "Processing telemetry frame: device={}, tag={}, value={}", 
            frame.device_id, frame.tag_id, frame.value
        );
        
        counter!("alert_telemetry_frames_processed_total").increment(1);
        
        // 更新历史数据缓存
        self.update_value_history(&frame).await;
        
        let mut triggered_events = Vec::new();
        let active_rules = self.active_rules.read().await;
        
        // 遍历所有活跃规则
        for rule in active_rules.values() {
            // 检查规则是否匹配此数据帧
            if !rule.matches(&frame) {
                continue;
            }
            
            // 检查评估间隔
            if !self.should_evaluate_rule(rule).await {
                continue;
            }
            
            // 构建评估上下文
            let context = self.build_evaluation_context(&frame, rule).await?;
            
            // 执行规则评估
            match self.evaluate_rule(rule, &context).await {
                Ok(Some(event)) => {
                    triggered_events.push(event);
                }
                Ok(None) => {
                    // 规则未触发，检查是否需要解决现有事件
                    self.check_resolution(rule, &context).await?;
                }
                Err(e) => {
                    error!("Failed to evaluate rule {}: {}", rule.name, e);
                    counter!("alert_evaluation_errors_total", "rule_id" => rule.id.to_string()).increment(1);
                }
            }
            
            // 更新最后评估时间
            self.update_last_evaluation(rule.id).await;
        }
        
        if !triggered_events.is_empty() {
            info!("Generated {} alert events", triggered_events.len());
            counter!("alert_events_generated_total").increment(triggered_events.len() as u64);
        }
        
        Ok(triggered_events)
    }
    
    /// 更新数值历史缓存
    async fn update_value_history(&self, frame: &TelemetryFrame) {
        let key = format!("{}:{}", frame.device_id, frame.tag_id);
        let mut history = self.value_history.write().await;
        
        let buffer = history.entry(key).or_insert_with(|| HistoryBuffer::new(100));
        buffer.push(frame.timestamp, frame.value);
    }
    
    /// 检查是否应该评估规则
    async fn should_evaluate_rule(&self, rule: &AlertRule) -> bool {
        let last_eval = self.last_evaluation.read().await;
        
        if let Some(last_time) = last_eval.get(&rule.id) {
            let elapsed = Utc::now() - *last_time;
            elapsed.num_seconds() >= rule.eval_every as i64
        } else {
            true // 首次评估
        }
    }
    
    /// 构建评估上下文
    async fn build_evaluation_context(
        &self, 
        frame: &TelemetryFrame, 
        rule: &AlertRule
    ) -> AlertResult<EvaluationContext> {
        let key = format!("{}:{}", frame.device_id, frame.tag_id);
        let history = self.value_history.read().await;
        
        let historical_values = if let Some(buffer) = history.get(&key) {
            buffer.get_recent_values(Duration::minutes(10)) // 获取最近10分钟的历史数据
        } else {
            vec![]
        };
        
        // 查询设备和点位名称（这里简化处理，实际应该缓存）
        let (device_name, tag_name, unit) = self.get_metadata(frame.device_id, frame.tag_id).await?;
        
        Ok(EvaluationContext {
            timestamp: frame.timestamp,
            device_id: frame.device_id,
            tag_id: frame.tag_id,
            current_value: frame.value,
            historical_values,
            device_name,
            tag_name,
            unit,
        })
    }
    
    /// 获取设备和点位元数据
    async fn get_metadata(
        &self, 
        device_id: Uuid, 
        tag_id: Uuid
    ) -> AlertResult<(Option<String>, Option<String>, Option<String>)> {
        // 查询设备名称
        let device_name = sqlx::query_scalar!(
            "SELECT name FROM devices WHERE id = $1",
            device_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to query device name: {}", e)))?;
        
        // 查询点位名称和单位
        let tag_info = sqlx::query!(
            "SELECT name, unit FROM tags WHERE id = $1",
            tag_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to query tag info: {}", e)))?;
        
        let (tag_name, unit) = if let Some(info) = tag_info {
            (Some(info.name), info.unit)
        } else {
            (None, None)
        };
        
        Ok((device_name, tag_name, unit))
    }
    
    /// 评估单个规则
    async fn evaluate_rule(
        &self, 
        rule: &AlertRule, 
        context: &EvaluationContext
    ) -> AlertResult<Option<AlertEvent>> {
        let start_time = std::time::Instant::now();
        
        // 检查规则是否在静默期
        if rule.is_silenced() {
            debug!("Rule {} is in silence period", rule.name);
            return Ok(None);
        }
        
        // 执行基本评估
        let condition_met = rule.evaluate(context);
        
        if !condition_met {
            counter!("alert_evaluations_total", "result" => "not_triggered", "rule_id" => rule.id.to_string()).increment(1);
            return Ok(None);
        }
        
        // 检查持续时间条件
        if let Some(eval_for) = rule.eval_for {
            if !self.check_duration_condition(rule, context, eval_for).await? {
                debug!("Rule {} condition met but duration not satisfied", rule.name);
                return Ok(None);
            }
        }
        
        // 检查是否已经有正在触发的事件
        let firing_events = self.firing_events.read().await;
        if firing_events.contains_key(&rule.id) {
            debug!("Rule {} already has a firing event", rule.name);
            return Ok(None);
        }
        drop(firing_events);
        
        // 创建新的报警事件
        let mut event = AlertEvent::new(rule, context);
        
        // 保存到数据库
        sqlx::query!(
            r#"
            INSERT INTO alert_events (
                id, rule_id, rule_name, device_id, tag_id,
                fired_at, value, threshold, level, status, message, context
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            event.id,
            event.rule_id,
            event.rule_name,
            event.device_id,
            event.tag_id,
            event.fired_at,
            event.value,
            event.threshold,
            event.level as AlertLevel,
            event.status as AlertEventStatus,
            event.message,
            event.context
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to save alert event: {}", e)))?;
        
        // 更新规则触发统计
        sqlx::query!(
            "UPDATE alert_rules SET fire_count = fire_count + 1, last_fired_at = $1 WHERE id = $2",
            Utc::now(),
            rule.id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to update rule stats: {}", e)))?;
        
        // 添加到正在触发的事件缓存
        let mut firing_events = self.firing_events.write().await;
        firing_events.insert(rule.id, event.clone());
        
        let eval_duration = start_time.elapsed();
        histogram!("alert_evaluation_duration_seconds").record(eval_duration.as_secs_f64());
        counter!("alert_evaluations_total", "result" => "triggered", "rule_id" => rule.id.to_string()).increment(1);
        
        info!(
            "Alert triggered: {} - {} (value: {}, threshold: {})",
            rule.name, event.message, context.current_value, rule.threshold
        );
        
        Ok(Some(event))
    }
    
    /// 检查持续时间条件
    async fn check_duration_condition(
        &self,
        rule: &AlertRule,
        context: &EvaluationContext,
        eval_for: u64,
    ) -> AlertResult<bool> {
        let key = format!("{}:{}", context.device_id, context.tag_id);
        let history = self.value_history.read().await;
        
        if let Some(buffer) = history.get(&key) {
            let duration = Duration::seconds(eval_for as i64);
            let recent_values = buffer.get_recent_values(duration);
            
            // 检查最近这段时间内是否一直满足条件
            let all_satisfy = recent_values.iter().all(|&value| {
                rule.operator.evaluate(value, rule.threshold)
            });
            
            // 需要有足够的数据点且都满足条件
            Ok(recent_values.len() >= 2 && all_satisfy)
        } else {
            Ok(false)
        }
    }
    
    /// 检查是否需要解决现有报警
    async fn check_resolution(&self, rule: &AlertRule, context: &EvaluationContext) -> AlertResult<()> {
        let mut firing_events = self.firing_events.write().await;
        
        if let Some(mut event) = firing_events.remove(&rule.id) {
            // 条件不再满足，解决报警
            event.resolve();
            
            // 更新数据库
            sqlx::query!(
                "UPDATE alert_events SET status = $1, resolved_at = $2 WHERE id = $3",
                event.status as AlertEventStatus,
                event.resolved_at,
                event.id
            )
            .execute(&self.db_pool)
            .await
            .map_err(|e| AlertError::database_error(format!("Failed to update resolved event: {}", e)))?;
            
            info!("Alert resolved: {} - {}", rule.name, event.message);
            counter!("alert_resolutions_total", "rule_id" => rule.id.to_string()).increment(1);
        }
        
        Ok(())
    }
    
    /// 更新最后评估时间
    async fn update_last_evaluation(&self, rule_id: Uuid) {
        let mut last_eval = self.last_evaluation.write().await;
        last_eval.insert(rule_id, Utc::now());
    }
    
    /// 获取正在触发的事件列表
    pub async fn get_firing_events(&self) -> Vec<AlertEvent> {
        let firing_events = self.firing_events.read().await;
        firing_events.values().cloned().collect()
    }
    
    /// 获取规则评估统计信息
    pub async fn get_evaluation_stats(&self) -> AlertResult<serde_json::Value> {
        let active_rules = self.active_rules.read().await;
        let firing_events = self.firing_events.read().await;
        
        Ok(serde_json::json!({
            "active_rules_count": active_rules.len(),
            "firing_events_count": firing_events.len(),
            "rules_by_level": {
                "INFO": active_rules.values().filter(|r| r.level == AlertLevel::INFO).count(),
                "WARN": active_rules.values().filter(|r| r.level == AlertLevel::WARN).count(),
                "CRIT": active_rules.values().filter(|r| r.level == AlertLevel::CRIT).count(),
            },
            "last_reload": Utc::now()
        }))
    }
    
    /// 强制触发规则重新加载
    pub async fn force_reload(&self) -> AlertResult<()> {
        self.reload_rules().await
    }
    
    /// 手动解决报警事件
    pub async fn resolve_event(&self, event_id: Uuid) -> AlertResult<()> {
        // 更新数据库
        sqlx::query!(
            "UPDATE alert_events SET status = 'resolved', resolved_at = $1 WHERE id = $2",
            Utc::now(),
            event_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to resolve event: {}", e)))?;
        
        // 从正在触发的事件中移除
        let mut firing_events = self.firing_events.write().await;
        firing_events.retain(|_, event| event.id != event_id);
        
        info!("Manually resolved alert event: {}", event_id);
        
        Ok(())
    }
    
    /// 确认报警事件
    pub async fn acknowledge_event(&self, event_id: Uuid) -> AlertResult<()> {
        sqlx::query!(
            "UPDATE alert_events SET status = 'acknowledged' WHERE id = $1",
            event_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to acknowledge event: {}", e)))?;
        
        info!("Acknowledged alert event: {}", event_id);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    
    #[test]
    fn test_history_buffer() {
        let mut buffer = HistoryBuffer::new(3);
        let now = Utc::now();
        
        buffer.push(now, 10.0);
        buffer.push(now + Duration::seconds(1), 20.0);
        buffer.push(now + Duration::seconds(2), 30.0);
        buffer.push(now + Duration::seconds(3), 40.0);
        
        // 应该只保留最近3个值
        assert_eq!(buffer.values.len(), 3);
        assert_eq!(buffer.latest_value(), Some(40.0));
        
        let recent = buffer.get_recent_values(Duration::seconds(2));
        assert_eq!(recent.len(), 2); // 最近2秒的数据
    }
    
    #[test]
    fn test_compare_operator() {
        use crate::models::CompareOperator;
        
        assert!(CompareOperator::GT.evaluate(10.0, 5.0));
        assert!(!CompareOperator::GT.evaluate(5.0, 10.0));
        
        assert!(CompareOperator::LT.evaluate(5.0, 10.0));
        assert!(!CompareOperator::LT.evaluate(10.0, 5.0));
        
        assert!(CompareOperator::EQ.evaluate(10.0, 10.0));
        assert!(!CompareOperator::EQ.evaluate(10.0, 5.0));
    }
}