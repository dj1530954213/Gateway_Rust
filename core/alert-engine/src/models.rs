//! models.rs —— 报警引擎数据模型
//!
//! 核心模型：
//! - AlertRule: 报警规则定义
//! - AlertEvent: 报警事件记录
//! - NotificationChannel: 通知通道配置
//! - EvaluationContext: 规则评估上下文
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 报警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// 规则ID
    pub id: Uuid,
    
    /// 规则名称
    pub name: String,
    
    /// 规则描述
    pub description: Option<String>,
    
    /// 目标设备ID（可选，为空则匹配所有设备）
    pub device_id: Option<Uuid>,
    
    /// 目标点位ID（可选，为空则匹配所有点位）
    pub tag_id: Option<Uuid>,
    
    /// 比较操作符
    pub operator: CompareOperator,
    
    /// 阈值
    pub threshold: f64,
    
    /// 报警级别
    pub level: AlertLevel,
    
    /// 评估间隔（秒）
    pub eval_every: u64,
    
    /// 持续时间（秒），条件满足多久后触发
    pub eval_for: Option<u64>,
    
    /// 是否启用
    pub enabled: bool,
    
    /// 通知通道ID列表
    pub notification_channels: Vec<Uuid>,
    
    /// 静默期（秒），同一规则触发后的静默时间
    pub silence_duration: Option<u64>,
    
    /// 创建者
    pub created_by: Option<String>,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    
    /// 最后触发时间
    pub last_fired_at: Option<DateTime<Utc>>,
    
    /// 触发次数统计
    pub fire_count: u64,
}

/// 比较操作符
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum CompareOperator {
    /// 大于
    GT,
    /// 大于等于
    GTE,
    /// 小于
    LT,
    /// 小于等于
    LTE,
    /// 等于
    EQ,
    /// 不等于
    NE,
}

/// 报警级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AlertLevel {
    /// 信息
    INFO,
    /// 警告
    WARN,
    /// 严重
    CRIT,
}

/// 报警事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    /// 事件ID
    pub id: Uuid,
    
    /// 规则ID
    pub rule_id: Uuid,
    
    /// 规则名称（冗余存储）
    pub rule_name: String,
    
    /// 设备ID
    pub device_id: Option<Uuid>,
    
    /// 点位ID
    pub tag_id: Option<Uuid>,
    
    /// 触发时间
    pub fired_at: DateTime<Utc>,
    
    /// 解决时间（可选）
    pub resolved_at: Option<DateTime<Utc>>,
    
    /// 触发时的数值
    pub value: Option<f64>,
    
    /// 阈值
    pub threshold: f64,
    
    /// 报警级别
    pub level: AlertLevel,
    
    /// 事件状态
    pub status: AlertEventStatus,
    
    /// 报警消息
    pub message: String,
    
    /// 额外上下文信息
    pub context: Option<serde_json::Value>,
    
    /// 通知发送状态
    pub notification_status: Vec<NotificationStatus>,
}

/// 报警事件状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertEventStatus {
    /// 触发中
    Firing,
    /// 已解决
    Resolved,
    /// 已确认
    Acknowledged,
    /// 已静默
    Silenced,
}

/// 通知状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStatus {
    /// 通知通道ID
    pub channel_id: Uuid,
    
    /// 通知通道名称
    pub channel_name: String,
    
    /// 发送状态
    pub status: NotificationStatusType,
    
    /// 发送时间
    pub sent_at: Option<DateTime<Utc>>,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 重试次数
    pub retry_count: u32,
}

/// 通知发送状态类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationStatusType {
    /// 等待发送
    Pending,
    /// 发送成功
    Sent,
    /// 发送失败
    Failed,
    /// 重试中
    Retrying,
}

/// 通知通道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// 通道ID
    pub id: Uuid,
    
    /// 通道名称
    pub name: String,
    
    /// 通道类型
    pub channel_type: NotificationChannelType,
    
    /// 通道配置
    pub config: serde_json::Value,
    
    /// 是否启用
    pub enabled: bool,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 通知通道类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationChannelType {
    /// 邮件
    Email,
    /// Webhook
    Webhook,
    /// WebSocket
    WebSocket,
    /// 短信
    Sms,
}

/// 规则评估上下文
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// 当前时间戳
    pub timestamp: DateTime<Utc>,
    
    /// 设备ID
    pub device_id: Uuid,
    
    /// 点位ID
    pub tag_id: Uuid,
    
    /// 当前值
    pub current_value: f64,
    
    /// 历史值（用于趋势分析）
    pub historical_values: Vec<f64>,
    
    /// 设备名称
    pub device_name: Option<String>,
    
    /// 点位名称
    pub tag_name: Option<String>,
    
    /// 单位
    pub unit: Option<String>,
}

/// 数据帧（从frame-bus接收）
#[derive(Debug, Clone)]
pub struct TelemetryFrame {
    /// 设备ID
    pub device_id: Uuid,
    
    /// 点位ID
    pub tag_id: Uuid,
    
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    
    /// 数值
    pub value: f64,
    
    /// 单位
    pub unit: Option<String>,
    
    /// 质量标识
    pub quality: Option<String>,
}

/// 报警统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatistics {
    /// 活跃规则数
    pub active_rules: u64,
    
    /// 今日触发次数
    pub today_fired_count: u64,
    
    /// 本周触发次数
    pub week_fired_count: u64,
    
    /// 本月触发次数
    pub month_fired_count: u64,
    
    /// 按级别统计
    pub by_level: std::collections::HashMap<AlertLevel, u64>,
    
    /// 按设备统计
    pub by_device: std::collections::HashMap<Uuid, u64>,
    
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    
    /// 统计时间
    pub generated_at: DateTime<Utc>,
}

impl CompareOperator {
    /// 评估比较操作
    pub fn evaluate(&self, left: f64, right: f64) -> bool {
        match self {
            CompareOperator::GT => left > right,
            CompareOperator::GTE => left >= right,
            CompareOperator::LT => left < right,
            CompareOperator::LTE => left <= right,
            CompareOperator::EQ => (left - right).abs() < f64::EPSILON,
            CompareOperator::NE => (left - right).abs() > f64::EPSILON,
        }
    }
}

impl AlertLevel {
    /// 获取数值权重（用于排序）
    pub fn weight(&self) -> u8 {
        match self {
            AlertLevel::INFO => 1,
            AlertLevel::WARN => 2,
            AlertLevel::CRIT => 3,
        }
    }
    
    /// 获取颜色代码（用于前端显示）
    pub fn color(&self) -> &'static str {
        match self {
            AlertLevel::INFO => "#409EFF",  // 蓝色
            AlertLevel::WARN => "#E6A23C",  // 橙色
            AlertLevel::CRIT => "#F56C6C",  // 红色
        }
    }
}

impl AlertRule {
    /// 创建新的报警规则
    pub fn new(
        name: String,
        device_id: Option<Uuid>,
        tag_id: Option<Uuid>,
        operator: CompareOperator,
        threshold: f64,
        level: AlertLevel,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            device_id,
            tag_id,
            operator,
            threshold,
            level,
            eval_every: 60, // 默认每分钟评估一次
            eval_for: None,
            enabled: true,
            notification_channels: vec![],
            silence_duration: None,
            created_by: None,
            created_at: now,
            updated_at: now,
            last_fired_at: None,
            fire_count: 0,
        }
    }
    
    /// 检查规则是否匹配给定的数据帧
    pub fn matches(&self, frame: &TelemetryFrame) -> bool {
        // 检查设备ID匹配
        if let Some(device_id) = self.device_id {
            if device_id != frame.device_id {
                return false;
            }
        }
        
        // 检查点位ID匹配
        if let Some(tag_id) = self.tag_id {
            if tag_id != frame.tag_id {
                return false;
            }
        }
        
        true
    }
    
    /// 评估规则是否应该触发
    pub fn evaluate(&self, context: &EvaluationContext) -> bool {
        if !self.enabled {
            return false;
        }
        
        self.operator.evaluate(context.current_value, self.threshold)
    }
    
    /// 检查是否在静默期内
    pub fn is_silenced(&self) -> bool {
        if let (Some(last_fired), Some(silence_duration)) = (self.last_fired_at, self.silence_duration) {
            let silence_until = last_fired + chrono::Duration::seconds(silence_duration as i64);
            Utc::now() < silence_until
        } else {
            false
        }
    }
    
    /// 生成报警消息
    pub fn generate_message(&self, context: &EvaluationContext) -> String {
        let device_name = context.device_name.as_deref().unwrap_or("Unknown Device");
        let tag_name = context.tag_name.as_deref().unwrap_or("Unknown Tag");
        let unit = context.unit.as_deref().unwrap_or("");
        
        format!(
            "Alert: {} - {} {} {} {:.2}{}",
            self.name,
            device_name,
            tag_name,
            self.operator_symbol(),
            self.threshold,
            unit
        )
    }
    
    /// 获取操作符符号
    fn operator_symbol(&self) -> &'static str {
        match self.operator {
            CompareOperator::GT => ">",
            CompareOperator::GTE => ">=",
            CompareOperator::LT => "<",
            CompareOperator::LTE => "<=",
            CompareOperator::EQ => "==",
            CompareOperator::NE => "!=",
        }
    }
}

impl AlertEvent {
    /// 创建新的报警事件
    pub fn new(rule: &AlertRule, context: &EvaluationContext) -> Self {
        Self {
            id: Uuid::new_v4(),
            rule_id: rule.id,
            rule_name: rule.name.clone(),
            device_id: Some(context.device_id),
            tag_id: Some(context.tag_id),
            fired_at: context.timestamp,
            resolved_at: None,
            value: Some(context.current_value),
            threshold: rule.threshold,
            level: rule.level.clone(),
            status: AlertEventStatus::Firing,
            message: rule.generate_message(context),
            context: Some(serde_json::json!({
                "device_name": context.device_name,
                "tag_name": context.tag_name,
                "unit": context.unit,
                "operator": rule.operator,
            })),
            notification_status: vec![],
        }
    }
    
    /// 标记事件为已解决
    pub fn resolve(&mut self) {
        self.status = AlertEventStatus::Resolved;
        self.resolved_at = Some(Utc::now());
    }
    
    /// 添加通知状态
    pub fn add_notification_status(&mut self, status: NotificationStatus) {
        self.notification_status.push(status);
    }
    
    /// 检查是否所有通知都已发送
    pub fn all_notifications_sent(&self) -> bool {
        !self.notification_status.is_empty() &&
        self.notification_status.iter().all(|s| s.status == NotificationStatusType::Sent)
    }
}