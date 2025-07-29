//! alert-engine —— 独立报警引擎服务
//!
//! 核心功能：
//! - 实时数据流监控
//! - 规则评估引擎
//! - 多通道通知发送（SMTP、Webhook、WebSocket）
//! - 报警事件历史记录
//! - 自适应阈值调整
//!
//! 架构设计：
//! - 独立进程运行，通过frame-bus订阅数据
//! - 基于规则的事件驱动架构
//! - 多种通知器插件化设计
//! - 高可用性与故障恢复
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

pub mod config;
pub mod engine;
pub mod evaluator;
pub mod notifiers;
pub mod models;
pub mod service;
pub mod error;
pub mod routes;

pub use config::AlertEngineConfig;
pub use engine::AlertEngine;
pub use evaluator::RuleEvaluator;
pub use service::AlertEngineService;
pub use error::{AlertError, AlertResult};
pub use notifiers::{Notifier, NotificationManager, NotifierFactory};