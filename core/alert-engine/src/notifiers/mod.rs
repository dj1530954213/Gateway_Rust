//! notifiers/mod.rs —— 通知器模块入口
//!
//! 模块组织：
//! - email: SMTP邮件通知器
//! - webhook: HTTP Webhook通知器  
//! - websocket: WebSocket实时推送通知器
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

pub mod email;
pub mod webhook;
pub mod websocket;

// 重新导出主要类型
pub use email::{EmailNotifier, EmailConfig, EmailChannelConfig};
pub use webhook::{WebhookNotifier, WebhookConfig, WebhookAuth, ApiKeyLocation};
pub use websocket::{WebSocketNotifier, WebSocketConfig, ConnectionManager, ConnectionConfig};