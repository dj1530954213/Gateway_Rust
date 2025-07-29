//! service.rs —— 报警引擎服务主服务类
//!
//! 统一管理：
//! - 配置初始化
//! - 数据库连接
//! - Frame Bus 订阅
//! - HTTP 服务器
//! - 报警引擎
//! - 生命周期管理
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::{AlertEngine, AlertEngineConfig, AlertError, AlertResult};
use metrics_exporter_prometheus::PrometheusBuilder;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{interval, Duration};
use tracing::{info, error, warn, debug};

/// 报警引擎服务
pub struct AlertEngineService {
    config: AlertEngineConfig,
    db_pool: PgPool,
    alert_engine: AlertEngine,
    shutdown_tx: broadcast::Sender<()>,
    http_server_handle: Option<tokio::task::JoinHandle<()>>,
    /// 遥测数据发送通道
    telemetry_tx: mpsc::Sender<crate::models::TelemetryFrame>,
    /// 报警事件接收通道
    event_rx: Option<mpsc::Receiver<crate::models::AlertEvent>>,
}


impl AlertEngineService {
    /// 创建新的报警引擎服务
    pub async fn new(config: AlertEngineConfig) -> AlertResult<Self> {
        info!("Initializing Alert Engine Service...");
        
        // 初始化数据库连接池
        let db_pool = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .connect_timeout(Duration::from_secs(config.database.connect_timeout))
            .connect(&config.database.pg_dsn)
            .await
            .map_err(|e| AlertError::config_error(format!("Failed to connect to database: {}", e)))?;
        
        info!("Database connection established");
        
        // 运行数据库迁移
        sqlx::migrate!("../../schema/migrations")
            .run(&db_pool)
            .await
            .map_err(|e| AlertError::config_error(format!("Failed to run migrations: {}", e)))?;
        
        info!("Database migrations completed");
        
        // 初始化Prometheus指标导出器
        if config.monitoring.enable_metrics {
            let prometheus_handle = PrometheusBuilder::new()
                .with_http_listener(([0, 0, 0, 0], config.monitoring.metrics_port))
                .install()
                .map_err(|e| AlertError::config_error(format!("Failed to install Prometheus recorder: {}", e)))?;
            
            info!("Prometheus metrics enabled on port {}", config.monitoring.metrics_port);
        }
        
        // 创建关闭信号通道
        let (shutdown_tx, _) = broadcast::channel(1);
        
        // 初始化报警引擎
        let (alert_engine, telemetry_tx, event_rx) = AlertEngine::new(
            config.clone(),
            db_pool.clone(),
            shutdown_tx.subscribe(),
        ).await?;
        
        info!("Alert Engine initialized");
        
        Ok(Self {
            config,
            db_pool,
            alert_engine,
            shutdown_tx,
            http_server_handle: None,
            telemetry_tx,
            event_rx: Some(event_rx),
        })
    }
    
    /// 启动服务
    pub async fn start(&mut self) -> AlertResult<()> {
        info!("Starting Alert Engine Service components...");
        
        // 启动HTTP服务器
        self.start_http_server().await?;
        
        // 启动报警引擎
        self.alert_engine.start().await?;
        
        // 启动健康检查任务
        self.start_health_check_task();
        
        // 启动事件处理任务
        self.start_event_processing_task().await;
        
        // 启动WebSocket通知推送
        self.start_websocket_notification().await;
        
        // 启动Frame Bus订阅任务（模拟）
        self.start_frame_bus_subscription().await;
        
        info!("All Alert Engine Service components started");
        
        // 保持服务运行
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        shutdown_rx.recv().await.ok();
        
        Ok(())
    }
    
    /// 关闭服务
    pub async fn shutdown(&self) {
        info!("Shutting down Alert Engine Service...");
        
        // 发送关闭信号
        let _ = self.shutdown_tx.send(());
        
        // 等待HTTP服务器关闭
        if let Some(handle) = &self.http_server_handle {
            handle.abort();
        }
        
        // 关闭数据库连接池
        self.db_pool.close().await;
        
        info!("Alert Engine Service shutdown complete");
    }
    
    /// 启动HTTP服务器
    async fn start_http_server(&self) -> AlertResult<()> {
        let app = crate::routes::create_routes(crate::routes::AppState {
            db_pool: self.db_pool.clone(),
            alert_engine: Arc::new(self.alert_engine.clone()),
        });
        
        let listener = tokio::net::TcpListener::bind(self.config.server.http_addr)
            .await
            .map_err(|e| AlertError::config_error(format!("Failed to bind HTTP server: {}", e)))?;
        
        info!("HTTP server listening on {}", self.config.server.http_addr);
        
        let server_task = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                error!("HTTP server error: {}", e);
            }
        });
        
        // 存储服务器句柄（这里简化处理，实际应该存储到结构体中）
        
        Ok(())
    }
    
    /// 启动健康检查任务
    fn start_health_check_task(&self) {
        let check_interval = self.config.monitoring.health_check_interval;
        let db_pool = self.db_pool.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        
        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(check_interval));
            
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        debug!("Performing health check...");
                        
                        // 检查数据库连接
                        match sqlx::query("SELECT 1").execute(&db_pool).await {
                            Ok(_) => debug!("Database health check passed"),
                            Err(e) => warn!("Database health check failed: {}", e),
                        }
                        
                        // 这里可以添加更多健康检查
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Health check task shutting down");
                        break;
                    }
                }
            }
        });
    }
    
    /// 启动事件处理任务
    async fn start_event_processing_task(&mut self) {
        let mut event_rx = self.event_rx.take()
            .expect("Event receiver should be available");
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        
        tokio::spawn(async move {
            info!("Alert event processing task started");
            
            loop {
                tokio::select! {
                    Some(event) = event_rx.recv() => {
                        info!("Processing alert event: {} - {}", event.rule_name, event.message);
                        
                        // 这里将来会发送到通知器
                        // TODO: 实现通知发送逻辑
                        
                        debug!("Alert event processed: {}", event.id);
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Event processing task shutting down");
                        break;
                    }
                }
            }
        });
    }
    
    /// 启动Frame Bus订阅任务（模拟）
    async fn start_frame_bus_subscription(&self) {
        let telemetry_tx = self.telemetry_tx.clone();
        let frame_bus_url = self.config.frame_bus.bus_url.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        
        tokio::spawn(async move {
            info!("Frame Bus subscription task started for {}", frame_bus_url);
            
            // 模拟从Frame Bus接收数据
            let mut interval_timer = interval(Duration::from_secs(5));
            
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        // 模拟遥测数据
                        let frame = crate::models::TelemetryFrame {
                            device_id: uuid::Uuid::new_v4(),
                            tag_id: uuid::Uuid::new_v4(),
                            timestamp: chrono::Utc::now(),
                            value: 25.5,
                            unit: Some("°C".to_string()),
                            quality: Some("GOOD".to_string()),
                        };
                        
                        if let Err(e) = telemetry_tx.send(frame).await {
                            warn!("Failed to send telemetry frame: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Frame Bus subscription task shutting down");
                        break;
                    }
                }
            }
        });
    }
    
    /// 启动WebSocket通知推送
    async fn start_websocket_notification(&self) {
        let db_pool = self.db_pool.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        
        // 创建WebSocket通知器连接管理器
        let websocket_notifier = crate::notifiers::websocket::WebSocketNotifier::new();
        let connection_manager = websocket_notifier.connection_manager();
        
        tokio::spawn(async move {
            info!("WebSocket notification task started");
            let mut last_check = chrono::Utc::now();
            let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(5));
            
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        // 查询最近5秒内的新触发事件
                        let current_time = chrono::Utc::now();
                        
                        match sqlx::query_as!(
                            crate::models::AlertEvent,
                            r#"
                            SELECT 
                                id, rule_id, rule_name, device_id, tag_id,
                                fired_at, resolved_at, value, threshold,
                                level as "level: crate::models::AlertLevel", 
                                status as "status: crate::models::AlertEventStatus",
                                message, context, notification_status
                            FROM alert_events 
                            WHERE fired_at > $1 AND status = 'firing'
                            ORDER BY fired_at DESC
                            "#,
                            last_check
                        ).fetch_all(&db_pool).await {
                            Ok(events) => {
                                for event in events {
                                    info!("Broadcasting alert event via WebSocket: {} - {}", event.rule_name, event.message);
                                    
                                    // 创建WebSocket消息
                                    let message = crate::notifiers::websocket::WebSocketMessage {
                                        message_type: "alert".to_string(),
                                        id: uuid::Uuid::new_v4().to_string(),
                                        timestamp: chrono::Utc::now(),
                                        data: serde_json::json!({
                                            "event_id": event.id,
                                            "rule_id": event.rule_id,
                                            "rule_name": event.rule_name,
                                            "level": event.level,
                                            "status": event.status,
                                            "message": event.message,
                                            "fired_at": event.fired_at,
                                            "device_id": event.device_id,
                                            "tag_id": event.tag_id,
                                            "value": event.value,
                                            "threshold": event.threshold,
                                            "context": event.context
                                        }),
                                        room: Some("alerts".to_string()),
                                    };
                                    
                                    // 广播消息
                                    connection_manager.broadcast_message(message).await;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to query recent alert events: {}", e);
                            }
                        }
                        
                        last_check = current_time;
                    }
                    _ = shutdown_rx.recv() => {
                        info!("WebSocket notification task shutting down");
                        break;
                    }
                }
            }
        });
    }
}

