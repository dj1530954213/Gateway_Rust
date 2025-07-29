//! notifiers.rs —— 报警通知器模块
//!
//! 支持的通知方式：
//! - SMTP邮件
//! - Webhook HTTP回调
//! - WebSocket实时推送
//! - SMS短信（预留）
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

pub mod email;
pub mod webhook;
pub mod websocket;

use crate::{AlertError, AlertResult};
use crate::models::{AlertEvent, NotificationChannel, NotificationStatus, NotificationStatusType};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error, debug};
use uuid::Uuid;

/// 通知器基础trait
#[async_trait]
pub trait Notifier: Send + Sync {
    /// 通知器类型
    fn name(&self) -> &'static str;
    
    /// 发送通知
    async fn send_notification(
        &self,
        event: &AlertEvent,
        channel: &NotificationChannel,
    ) -> AlertResult<()>;
    
    /// 验证通道配置
    async fn validate_config(&self, config: &serde_json::Value) -> AlertResult<()>;
    
    /// 健康检查
    async fn health_check(&self) -> bool {
        true // 默认实现
    }
}

/// 通知管理器
pub struct NotificationManager {
    /// 注册的通知器
    notifiers: std::collections::HashMap<String, Arc<dyn Notifier>>,
    /// 事件处理通道
    event_rx: Option<mpsc::Receiver<AlertEvent>>,
    /// 通知状态更新通道
    status_tx: mpsc::Sender<NotificationStatus>,
}

impl NotificationManager {
    /// 创建通知管理器
    pub fn new(
        event_rx: mpsc::Receiver<AlertEvent>,
        status_tx: mpsc::Sender<NotificationStatus>,
    ) -> Self {
        let mut manager = Self {
            notifiers: std::collections::HashMap::new(),
            event_rx: Some(event_rx),
            status_tx,
        };
        
        // 注册默认通知器
        manager.register_default_notifiers();
        
        manager
    }
    
    /// 注册默认通知器
    fn register_default_notifiers(&mut self) {
        // 注册邮件通知器
        let email_notifier = Arc::new(email::EmailNotifier::new());
        self.register_notifier("email", email_notifier);
        
        // 注册Webhook通知器
        let webhook_notifier = Arc::new(webhook::WebhookNotifier::new());
        self.register_notifier("webhook", webhook_notifier);
        
        // 注册WebSocket通知器
        let websocket_notifier = Arc::new(websocket::WebSocketNotifier::new());
        self.register_notifier("websocket", websocket_notifier);
        
        info!("Registered {} notification types", self.notifiers.len());
    }
    
    /// 注册通知器
    pub fn register_notifier(&mut self, name: &str, notifier: Arc<dyn Notifier>) {
        self.notifiers.insert(name.to_string(), notifier);
        debug!("Registered notifier: {}", name);
    }
    
    /// 获取通知器
    pub fn get_notifier(&self, name: &str) -> Option<Arc<dyn Notifier>> {
        self.notifiers.get(name).cloned()
    }
    
    /// 启动通知处理循环
    pub async fn start(&mut self) -> AlertResult<()> {
        let mut event_rx = self.event_rx.take()
            .ok_or_else(|| AlertError::config_error("Event receiver already taken".to_string()))?;
        
        let notifiers = self.notifiers.clone();
        let status_tx = self.status_tx.clone();
        
        tokio::spawn(async move {
            info!("Notification manager started");
            
            while let Some(event) = event_rx.recv().await {
                debug!("Processing notification for event: {}", event.id);
                
                // 根据事件的通知通道列表发送通知
                Self::process_event_notifications(
                    &event,
                    &notifiers,
                    &status_tx,
                ).await;
            }
            
            info!("Notification manager stopped");
        });
        
        Ok(())
    }
    
    /// 处理单个事件的所有通知
    async fn process_event_notifications(
        event: &AlertEvent,
        notifiers: &std::collections::HashMap<String, Arc<dyn Notifier>>,
        status_tx: &mpsc::Sender<NotificationStatus>,
    ) {
        // 这里应该从数据库查询通知通道配置
        // 为了演示，我们创建一些示例通道
        let sample_channels = Self::get_sample_notification_channels();
        
        for channel in sample_channels {
            if let Some(notifier) = notifiers.get(&channel.channel_type.to_string().to_lowercase()) {
                let mut status = NotificationStatus {
                    channel_id: channel.id,
                    channel_name: channel.name.clone(),
                    status: NotificationStatusType::Pending,
                    sent_at: None,
                    error_message: None,
                    retry_count: 0,
                };
                
                // 发送通知
                match notifier.send_notification(event, &channel).await {
                    Ok(()) => {
                        status.status = NotificationStatusType::Sent;
                        status.sent_at = Some(Utc::now());
                        info!("Notification sent successfully: {} -> {}", event.rule_name, channel.name);
                    }
                    Err(e) => {
                        status.status = NotificationStatusType::Failed;
                        status.error_message = Some(e.to_string());
                        error!("Failed to send notification: {} -> {}: {}", event.rule_name, channel.name, e);
                    }
                }
                
                // 发送状态更新
                if let Err(e) = status_tx.send(status).await {
                    error!("Failed to send notification status: {}", e);
                }
            } else {
                error!("Unknown notification type: {:?}", channel.channel_type);
            }
        }
    }
    
    /// 获取示例通知通道（实际应从数据库查询）
    fn get_sample_notification_channels() -> Vec<NotificationChannel> {
        use crate::models::NotificationChannelType;
        
        vec![
            NotificationChannel {
                id: Uuid::new_v4(),
                name: "Admin Email".to_string(),
                channel_type: NotificationChannelType::Email,
                config: serde_json::json!({
                    "to": "admin@example.com",
                    "subject_template": "[ALERT] {{rule_name}}",
                    "body_template": "Alert: {{message}}\nDevice: {{device_name}}\nValue: {{value}}\nTime: {{timestamp}}"
                }),
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            NotificationChannel {
                id: Uuid::new_v4(),
                name: "Webhook Alert".to_string(),
                channel_type: NotificationChannelType::Webhook,
                config: serde_json::json!({
                    "url": "https://hooks.slack.com/services/...",
                    "method": "POST",
                    "headers": {
                        "Content-Type": "application/json"
                    },
                    "body_template": {
                        "text": "🚨 Alert: {{rule_name}}\n{{message}}",
                        "channel": "#alerts"
                    }
                }),
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        ]
    }
    
    /// 获取所有通知器的健康状态
    pub async fn get_health_status(&self) -> std::collections::HashMap<String, bool> {
        let mut health_status = std::collections::HashMap::new();
        
        for (name, notifier) in &self.notifiers {
            let healthy = notifier.health_check().await;
            health_status.insert(name.clone(), healthy);
        }
        
        health_status
    }
    
    /// 验证通知通道配置
    pub async fn validate_channel_config(
        &self,
        channel_type: &str,
        config: &serde_json::Value,
    ) -> AlertResult<()> {
        if let Some(notifier) = self.get_notifier(channel_type) {
            notifier.validate_config(config).await
        } else {
            Err(AlertError::config_error(format!("Unknown notification type: {}", channel_type)))
        }
    }
}

/// 通知器工厂
pub struct NotifierFactory;

impl NotifierFactory {
    /// 创建通知器实例
    pub fn create_notifier(notifier_type: &str) -> AlertResult<Arc<dyn Notifier>> {
        match notifier_type.to_lowercase().as_str() {
            "email" => Ok(Arc::new(email::EmailNotifier::new())),
            "webhook" => Ok(Arc::new(webhook::WebhookNotifier::new())),
            "websocket" => Ok(Arc::new(websocket::WebSocketNotifier::new())),
            _ => Err(AlertError::config_error(format!("Unknown notifier type: {}", notifier_type))),
        }
    }
    
    /// 获取支持的通知器类型
    pub fn supported_types() -> Vec<&'static str> {
        vec!["email", "webhook", "websocket"]
    }
}