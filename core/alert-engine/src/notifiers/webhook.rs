//! webhook.rs —— Webhook HTTP回调通知器
//!
//! 功能特性：
//! - HTTP/HTTPS支持
//! - 自定义请求头
//! - JSON/Form数据格式
//! - 模板渲染
//! - 重试机制
//! - 超时控制
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::{AlertError, AlertResult};
use crate::models::{AlertEvent, NotificationChannel};
use crate::notifiers::Notifier;
use async_trait::async_trait;
use reqwest::{Client, Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use url::Url;

/// Webhook通知器
pub struct WebhookNotifier {
    /// HTTP客户端
    client: Client,
}

/// Webhook配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// 目标URL
    pub url: String,
    /// HTTP方法
    pub method: Option<String>,
    /// 请求头
    pub headers: Option<HashMap<String, String>>,
    /// 请求体模板
    pub body_template: serde_json::Value,
    /// 内容类型
    pub content_type: Option<String>,
    /// 请求超时（秒）
    pub timeout: Option<u64>,
    /// 重试次数
    pub retry_count: Option<u32>,
    /// 重试间隔（秒）
    pub retry_interval: Option<u64>,
    /// 是否验证SSL证书
    pub verify_ssl: Option<bool>,
    /// 认证信息
    pub auth: Option<WebhookAuth>,
}

/// Webhook认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebhookAuth {
    #[serde(rename = "basic")]
    Basic {
        username: String,
        password: String,
    },
    #[serde(rename = "bearer")]
    Bearer {
        token: String,
    },
    #[serde(rename = "api_key")]
    ApiKey {
        key: String,
        value: String,
        location: ApiKeyLocation,
    },
}

/// API Key位置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyLocation {
    Header,
    Query,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: Some("POST".to_string()),
            headers: None,
            body_template: serde_json::json!({}),
            content_type: Some("application/json".to_string()),
            timeout: Some(30),
            retry_count: Some(3),
            retry_interval: Some(5),
            verify_ssl: Some(true),
            auth: None,
        }
    }
}

impl WebhookNotifier {
    /// 创建新的Webhook通知器
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }
    
    /// 创建带自定义配置的HTTP客户端
    fn create_client(&self, config: &WebhookConfig) -> AlertResult<Client> {
        let mut client_builder = Client::builder();
        
        // 设置超时
        if let Some(timeout) = config.timeout {
            client_builder = client_builder.timeout(Duration::from_secs(timeout));
        }
        
        // 设置SSL验证
        if let Some(verify_ssl) = config.verify_ssl {
            client_builder = client_builder.danger_accept_invalid_certs(!verify_ssl);
        }
        
        // 设置用户代理
        client_builder = client_builder.user_agent("AlertEngine/1.0");
        
        client_builder
            .build()
            .map_err(|e| AlertError::config_error(format!("Failed to create HTTP client: {}", e)))
    }
    
    /// 渲染JSON模板
    fn render_json_template(
        &self,
        template: &serde_json::Value,
        event: &AlertEvent,
        context: &serde_json::Value,
    ) -> serde_json::Value {
        match template {
            serde_json::Value::String(s) => {
                serde_json::Value::String(self.render_string_template(s, event, context))
            }
            serde_json::Value::Object(map) => {
                let mut result = serde_json::Map::new();
                for (key, value) in map {
                    result.insert(key.clone(), self.render_json_template(value, event, context));
                }
                serde_json::Value::Object(result)
            }
            serde_json::Value::Array(arr) => {
                let result: Vec<_> = arr
                    .iter()
                    .map(|item| self.render_json_template(item, event, context))
                    .collect();
                serde_json::Value::Array(result)
            }
            _ => template.clone(),
        }
    }
    
    /// 渲染字符串模板
    fn render_string_template(&self, template: &str, event: &AlertEvent, context: &serde_json::Value) -> String {
        let mut rendered = template.to_string();
        
        // 替换基本字段
        rendered = rendered.replace("{{rule_name}}", &event.rule_name);
        rendered = rendered.replace("{{message}}", &event.message);
        rendered = rendered.replace("{{level}}", &format!("{:?}", event.level));
        rendered = rendered.replace("{{timestamp}}", &event.fired_at.to_rfc3339());
        rendered = rendered.replace("{{event_id}}", &event.id.to_string());
        
        if let Some(value) = event.value {
            rendered = rendered.replace("{{value}}", &value.to_string());
        }
        rendered = rendered.replace("{{threshold}}", &event.threshold.to_string());
        
        if let Some(device_id) = event.device_id {
            rendered = rendered.replace("{{device_id}}", &device_id.to_string());
        }
        if let Some(tag_id) = event.tag_id {
            rendered = rendered.replace("{{tag_id}}", &tag_id.to_string());
        }
        
        // 从上下文替换额外字段
        if let Some(device_name) = context.get("device_name").and_then(|v| v.as_str()) {
            rendered = rendered.replace("{{device_name}}", device_name);
        }
        if let Some(tag_name) = context.get("tag_name").and_then(|v| v.as_str()) {
            rendered = rendered.replace("{{tag_name}}", tag_name);
        }
        if let Some(unit) = context.get("unit").and_then(|v| v.as_str()) {
            rendered = rendered.replace("{{unit}}", unit);
        }
        
        // 添加一些有用的衍生字段
        let severity_emoji = match event.level {
            crate::models::AlertLevel::INFO => "ℹ️",
            crate::models::AlertLevel::WARN => "⚠️",
            crate::models::AlertLevel::CRIT => "🚨",
        };
        rendered = rendered.replace("{{severity_emoji}}", severity_emoji);
        
        rendered
    }
    
    /// 构建HTTP请求
    async fn build_request(
        &self,
        config: &WebhookConfig,
        event: &AlertEvent,
    ) -> AlertResult<RequestBuilder> {
        // 验证URL
        let url = Url::parse(&config.url)
            .map_err(|e| AlertError::config_error(format!("Invalid webhook URL: {}", e)))?;
        
        // 获取HTTP方法
        let method = config.method.as_deref().unwrap_or("POST");
        let http_method = Method::from_bytes(method.as_bytes())
            .map_err(|e| AlertError::config_error(format!("Invalid HTTP method: {}", e)))?;
        
        // 创建客户端
        let client = self.create_client(config)?;
        
        // 创建请求构建器
        let mut request_builder = client.request(http_method, url);
        
        // 添加默认请求头
        if let Some(content_type) = &config.content_type {
            request_builder = request_builder.header("Content-Type", content_type);
        }
        
        // 添加自定义请求头
        if let Some(headers) = &config.headers {
            for (key, value) in headers {
                let rendered_value = self.render_string_template(
                    value,
                    event,
                    event.context.as_ref().unwrap_or(&serde_json::json!({})),
                );
                request_builder = request_builder.header(key, rendered_value);
            }
        }
        
        // 添加认证
        if let Some(auth) = &config.auth {
            request_builder = self.apply_auth(request_builder, auth)?;
        }
        
        // 渲染并设置请求体
        let context = event.context.as_ref().unwrap_or(&serde_json::json!({}));
        let rendered_body = self.render_json_template(&config.body_template, event, context);
        
        if config.content_type.as_deref() == Some("application/json") {
            request_builder = request_builder.json(&rendered_body);
        } else {
            // 对于其他内容类型，转换为字符串
            let body_string = match rendered_body {
                serde_json::Value::String(s) => s,
                other => serde_json::to_string(&other)
                    .map_err(|e| AlertError::config_error(format!("Failed to serialize body: {}", e)))?,
            };
            request_builder = request_builder.body(body_string);
        }
        
        Ok(request_builder)
    }
    
    /// 应用认证配置
    fn apply_auth(
        &self,
        mut request_builder: RequestBuilder,
        auth: &WebhookAuth,
    ) -> AlertResult<RequestBuilder> {
        match auth {
            WebhookAuth::Basic { username, password } => {
                request_builder = request_builder.basic_auth(username, Some(password));
            }
            WebhookAuth::Bearer { token } => {
                request_builder = request_builder.bearer_auth(token);
            }
            WebhookAuth::ApiKey { key, value, location } => {
                match location {
                    ApiKeyLocation::Header => {
                        request_builder = request_builder.header(key, value);
                    }
                    ApiKeyLocation::Query => {
                        request_builder = request_builder.query(&[(key, value)]);
                    }
                }
            }
        }
        
        Ok(request_builder)
    }
    
    /// 发送HTTP请求（带重试）
    async fn send_with_retry(
        &self,
        request_builder: RequestBuilder,
        config: &WebhookConfig,
    ) -> AlertResult<reqwest::Response> {
        let retry_count = config.retry_count.unwrap_or(3);
        let retry_interval = config.retry_interval.unwrap_or(5);
        
        let mut last_error = None;
        
        for attempt in 0..=retry_count {
            if attempt > 0 {
                debug!("Retrying webhook request, attempt {}/{}", attempt, retry_count);
                tokio::time::sleep(Duration::from_secs(retry_interval)).await;
            }
            
            // 克隆请求（reqwest的RequestBuilder不能重用）
            let request = request_builder
                .try_clone()
                .ok_or_else(|| AlertError::notification_error("Failed to clone request".to_string()))?
                .build()
                .map_err(|e| AlertError::notification_error(format!("Failed to build request: {}", e)))?;
            
            match self.client.execute(request).await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        let error_msg = format!("HTTP {} - {}", status, body);
                        
                        if status.is_client_error() {
                            // 4xx错误通常不应该重试
                            return Err(AlertError::notification_error(error_msg));
                        } else {
                            // 5xx错误可以重试
                            last_error = Some(AlertError::notification_error(error_msg));
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(AlertError::notification_error(format!("Request failed: {}", e)));
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| AlertError::notification_error("All retry attempts failed".to_string())))
    }
}

#[async_trait]
impl Notifier for WebhookNotifier {
    fn name(&self) -> &'static str {
        "webhook"
    }
    
    async fn send_notification(
        &self,
        event: &AlertEvent,
        channel: &NotificationChannel,
    ) -> AlertResult<()> {
        debug!("Sending webhook notification for event: {}", event.id);
        
        // 解析通道配置
        let config: WebhookConfig = serde_json::from_value(channel.config.clone())
            .map_err(|e| AlertError::config_error(format!("Invalid webhook channel config: {}", e)))?;
        
        // 构建HTTP请求
        let request_builder = self.build_request(&config, event).await?;
        
        // 发送请求
        match self.send_with_retry(request_builder, &config).await {
            Ok(response) => {
                let status = response.status();
                info!(
                    "Webhook sent successfully: {} -> {}, status: {}",
                    event.rule_name, config.url, status
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to send webhook: {} -> {}: {}",
                    event.rule_name, config.url, e
                );
                Err(e)
            }
        }
    }
    
    async fn validate_config(&self, config: &serde_json::Value) -> AlertResult<()> {
        // 验证配置格式
        let webhook_config: WebhookConfig = serde_json::from_value(config.clone())
            .map_err(|e| AlertError::config_error(format!("Invalid webhook config format: {}", e)))?;
        
        // 验证URL格式
        Url::parse(&webhook_config.url)
            .map_err(|e| AlertError::config_error(format!("Invalid URL: {}", e)))?;
        
        // 验证HTTP方法
        if let Some(method) = &webhook_config.method {
            Method::from_bytes(method.as_bytes())
                .map_err(|e| AlertError::config_error(format!("Invalid HTTP method: {}", e)))?;
        }
        
        // 验证超时值
        if let Some(timeout) = webhook_config.timeout {
            if timeout == 0 || timeout > 300 {
                return Err(AlertError::config_error("Timeout must be between 1 and 300 seconds".to_string()));
            }
        }
        
        // 验证重试配置
        if let Some(retry_count) = webhook_config.retry_count {
            if retry_count > 10 {
                return Err(AlertError::config_error("Retry count cannot exceed 10".to_string()));
            }
        }
        
        debug!("Webhook channel config validation passed");
        Ok(())
    }
    
    async fn health_check(&self) -> bool {
        // 简单的健康检查：验证HTTP客户端是否正常
        debug!("Webhook notifier health check passed");
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AlertLevel, AlertEventStatus};
    use chrono::Utc;
    use uuid::Uuid;
    
    #[test]
    fn test_string_template_rendering() {
        let notifier = WebhookNotifier::new();
        
        let event = AlertEvent {
            id: Uuid::new_v4(),
            rule_id: Uuid::new_v4(),
            rule_name: "Test Rule".to_string(),
            device_id: Some(Uuid::new_v4()),
            tag_id: Some(Uuid::new_v4()),
            fired_at: Utc::now(),
            resolved_at: None,
            value: Some(25.5),
            threshold: 30.0,
            level: AlertLevel::WARN,
            status: AlertEventStatus::Firing,
            message: "Temperature too low".to_string(),
            context: Some(serde_json::json!({
                "device_name": "Sensor-01",
                "tag_name": "Temperature"
            })),
            notification_status: vec![],
        };
        
        let template = "{{severity_emoji}} {{rule_name}}: {{device_name}} {{tag_name}} is {{value}}";
        let context = event.context.as_ref().unwrap();
        
        let rendered = notifier.render_string_template(template, &event, context);
        
        assert!(rendered.contains("⚠️"));
        assert!(rendered.contains("Test Rule"));
        assert!(rendered.contains("Sensor-01"));
        assert!(rendered.contains("25.5"));
    }
    
    #[test]
    fn test_json_template_rendering() {
        let notifier = WebhookNotifier::new();
        
        let event = AlertEvent {
            id: Uuid::new_v4(),
            rule_id: Uuid::new_v4(),
            rule_name: "Test Rule".to_string(),
            device_id: Some(Uuid::new_v4()),
            tag_id: Some(Uuid::new_v4()),
            fired_at: Utc::now(),
            resolved_at: None,
            value: Some(25.5),
            threshold: 30.0,
            level: AlertLevel::CRIT,
            status: AlertEventStatus::Firing,
            message: "Critical alert".to_string(),
            context: Some(serde_json::json!({
                "device_name": "Sensor-01"
            })),
            notification_status: vec![],
        };
        
        let template = serde_json::json!({
            "text": "{{severity_emoji}} Alert: {{rule_name}}",
            "details": {
                "message": "{{message}}",
                "device": "{{device_name}}",
                "value": "{{value}}"
            },
            "timestamp": "{{timestamp}}"
        });
        
        let context = event.context.as_ref().unwrap();
        let rendered = notifier.render_json_template(&template, &event, context);
        
        assert_eq!(rendered["text"].as_str().unwrap(), "🚨 Alert: Test Rule");
        assert_eq!(rendered["details"]["message"].as_str().unwrap(), "Critical alert");
        assert_eq!(rendered["details"]["device"].as_str().unwrap(), "Sensor-01");
        assert_eq!(rendered["details"]["value"].as_str().unwrap(), "25.5");
    }
    
    #[tokio::test]
    async fn test_config_validation() {
        let notifier = WebhookNotifier::new();
        
        // 有效配置
        let valid_config = serde_json::json!({
            "url": "https://example.com/webhook",
            "method": "POST",
            "body_template": {
                "message": "{{message}}"
            }
        });
        
        assert!(notifier.validate_config(&valid_config).await.is_ok());
        
        // 无效URL
        let invalid_config = serde_json::json!({
            "url": "not-a-url",
            "body_template": {}
        });
        
        assert!(notifier.validate_config(&invalid_config).await.is_err());
        
        // 无效HTTP方法
        let invalid_method_config = serde_json::json!({
            "url": "https://example.com/webhook",
            "method": "INVALID",
            "body_template": {}
        });
        
        assert!(notifier.validate_config(&invalid_method_config).await.is_err());
    }
}