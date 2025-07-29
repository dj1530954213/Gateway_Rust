//! engine.rs —— 报警引擎核心
//!
//! 报警引擎主要组件：
//! - 数据流监听器
//! - 规则评估调度器
//! - 事件管理器
//! - 通知分发器
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::{AlertEngineConfig, AlertError, AlertResult};
use crate::models::{AlertStatistics, TelemetryFrame, AlertEvent};
use crate::evaluator::RuleEvaluator;
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{interval, Duration};
use tracing::{info, debug, warn, error};
use metrics::{counter, gauge};

/// 报警引擎
#[derive(Clone)]
pub struct AlertEngine {
    config: AlertEngineConfig,
    db_pool: PgPool,
    shutdown_rx: broadcast::Receiver<()>,
    running: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    /// 规则评估器
    evaluator: Arc<RuleEvaluator>,
    /// 遥测数据接收通道
    telemetry_rx: Option<mpsc::Receiver<TelemetryFrame>>,
    /// 报警事件发送通道
    event_tx: mpsc::Sender<AlertEvent>,
    /// WebSocket事件发送通道
    websocket_tx: Option<mpsc::Sender<AlertEvent>>,
}

impl AlertEngine {
    /// 创建新的报警引擎
    pub async fn new(
        config: AlertEngineConfig,
        db_pool: PgPool,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> AlertResult<(Self, mpsc::Sender<TelemetryFrame>, mpsc::Receiver<AlertEvent>)> {
        info!("Initializing Alert Engine...");
        
        // 创建规则评估器
        let evaluator = Arc::new(RuleEvaluator::new(db_pool.clone()).await?);
        info!("Rule evaluator initialized");
        
        // 创建通信通道
        let (telemetry_tx, telemetry_rx) = mpsc::channel::<TelemetryFrame>(1000);
        let (event_tx, event_rx) = mpsc::channel::<AlertEvent>(1000);
        
        let engine = Self {
            config,
            db_pool,
            shutdown_rx,
            running: Arc::new(AtomicBool::new(false)),
            ready: Arc::new(AtomicBool::new(false)),
            evaluator,
            telemetry_rx: Some(telemetry_rx),
            event_tx,
            websocket_tx: None,
        };
        
        info!("Alert Engine initialized");
        
        Ok((engine, telemetry_tx, event_rx))
    }
    
    /// 启动报警引擎
    pub async fn start(&mut self) -> AlertResult<()> {
        info!("Starting Alert Engine...");
        
        self.running.store(true, Ordering::SeqCst);
        
        // 取出遥测数据接收器
        let mut telemetry_rx = self.telemetry_rx.take()
            .ok_or_else(|| AlertError::config_error("Telemetry receiver already taken".to_string()))?;
        
        let evaluator = self.evaluator.clone();
        let event_tx = self.event_tx.clone();
        let websocket_tx = self.websocket_tx.clone();
        let running = self.running.clone();
        let ready = self.ready.clone();
        let mut shutdown_rx = self.shutdown_rx.resubscribe();
        
        // 启动主处理循环
        tokio::spawn(async move {
            info!("Alert Engine main loop started");
            ready.store(true, Ordering::SeqCst);
            
            loop {
                tokio::select! {
                    // 处理遥测数据
                    Some(frame) = telemetry_rx.recv() => {
                        debug!("Received telemetry frame: device={}, tag={}", frame.device_id, frame.tag_id);
                        
                        match evaluator.process_telemetry_frame(frame).await {
                            Ok(events) => {
                                // 发送生成的报警事件
                                for event in events {
                                    // 发送到主事件通道
                                    if let Err(e) = event_tx.send(event.clone()).await {
                                        error!("Failed to send alert event: {}", e);
                                    }
                                    
                                    // 发送到WebSocket通知器（如果有）
                                    if let Some(ref ws_tx) = websocket_tx {
                                        if let Err(e) = ws_tx.send(event).await {
                                            warn!("Failed to send event to WebSocket notifier: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to process telemetry frame: {}", e);
                                counter!("alert_processing_errors_total").increment(1);
                            }
                        }
                    }
                    
                    // 接收关闭信号
                    _ = shutdown_rx.recv() => {
                        info!("Alert Engine received shutdown signal");
                        break;
                    }
                }
            }
            
            running.store(false, Ordering::SeqCst);
            ready.store(false, Ordering::SeqCst);
            info!("Alert Engine main loop stopped");
        });
        
        // 启动规则重新加载定时任务
        self.start_rule_reload_task().await;
        
        // 启动统计更新任务
        self.start_statistics_task().await;
        
        info!("Alert Engine started successfully");
        
        Ok(())
    }
    
    /// 检查引擎是否运行中
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// 检查引擎是否就绪
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }
    
    /// 启动规则重新加载定时任务
    async fn start_rule_reload_task(&self) {
        let evaluator = self.evaluator.clone();
        let reload_interval = self.config.engine.rule_reload_interval;
        let mut shutdown_rx = self.shutdown_rx.resubscribe();
        
        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(reload_interval));
            
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        debug!("Reloading alert rules...");
                        
                        if let Err(e) = evaluator.reload_rules().await {
                            error!("Failed to reload rules: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Rule reload task shutting down");
                        break;
                    }
                }
            }
        });
    }
    
    /// 启动统计信息更新任务
    async fn start_statistics_task(&self) {
        let evaluator = self.evaluator.clone();
        let db_pool = self.db_pool.clone();
        let mut shutdown_rx = self.shutdown_rx.resubscribe();
        
        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(60)); // 每分钟更新一次统计
            
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        debug!("Updating alert statistics...");
                        
                        // 更新统计信息指标
                        if let Ok(firing_events) = sqlx::query_scalar!(
                            "SELECT COUNT(*) FROM alert_events WHERE status = 'firing'"
                        ).fetch_one(&db_pool).await {
                            gauge!("alert_firing_events_total").set(firing_events.unwrap_or(0) as f64);
                        }
                        
                        if let Ok(stats) = evaluator.get_evaluation_stats().await {
                            if let Some(active_rules) = stats.get("active_rules_count") {
                                gauge!("alert_active_rules_total").set(active_rules.as_f64().unwrap_or(0.0));
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Statistics task shutting down");
                        break;
                    }
                }
            }
        });
    }
    
    /// 获取统计信息
    pub async fn get_statistics(&self) -> AlertResult<AlertStatistics> {
        debug!("Generating alert statistics");
        
        use chrono::{Duration, Utc};
        use std::collections::HashMap;
        use crate::models::AlertLevel;
        
        let now = Utc::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let week_start = now - Duration::days(7);
        let month_start = now - Duration::days(30);
        
        // 查询活跃规则数
        let active_rules = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM alert_rules WHERE enabled = true"
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to count active rules: {}", e)))?
        .unwrap_or(0) as u64;
        
        // 查询今日触发次数
        let today_fired_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM alert_events WHERE fired_at >= $1",
            today_start
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to count today's alerts: {}", e)))?
        .unwrap_or(0) as u64;
        
        // 查询本周触发次数
        let week_fired_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM alert_events WHERE fired_at >= $1",
            week_start
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to count weekly alerts: {}", e)))?
        .unwrap_or(0) as u64;
        
        // 查询本月触发次数
        let month_fired_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM alert_events WHERE fired_at >= $1",
            month_start
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to count monthly alerts: {}", e)))?
        .unwrap_or(0) as u64;
        
        // 按级别统计
        let level_stats = sqlx::query!(
            "SELECT level, COUNT(*) as count FROM alert_events WHERE fired_at >= $1 GROUP BY level",
            week_start
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to get level stats: {}", e)))?;
        
        let mut by_level = HashMap::new();
        for row in level_stats {
            let level: AlertLevel = row.level;
            by_level.insert(level, row.count.unwrap_or(0) as u64);
        }
        
        // 按设备统计
        let device_stats = sqlx::query!(
            r#"
            SELECT device_id, COUNT(*) as count 
            FROM alert_events 
            WHERE fired_at >= $1 AND device_id IS NOT NULL 
            GROUP BY device_id
            "#,
            week_start
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AlertError::database_error(format!("Failed to get device stats: {}", e)))?;
        
        let mut by_device = HashMap::new();
        for row in device_stats {
            if let Some(device_id) = row.device_id {
                by_device.insert(device_id, row.count.unwrap_or(0) as u64);
            }
        }
        
        // 计算平均响应时间（简化实现）
        let avg_response_time_ms = 50.0; // TODO: 实际计算
        
        Ok(AlertStatistics {
            active_rules,
            today_fired_count,
            week_fired_count,
            month_fired_count,
            by_level,
            by_device,
            avg_response_time_ms,
            generated_at: now,
        })
    }
    
    /// 获取当前正在触发的事件
    pub async fn get_firing_events(&self) -> Vec<AlertEvent> {
        self.evaluator.get_firing_events().await
    }
    
    /// 强制重新加载规则
    pub async fn reload_rules(&self) -> AlertResult<()> {
        self.evaluator.force_reload().await
    }
    
    /// 手动解决报警事件
    pub async fn resolve_event(&self, event_id: uuid::Uuid) -> AlertResult<()> {
        self.evaluator.resolve_event(event_id).await
    }
    
    /// 确认报警事件
    pub async fn acknowledge_event(&self, event_id: uuid::Uuid) -> AlertResult<()> {
        self.evaluator.acknowledge_event(event_id).await
    }
    
    /// 添加WebSocket通知器
    pub async fn add_websocket_notifier(&mut self, websocket_tx: mpsc::Sender<AlertEvent>) -> AlertResult<()> {
        info!("Adding WebSocket notifier to alert engine");
        self.websocket_tx = Some(websocket_tx);
        Ok(())
    }
}