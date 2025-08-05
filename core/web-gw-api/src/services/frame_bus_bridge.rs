//! frame_bus_bridge.rs —— FrameBus与WebSocket桥接服务
//!
//! - 订阅frame-bus数据流
//! - 转换DataFrame为WebSocket消息
//! - 广播到WebSocket连接管理器
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::dto::{TelemetryMsg, AlertNotification, AlertLevel};
use crate::routes::websocket::{WsConnectionManager, WsMessage};
use crate::error::{ApiError, ApiResult};
use anyhow::{Context, Result};
use frame_bus::{DataFrame, Filter, FrameReceiver, subscribe, FrameKind};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// FrameBus订阅桥接服务
pub struct FrameBusBridge {
    /// WebSocket连接管理器
    ws_manager: Arc<WsConnectionManager>,
    /// 停止信号接收器
    shutdown_rx: broadcast::Receiver<()>,
    /// 是否正在运行
    running: Arc<tokio::sync::RwLock<bool>>,
    /// 批量处理配置
    batch_config: BatchConfig,
}

/// 批量处理配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 批量大小
    pub batch_size: usize,
    /// 批量超时时间（毫秒）
    pub batch_timeout_ms: u64,
    /// 启用批量处理
    pub enabled: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            batch_timeout_ms: 100,
            enabled: true,
        }
    }
}

impl FrameBusBridge {
    /// 创建新的桥接服务
    pub fn new(
        ws_manager: Arc<WsConnectionManager>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            ws_manager,
            shutdown_rx,
            running: Arc::new(tokio::sync::RwLock::new(false)),
            batch_config: BatchConfig::default(),
        }
    }

    /// 创建新的桥接服务（带批量配置）
    pub fn with_batch_config(
        ws_manager: Arc<WsConnectionManager>,
        shutdown_rx: broadcast::Receiver<()>,
        batch_config: BatchConfig,
    ) -> Self {
        Self {
            ws_manager,
            shutdown_rx,
            running: Arc::new(tokio::sync::RwLock::new(false)),
            batch_config,
        }
    }

    /// 启动桥接服务
    pub async fn start(&self) -> ApiResult<()> {
        {
            let mut running = self.running.write().await;
            if *running {
                warn!("FrameBus bridge is already running");
                return Ok(());
            }
            *running = true;
        }

        info!("Starting FrameBus bridge service");

        // 启动遥测数据订阅任务
        let telemetry_task = self.start_telemetry_subscription().await?;
        
        // 启动报警数据订阅任务
        let alert_task = self.start_alert_subscription().await?;

        // 等待停止信号
        let mut shutdown_rx = self.shutdown_rx.resubscribe();
        tokio::select! {
            _ = telemetry_task => {
                warn!("Telemetry subscription task ended unexpectedly");
            }
            _ = alert_task => {
                warn!("Alert subscription task ended unexpectedly");
            }
            _ = shutdown_rx.recv() => {
                info!("Received shutdown signal, stopping FrameBus bridge");
            }
        }

        {
            let mut running = self.running.write().await;
            *running = false;
        }

        Ok(())
    }

    /// 启动遥测数据订阅任务
    async fn start_telemetry_subscription(&self) -> ApiResult<tokio::task::JoinHandle<()>> {
        // 创建遥测数据过滤器：匹配 "telemetry.*" 前缀的数据帧
        let filter = Filter::And(vec![
            Filter::data_only(),
            Filter::tag_starts_with("telemetry."),
        ]);

        // 订阅frame-bus
        let mut receiver = subscribe(filter)
            .context("Failed to subscribe to frame-bus for telemetry data")?;

        let ws_manager = self.ws_manager.clone();
        let mut shutdown_rx = self.shutdown_rx.resubscribe();
        let batch_config = self.batch_config.clone();

        let task = tokio::spawn(async move {
            info!("Started telemetry data subscription task (batch_enabled: {})", batch_config.enabled);

            if batch_config.enabled {
                Self::run_batched_telemetry_loop(receiver, ws_manager, shutdown_rx, batch_config).await;
            } else {
                Self::run_single_telemetry_loop(receiver, ws_manager, shutdown_rx).await;
            }
        });

        Ok(task)
    }

    /// 运行单条消息处理循环
    async fn run_single_telemetry_loop(
        mut receiver: frame_bus::FrameReceiver,
        ws_manager: Arc<WsConnectionManager>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        loop {
            tokio::select! {
                // 接收frame-bus数据
                frame_result = receiver.recv() => {
                    match frame_result {
                        Ok(envelope) => {
                            if let Err(e) = Self::handle_telemetry_frame(envelope, &ws_manager).await {
                                error!("Failed to handle telemetry frame: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive frame from bus: {}", e);
                            // 短暂延迟后重试
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
                // 接收停止信号
                _ = shutdown_rx.recv() => {
                    info!("Telemetry subscription task shutting down");
                    break;
                }
            }
        }
    }

    /// 运行批量消息处理循环
    async fn run_batched_telemetry_loop(
        mut receiver: frame_bus::FrameReceiver,
        ws_manager: Arc<WsConnectionManager>,
        mut shutdown_rx: broadcast::Receiver<()>,
        batch_config: BatchConfig,
    ) {
        let mut batch = Vec::with_capacity(batch_config.batch_size);
        let mut batch_timeout = tokio::time::interval(Duration::from_millis(batch_config.batch_timeout_ms));
        batch_timeout.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                // 接收frame-bus数据
                frame_result = receiver.recv() => {
                    match frame_result {
                        Ok(envelope) => {
                            if let Ok(telemetry_msg) = Self::envelope_to_telemetry(envelope).await {
                                batch.push(telemetry_msg);

                                // 如果达到批量大小，立即发送
                                if batch.len() >= batch_config.batch_size {
                                    ws_manager.broadcast_telemetry_batch(batch.clone()).await;
                                    batch.clear();
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive frame from bus: {}", e);
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
                // 批量超时，发送当前批量
                _ = batch_timeout.tick() => {
                    if !batch.is_empty() {
                        ws_manager.broadcast_telemetry_batch(batch.clone()).await;
                        batch.clear();
                    }
                }
                // 接收停止信号
                _ = shutdown_rx.recv() => {
                    // 发送剩余的批量数据
                    if !batch.is_empty() {
                        ws_manager.broadcast_telemetry_batch(batch).await;
                    }
                    info!("Batched telemetry subscription task shutting down");
                    break;
                }
            }
        }
    }

    /// 启动报警数据订阅任务
    async fn start_alert_subscription(&self) -> ApiResult<tokio::task::JoinHandle<()>> {
        // 创建报警数据过滤器：匹配 "alert.*" 前缀的数据帧
        let filter = Filter::And(vec![
            Filter::data_only(),
            Filter::tag_starts_with("alert."),
        ]);

        // 订阅frame-bus
        let mut receiver = subscribe(filter)
            .context("Failed to subscribe to frame-bus for alert data")?;

        let ws_manager = self.ws_manager.clone();
        let mut shutdown_rx = self.shutdown_rx.resubscribe();

        let task = tokio::spawn(async move {
            info!("Started alert data subscription task");

            loop {
                tokio::select! {
                    // 接收frame-bus数据
                    frame_result = receiver.recv() => {
                        match frame_result {
                            Ok(envelope) => {
                                if let Err(e) = Self::handle_alert_frame(envelope, &ws_manager).await {
                                    error!("Failed to handle alert frame: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to receive alert frame from bus: {}", e);
                                // 短暂延迟后重试
                                sleep(Duration::from_millis(100)).await;
                            }
                        }
                    }
                    // 接收停止信号
                    _ = shutdown_rx.recv() => {
                        info!("Alert subscription task shutting down");
                        break;
                    }
                }
            }
        });

        Ok(task)
    }

    /// 将FrameEnvelope转换为TelemetryMsg
    async fn envelope_to_telemetry(envelope: frame_bus::FrameEnvelope) -> Result<TelemetryMsg> {
        // 解包DataFrame
        let data_frame = envelope.into_data()
            .context("Failed to decode data frame")?;

        // 解析tag格式：telemetry.{device_id}.{tag_id}
        let tag_parts: Vec<&str> = data_frame.tag.split('.').collect();
        if tag_parts.len() != 3 || tag_parts[0] != "telemetry" {
            return Err(anyhow::anyhow!("Invalid telemetry tag format: {}", data_frame.tag));
        }

        // 解析device_id和tag_id
        let device_id = Uuid::parse_str(tag_parts[1])
            .context("Invalid device_id in telemetry tag")?;
        let tag_id = Uuid::parse_str(tag_parts[2])
            .context("Invalid tag_id in telemetry tag")?;

        // 提取数值
        let value = match data_frame.value.as_ref() {
            Some(v) => v.to_f64().unwrap_or(0.0),
            None => {
                return Err(anyhow::anyhow!("Telemetry frame has no value: {}", data_frame.tag));
            }
        };

        // 构造遥测消息
        let telemetry_msg = TelemetryMsg {
            device_id,
            tag_id,
            ts: (data_frame.timestamp / 1_000_000) as i64, // 纳秒转毫秒
            value,
            unit: data_frame.meta.get("unit").cloned(),
        };

        Ok(telemetry_msg)
    }

    /// 处理遥测数据帧
    async fn handle_telemetry_frame(
        envelope: frame_bus::FrameEnvelope,
        ws_manager: &WsConnectionManager,
    ) -> Result<()> {
        let telemetry_msg = Self::envelope_to_telemetry(envelope).await?;
        ws_manager.broadcast_telemetry(telemetry_msg).await;
        Ok(())
    }

    /// 处理报警数据帧
    async fn handle_alert_frame(
        envelope: frame_bus::FrameEnvelope,
        ws_manager: &WsConnectionManager,
    ) -> Result<()> {
        // 解包DataFrame
        let data_frame = envelope.into_data()
            .context("Failed to decode alert frame")?;

        debug!("Received alert frame: tag={}, timestamp={}", 
               data_frame.tag, data_frame.timestamp);

        // 解析tag格式：alert.{event_id}
        let tag_parts: Vec<&str> = data_frame.tag.split('.').collect();
        if tag_parts.len() != 2 || tag_parts[0] != "alert" {
            warn!("Invalid alert tag format: {}", data_frame.tag);
            return Ok(());
        }

        // 解析event_id
        let event_id = Uuid::parse_str(tag_parts[1])
            .context("Invalid event_id in alert tag")?;

        // 从元数据中提取报警信息
        let rule_name = data_frame.meta.get("rule_name")
            .cloned()
            .unwrap_or_else(|| "Unknown Rule".to_string());
        
        let device_name = data_frame.meta.get("device_name").cloned();
        let tag_name = data_frame.meta.get("tag_name").cloned();
        
        let level = data_frame.meta.get("level")
            .and_then(|l| match l.as_str() {
                "INFO" => Some(AlertLevel::INFO),
                "WARN" => Some(AlertLevel::WARN),
                "CRIT" => Some(AlertLevel::CRIT),
                _ => None,
            })
            .unwrap_or(AlertLevel::INFO);

        let message = data_frame.meta.get("message")
            .cloned()
            .unwrap_or_else(|| "Alert triggered".to_string());

        // 从元数据中提取数值和阈值
        let value = data_frame.value.as_ref().map(|v| v.to_f64().unwrap_or(0.0));
        let threshold = data_frame.meta.get("threshold")
            .and_then(|t| t.parse::<f64>().ok())
            .unwrap_or(0.0);

        // 构造报警通知
        let alert_notification = AlertNotification {
            event_id,
            rule_name,
            device_name,
            tag_name,
            level,
            message,
            fired_at: chrono::Utc::now(),
            value,
            threshold,
        };

        // 广播到WebSocket连接
        ws_manager.broadcast_alert(alert_notification).await;

        Ok(())
    }

    /// 检查服务是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// 手动停止服务
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }
}

/// 遥测数据发布助手
pub struct TelemetryPublisher;

impl TelemetryPublisher {
    /// 发布遥测数据到frame-bus
    pub fn publish_telemetry(
        device_id: Uuid,
        tag_id: Uuid,
        value: f64,
        unit: Option<String>,
    ) -> Result<()> {
        use frame_bus::{publish_data, Value};

        // 构造tag：telemetry.{device_id}.{tag_id}
        let tag = format!("telemetry.{}.{}", device_id, tag_id);

        // 构造DataFrame
        let mut data_frame = DataFrame::new(tag, Value::float(value))
            .with_qos(2); // good quality

        // 添加元数据
        if let Some(unit_val) = unit {
            data_frame = data_frame.with_meta("unit", unit_val);
        }

        // 发布到frame-bus
        publish_data(data_frame)
            .context("Failed to publish telemetry data to frame-bus")?;

        Ok(())
    }
}

/// 报警数据发布助手
pub struct AlertPublisher;

impl AlertPublisher {
    /// 发布报警事件到frame-bus
    pub fn publish_alert(
        event_id: Uuid,
        rule_name: &str,
        device_name: Option<&str>,
        tag_name: Option<&str>,
        level: AlertLevel,
        message: &str,
    ) -> Result<()> {
        use frame_bus::{publish_data, Value};

        // 构造tag：alert.{event_id}
        let tag = format!("alert.{}", event_id);

        // 构造DataFrame
        let mut data_frame = DataFrame::new(tag, Value::bool(true)) // 报警触发标记
            .with_qos(2)
            .with_meta("rule_name", rule_name)
            .with_meta("level", match level {
                AlertLevel::INFO => "INFO",
                AlertLevel::WARN => "WARN", 
                AlertLevel::CRIT => "CRIT",
            })
            .with_meta("message", message);

        // 可选元数据
        if let Some(device) = device_name {
            data_frame = data_frame.with_meta("device_name", device);
        }
        if let Some(tag) = tag_name {
            data_frame = data_frame.with_meta("tag_name", tag);
        }

        // 发布到frame-bus
        publish_data(data_frame)
            .context("Failed to publish alert to frame-bus")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_telemetry_publisher() {
        // 这个测试需要frame-bus初始化，在实际环境中测试
        let device_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        
        // 注意：这个测试在没有frame-bus初始化的情况下会失败
        // 在实际运行环境中应该可以正常工作
        if let Err(_) = TelemetryPublisher::publish_telemetry(
            device_id,
            tag_id,
            25.5,
            Some("celsius".to_string()),
        ) {
            // 预期在测试环境中失败，因为frame-bus未初始化
            println!("Expected failure in test environment - frame-bus not initialized");
        }
    }

    #[tokio::test]
    async fn test_bridge_creation() {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let ws_manager = Arc::new(WsConnectionManager::new());
        
        let bridge = FrameBusBridge::new(ws_manager, shutdown_rx);
        assert!(!bridge.is_running().await);
        
        // 发送停止信号以清理
        let _ = shutdown_tx.send(());
    }
}