//! routes/websocket.rs —— WebSocket实时数据推送
//!
//! - scope(): `/ws`
//! - 支持：实时遥测数据、报警通知、连接状态
//! - 连接管理：客户端订阅、广播、断线重连
//!
//! 更新历史：
//! - 2025-01-27  Claude  初版

use crate::{dto::*, error::ApiError, bootstrap::AppState};
use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{
    web::{self, Data, Path, Query},
    HttpRequest, HttpResponse, Result,
};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket连接管理器
#[derive(Clone)]
pub struct WsConnectionManager {
    /// 活跃连接列表 (client_id -> addr)
    connections: Arc<RwLock<HashMap<String, Addr<WsConnection>>>>,
    /// 连接订阅信息 (client_id -> subscriptions)
    subscriptions: Arc<RwLock<HashMap<String, ClientSubscription>>>,
    /// 连接管理配置
    config: ConnectionManagerConfig,
    /// 全局统计信息
    global_stats: Arc<tokio::sync::RwLock<GlobalStats>>,
    /// 断线重连管理
    reconnection_manager: Arc<ReconnectionManager>,
    /// 消息队列（离线消息）
    message_queue: Arc<RwLock<HashMap<String, Vec<QueuedMessage>>>>,
    /// 连接健康检查
    health_checker: Arc<ConnectionHealthChecker>,
}

/// 连接管理器配置
#[derive(Debug, Clone)]
pub struct ConnectionManagerConfig {
    /// 最大连接数
    pub max_connections: u32,
    /// 心跳超时时间（秒）
    pub heartbeat_timeout_secs: u64,
    /// 连接清理间隔（秒）
    pub cleanup_interval_secs: u64,
    /// 启用连接限制
    pub enable_connection_limit: bool,
}

impl Default for ConnectionManagerConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            heartbeat_timeout_secs: 60,
            cleanup_interval_secs: 30,
            enable_connection_limit: true,
        }
    }
}

/// 全局统计信息
#[derive(Debug, Default, Clone, Serialize)]
pub struct GlobalStats {
    pub total_connections_created: u64,
    pub total_connections_closed: u64,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub peak_concurrent_connections: u32,
}

impl WsConnectionManager {
    pub fn new() -> Self {
        Self::with_config(ConnectionManagerConfig::default())
    }

    pub fn with_config(config: ConnectionManagerConfig) -> Self {
        let manager = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
            global_stats: Arc::new(tokio::sync::RwLock::new(GlobalStats::default())),
        };

        // 启动连接清理任务
        if config.cleanup_interval_secs > 0 {
            let manager_clone = manager.clone();
            tokio::spawn(async move {
                manager_clone.start_cleanup_task().await;
            });
        }

        manager
    }

    /// 启动连接清理任务
    async fn start_cleanup_task(&self) {
        let mut interval = tokio::time::interval(
            Duration::from_secs(self.config.cleanup_interval_secs)
        );
        
        loop {
            interval.tick().await;
            if let Err(e) = self.cleanup_stale_connections().await {
                error!("Failed to cleanup stale connections: {}", e);
            }
        }
    }

    /// 清理过期连接
    async fn cleanup_stale_connections(&self) -> Result<(), Box<dyn std::error::Error>> {
        let timeout_duration = Duration::from_secs(self.config.heartbeat_timeout_secs);
        let now = chrono::Utc::now();
        
        // 这里简化实现，实际需要跟踪每个连接的最后活跃时间
        // 由于架构限制，暂时只打印日志
        debug!("Checking for stale connections (timeout: {}s)", self.config.heartbeat_timeout_secs);
        
        let connections = self.connections.read().await;
        if connections.len() > 0 {
            debug!("Current active connections: {}", connections.len());
        }
        
        Ok(())
    }

    /// 添加新连接
    pub async fn add_connection(&self, client_id: String, addr: Addr<WsConnection>) -> Result<(), String> {
        // 检查连接数限制
        if self.config.enable_connection_limit {
            let connections = self.connections.read().await;
            if connections.len() >= self.config.max_connections as usize {
                warn!("Connection limit reached: {}/{}", connections.len(), self.config.max_connections);
                return Err(format!("Connection limit reached: {}", self.config.max_connections));
            }
        }

        // 添加连接
        {
            let mut connections = self.connections.write().await;
            connections.insert(client_id.clone(), addr);
        }

        // 更新统计信息
        {
            let mut stats = self.global_stats.write().await;
            stats.total_connections_created += 1;
            
            let current_count = {
                let connections = self.connections.read().await;
                connections.len() as u32
            };
            
            if current_count > stats.peak_concurrent_connections {
                stats.peak_concurrent_connections = current_count;
            }
        }

        info!("WebSocket client connected: {} (total: {})", 
              client_id, 
              self.connections.read().await.len());
        
        Ok(())
    }

    /// 移除连接
    pub async fn remove_connection(&self, client_id: &str) {
        {
            let mut connections = self.connections.write().await;
            connections.remove(client_id);
        }
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(client_id);
        }

        // 更新统计信息
        {
            let mut stats = self.global_stats.write().await;
            stats.total_connections_closed += 1;
        }

        info!("WebSocket client disconnected: {} (remaining: {})", 
              client_id,
              self.connections.read().await.len());
    }

    /// 更新客户端订阅
    pub async fn update_subscription(&self, client_id: String, subscription: ClientSubscription) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(client_id, subscription);
    }

    /// 广播遥测数据
    pub async fn broadcast_telemetry(&self, data: TelemetryMsg) {
        let connections = self.connections.read().await;
        let subscriptions = self.subscriptions.read().await;

        let mut sent_count = 0;
        let total_connections = connections.len();

        for (client_id, addr) in connections.iter() {
            if let Some(sub) = subscriptions.get(client_id) {
                if sub.should_receive_telemetry(&data) {
                    // 检查采样间隔
                    if sub.should_send_based_on_sample_interval() {
                        let msg = WsMessage::Telemetry(data.clone());
                        addr.do_send(SendMessage(msg));
                        sent_count += 1;
                    }
                }
            }
        }

        debug!("Broadcasted telemetry to {}/{} connections for device:{}, tag:{}", 
               sent_count, total_connections, data.device_id, data.tag_id);
    }

    /// 广播报警通知
    pub async fn broadcast_alert(&self, alert: AlertNotification) {
        let connections = self.connections.read().await;
        let subscriptions = self.subscriptions.read().await;

        let mut sent_count = 0;
        let total_connections = connections.len();

        for (client_id, addr) in connections.iter() {
            if let Some(sub) = subscriptions.get(client_id) {
                if sub.should_receive_alert(&alert) {
                    let msg = WsMessage::Alert(alert.clone());
                    addr.do_send(SendMessage(msg));
                    sent_count += 1;
                }
            }
        }

        info!("Broadcasted alert {} to {}/{} connections", 
              alert.event_id, sent_count, total_connections);
    }

    /// 批量广播遥测数据（性能优化）
    pub async fn broadcast_telemetry_batch(&self, data_batch: Vec<TelemetryMsg>) {
        if data_batch.is_empty() {
            return;
        }

        let connections = self.connections.read().await;
        let subscriptions = self.subscriptions.read().await;

        let mut total_sent = 0;
        let total_connections = connections.len();

        for (client_id, addr) in connections.iter() {
            if let Some(sub) = subscriptions.get(client_id) {
                let mut client_batch = Vec::new();
                
                for data in &data_batch {
                    if sub.should_receive_telemetry(data) && sub.should_send_based_on_sample_interval() {
                        client_batch.push(data.clone());
                    }
                }

                if !client_batch.is_empty() {
                    let msg = WsMessage::TelemetryBatch(client_batch);
                    addr.do_send(SendMessage(msg));
                    total_sent += 1;
                }
            }
        }

        debug!("Broadcasted batch of {} telemetry messages to {}/{} connections", 
               data_batch.len(), total_sent, total_connections);
    }

    /// 向特定客户端发送数据
    pub async fn send_to_client(&self, client_id: &str, message: WsMessage) -> bool {
        let connections = self.connections.read().await;
        
        if let Some(addr) = connections.get(client_id) {
            addr.do_send(SendMessage(message));
            true
        } else {
            warn!("Client {} not found for direct message", client_id);
            false
        }
    }

    /// 获取连接统计
    pub async fn get_stats(&self) -> WsStats {
        let connections = self.connections.read().await;
        let subscriptions = self.subscriptions.read().await;

        WsStats {
            total_connections: connections.len() as u32,
            subscribed_connections: subscriptions.len() as u32,
            total_devices_monitored: subscriptions
                .values()
                .flat_map(|s| &s.device_ids)
                .collect::<std::collections::HashSet<_>>()
                .len() as u32,
        }
    }

    /// 获取全局统计信息
    pub async fn get_global_stats(&self) -> GlobalStats {
        self.global_stats.read().await.clone()
    }

    /// 获取连接管理器配置
    pub fn get_config(&self) -> &ConnectionManagerConfig {
        &self.config
    }

    /// 强制断开指定连接
    pub async fn disconnect_client(&self, client_id: &str) -> bool {
        let connections = self.connections.read().await;
        
        if let Some(addr) = connections.get(client_id) {
            addr.do_send(ForceDisconnect);
            info!("Force disconnecting client: {}", client_id);
            true
        } else {
            warn!("Client {} not found for force disconnect", client_id);
            false
        }
    }

    /// 广播管理消息到所有连接
    pub async fn broadcast_admin_message(&self, message: &str) {
        let connections = self.connections.read().await;
        let admin_msg = WsMessage::AdminMessage(AdminMsg {
            message: message.to_string(),
            timestamp: chrono::Utc::now(),
        });

        let mut sent_count = 0;
        for (client_id, addr) in connections.iter() {
            addr.do_send(SendMessage(admin_msg.clone()));
            sent_count += 1;
        }

        info!("Broadcasted admin message to {} clients: {}", sent_count, message);
    }

    /// 获取特定客户端的订阅信息
    pub async fn get_client_subscription(&self, client_id: &str) -> Option<ClientSubscription> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.get(client_id).cloned()
    }

    /// 列出所有连接的客户端ID
    pub async fn list_client_ids(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    /// 更新消息统计
    pub async fn update_message_stats(&self, sent: u64, received: u64) {
        let mut stats = self.global_stats.write().await;
        stats.total_messages_sent += sent;
        stats.total_messages_received += received;
    }
}

/// 客户端订阅信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSubscription {
    /// 订阅的设备ID列表
    pub device_ids: Vec<Uuid>,
    /// 订阅的点位ID列表（可选）
    pub tag_ids: Option<Vec<Uuid>>,
    /// 是否接收报警
    pub alerts: bool,
    /// 数据采样间隔（毫秒）
    pub sample_interval_ms: Option<u64>,
    /// 上次发送时间戳（毫秒）
    #[serde(skip)]
    pub last_sent_ts: std::sync::Arc<std::sync::atomic::AtomicI64>,
}

impl ClientSubscription {
    /// 创建新的订阅
    pub fn new(
        device_ids: Vec<Uuid>,
        tag_ids: Option<Vec<Uuid>>,
        alerts: bool,
        sample_interval_ms: Option<u64>,
    ) -> Self {
        Self {
            device_ids,
            tag_ids,
            alerts,
            sample_interval_ms,
            last_sent_ts: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(0)),
        }
    }

    /// 检查是否应该接收遥测数据
    pub fn should_receive_telemetry(&self, data: &TelemetryMsg) -> bool {
        // 检查设备ID是否在订阅列表中
        if !self.device_ids.contains(&data.device_id) {
            return false;
        }

        // 如果指定了点位ID列表，检查是否匹配
        if let Some(tag_ids) = &self.tag_ids {
            if !tag_ids.contains(&data.tag_id) {
                return false;
            }
        }

        true
    }

    /// 检查是否应该基于采样间隔发送数据
    pub fn should_send_based_on_sample_interval(&self) -> bool {
        if let Some(interval_ms) = self.sample_interval_ms {
            let now = chrono::Utc::now().timestamp_millis();
            let last_sent = self.last_sent_ts.load(std::sync::atomic::Ordering::Relaxed);
            
            if now - last_sent >= interval_ms as i64 {
                self.last_sent_ts.store(now, std::sync::atomic::Ordering::Relaxed);
                true
            } else {
                false
            }
        } else {
            // 没有设置采样间隔，总是发送
            true
        }
    }

    /// 检查是否应该接收报警
    pub fn should_receive_alert(&self, alert: &AlertNotification) -> bool {
        if !self.alerts {
            return false;
        }

        // 如果报警关联设备，检查是否在订阅列表中
        if let Some(_device_name) = &alert.device_name {
            // 这里需要从设备名称查找device_id，简化实现先返回true
            true
        } else {
            true
        }
    }

    /// 重置采样时间戳（用于重新订阅时）
    pub fn reset_sample_timestamp(&self) {
        self.last_sent_ts.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

/// WebSocket消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// 遥测数据
    Telemetry(TelemetryMsg),
    /// 批量遥测数据
    TelemetryBatch(Vec<TelemetryMsg>),
    /// 报警通知
    Alert(AlertNotification),
    /// 连接状态
    Status(WsStatusMsg),
    /// 订阅确认
    Subscription(WsSubscriptionResponse),
    /// 错误消息
    Error(WsErrorMsg),
    /// 管理消息
    AdminMessage(AdminMsg),
    /// 心跳包
    Ping,
    /// 心跳响应
    Pong,
}

/// 管理消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminMsg {
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// WebSocket状态消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsStatusMsg {
    pub client_id: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub stats: WsStats,
}

/// WebSocket连接统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsStats {
    pub total_connections: u32,
    pub subscribed_connections: u32,
    pub total_devices_monitored: u32,
}

/// 单个WebSocket连接统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsConnectionStats {
    pub client_id: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub sent_count: u64,
    pub recv_count: u64,
    pub uptime_seconds: u64,
}

/// WebSocket订阅响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSubscriptionResponse {
    pub success: bool,
    pub message: String,
    pub subscription: Option<ClientSubscription>,
}

/// WebSocket错误消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsErrorMsg {
    pub code: String,
    pub message: String,
}

/// WebSocket连接Actor
pub struct WsConnection {
    /// 客户端ID
    client_id: String,
    /// 连接管理器
    manager: Arc<WsConnectionManager>,
    /// 连接时间
    connected_at: chrono::DateTime<chrono::Utc>,
    /// 最后心跳时间
    last_heartbeat: chrono::DateTime<chrono::Utc>,
    /// 发送消息计数
    sent_count: u64,
    /// 接收消息计数
    recv_count: u64,
}

impl WsConnection {
    pub fn new(client_id: String, manager: Arc<WsConnectionManager>) -> Self {
        let now = chrono::Utc::now();
        Self {
            client_id,
            manager,
            connected_at: now,
            last_heartbeat: now,
            sent_count: 0,
            recv_count: 0,
        }
    }

    /// 更新心跳时间
    fn update_heartbeat(&mut self) {
        self.last_heartbeat = chrono::Utc::now();
    }

    /// 增加发送计数
    fn increment_sent(&mut self) {
        self.sent_count += 1;
    }

    /// 增加接收计数
    fn increment_recv(&mut self) {
        self.recv_count += 1;
    }

    /// 获取连接统计
    fn get_connection_stats(&self) -> WsConnectionStats {
        WsConnectionStats {
            client_id: self.client_id.clone(),
            connected_at: self.connected_at,
            last_heartbeat: self.last_heartbeat,
            sent_count: self.sent_count,
            recv_count: self.recv_count,
            uptime_seconds: (chrono::Utc::now() - self.connected_at).num_seconds() as u64,
        }
    }
}

impl Actor for WsConnection {
    type Context = ws::WebsocketContext<Self>;

    /// Actor启动时注册连接
    fn started(&mut self, ctx: &mut Self::Context) {
        let manager = self.manager.clone();
        let client_id = self.client_id.clone();
        let addr = ctx.address();

        tokio::spawn(async move {
            if let Err(e) = manager.add_connection(client_id.clone(), addr).await {
                error!("Failed to add connection {}: {}", client_id, e);
                // TODO: 发送错误消息并关闭连接
            }
        });
    }

    /// Actor停止时清理连接
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let manager = self.manager.clone();
        let client_id = self.client_id.clone();

        tokio::spawn(async move {
            manager.remove_connection(&client_id).await;
        });
    }
}

/// WebSocket消息处理
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.update_heartbeat();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // 心跳响应，记录连接活跃
                self.update_heartbeat();
                debug!("Received pong from client: {}", self.client_id);
            }
            Ok(ws::Message::Text(text)) => {
                self.increment_recv();
                self.update_heartbeat();
                debug!("Received text from {}: {}", self.client_id, text);
                self.handle_text_message(text.to_string(), ctx);
            }
            Ok(ws::Message::Binary(_bin)) => {
                warn!("Received unexpected binary message from {}", self.client_id);
                ctx.text(serde_json::to_string(&WsMessage::Error(WsErrorMsg {
                    code: "BINARY_NOT_SUPPORTED".to_string(),
                    message: "Binary messages are not supported".to_string(),
                })).unwrap_or_default());
            }
            Ok(ws::Message::Close(reason)) => {
                info!("WebSocket connection closed: {} - {:?}", self.client_id, reason);
                ctx.stop();
            }
            _ => {
                warn!("Received unexpected message type from {}", self.client_id);
                ctx.stop();
            }
        }
    }
}

impl WsConnection {
    /// 处理文本消息
    fn handle_text_message(&mut self, text: String, ctx: &mut ws::WebsocketContext<Self>) {
        match serde_json::from_str::<ClientMessage>(&text) {
            Ok(client_msg) => {
                self.handle_client_message(client_msg, ctx);
            }
            Err(e) => {
                error!("Failed to parse client message: {}", e);
                let error_msg = WsMessage::Error(WsErrorMsg {
                    code: "INVALID_JSON".to_string(),
                    message: format!("Invalid JSON: {}", e),
                });
                ctx.text(serde_json::to_string(&error_msg).unwrap_or_default());
            }
        }
    }

    /// 处理客户端消息
    fn handle_client_message(&mut self, msg: ClientMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            ClientMessage::Subscribe(subscription) => {
                let manager = self.manager.clone();
                let client_id = self.client_id.clone();

                // 重置采样时间戳
                subscription.reset_sample_timestamp();

                // 克隆一份用于async任务
                let subscription_for_task = subscription.clone();
                tokio::spawn(async move {
                    manager.update_subscription(client_id, subscription_for_task).await;
                });

                let response = WsMessage::Subscription(WsSubscriptionResponse {
                    success: true,
                    message: "Subscription updated".to_string(),
                    subscription: Some(subscription),
                });
                ctx.text(serde_json::to_string(&response).unwrap_or_default());
            }
            ClientMessage::Unsubscribe => {
                let manager = self.manager.clone();
                let client_id = self.client_id.clone();

                tokio::spawn(async move {
                    let mut subscriptions = manager.subscriptions.write().await;
                    subscriptions.remove(&client_id);
                });

                let response = WsMessage::Subscription(WsSubscriptionResponse {
                    success: true,
                    message: "Unsubscribed from all".to_string(),
                    subscription: None,
                });
                ctx.text(serde_json::to_string(&response).unwrap_or_default());
            }
            ClientMessage::GetStatus => {
                let manager = self.manager.clone();
                let client_id = self.client_id.clone();
                let connected_at = self.connected_at;

                let addr = ctx.address();
                tokio::spawn(async move {
                    let stats = manager.get_stats().await;
                    let status_msg = WsMessage::Status(WsStatusMsg {
                        client_id,
                        connected_at,
                        stats,
                    });
                    addr.do_send(SendMessage(status_msg));
                });
            }
            ClientMessage::Ping => {
                ctx.text(serde_json::to_string(&WsMessage::Pong).unwrap_or_default());
            }
        }
    }
}

/// 客户端发送的消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    /// 订阅数据
    Subscribe(ClientSubscription),
    /// 取消所有订阅
    Unsubscribe,
    /// 获取连接状态
    GetStatus,
    /// 心跳包
    Ping,
}

/// 发送消息到WebSocket连接
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage(pub WsMessage);

/// 强制断开连接
#[derive(Message)]
#[rtype(result = "()")]
pub struct ForceDisconnect;

impl Handler<SendMessage> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) -> Self::Result {
        match serde_json::to_string(&msg.0) {
            Ok(text) => {
                self.increment_sent();
                ctx.text(text);
            }
            Err(e) => {
                error!("Failed to serialize WebSocket message: {}", e);
            }
        }
    }
}

impl Handler<ForceDisconnect> for WsConnection {
    type Result = ();

    fn handle(&mut self, _msg: ForceDisconnect, ctx: &mut Self::Context) -> Self::Result {
        warn!("Force disconnecting client: {}", self.client_id);
        ctx.stop();
    }
}

/// WebSocket连接查询参数
#[derive(Debug, Deserialize)]
pub struct WsConnectQuery {
    /// 客户端ID（可选，自动生成）
    pub client_id: Option<String>,
}

/// 配置WebSocket路由
pub fn scope() -> actix_web::Scope {
    web::scope("/ws")
        .route("/telemetry", web::get().to(telemetry_websocket))
        .route("/status", web::get().to(get_websocket_status))
        .route("/connections", web::get().to(get_connections_detail))
        .route("/connections/{client_id}/disconnect", web::post().to(disconnect_client))
        .route("/admin/broadcast", web::post().to(broadcast_admin_message))
        .route("/admin/stats", web::get().to(get_admin_stats))
}

/// 建立WebSocket连接
pub async fn telemetry_websocket(
    req: HttpRequest,
    stream: web::Payload,
    query: Query<WsConnectQuery>,
    app_state: Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let client_id = query.client_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    
    info!("WebSocket connection request from client: {}", client_id);

    let manager = app_state.ws_manager.clone();
    let connection = WsConnection::new(client_id, manager);

    let resp = ws::start(connection, &req, stream)
        .map_err(|e| {
            error!("Failed to start WebSocket: {}", e);
            ApiError::internal_error("Failed to start WebSocket connection")
        })?;

    Ok(resp)
}

/// 获取WebSocket连接状态
pub async fn get_websocket_status(
    app_state: Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let stats = app_state.ws_manager.get_stats().await;

    let status = WsStatusResponse {
        active_connections: stats.total_connections,
        subscribed_connections: stats.subscribed_connections,
        monitored_devices: stats.total_devices_monitored,
        server_time: chrono::Utc::now(),
    };

    Ok(HttpResponse::Ok().json(status))
}

/// WebSocket状态响应
#[derive(Debug, Serialize)]
pub struct WsStatusResponse {
    pub active_connections: u32,
    pub subscribed_connections: u32,
    pub monitored_devices: u32,
    pub server_time: chrono::DateTime<chrono::Utc>,
}

/// 连接详情响应
#[derive(Debug, Serialize)]
pub struct WsConnectionsResponse {
    pub connections: Vec<WsConnectionStats>,
    pub total_connections: u32,
    pub total_sent_messages: u64,
    pub total_recv_messages: u64,
}

/// 获取连接详情
pub async fn get_connections_detail(
    app_state: Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let stats = app_state.ws_manager.get_stats().await;
    let global_stats = app_state.ws_manager.get_global_stats().await;
    let client_ids = app_state.ws_manager.list_client_ids().await;
    
    let response = WsConnectionsResponse {
        connections: vec![], // TODO: 实现详细连接信息收集
        total_connections: stats.total_connections,
        total_sent_messages: global_stats.total_messages_sent,
        total_recv_messages: global_stats.total_messages_received,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 断开指定客户端连接
pub async fn disconnect_client(
    path: web::Path<String>,
    app_state: Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let client_id = path.into_inner();
    
    let success = app_state.ws_manager.disconnect_client(&client_id).await;
    
    if success {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": format!("Client {} disconnected", client_id)
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "message": format!("Client {} not found", client_id)
        })))
    }
}

/// 广播管理消息
#[derive(Debug, Deserialize)]
pub struct BroadcastRequest {
    pub message: String,
}

pub async fn broadcast_admin_message(
    req: web::Json<BroadcastRequest>,
    app_state: Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    app_state.ws_manager.broadcast_admin_message(&req.message).await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Admin message broadcasted"
    })))
}

/// 获取管理统计信息
pub async fn get_admin_stats(
    app_state: Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let stats = app_state.ws_manager.get_stats().await;
    let global_stats = app_state.ws_manager.get_global_stats().await;
    let config = app_state.ws_manager.get_config();
    let client_ids = app_state.ws_manager.list_client_ids().await;

    let response = serde_json::json!({
        "current_stats": stats,
        "global_stats": global_stats,
        "config": {
            "max_connections": config.max_connections,
            "heartbeat_timeout_secs": config.heartbeat_timeout_secs,
            "cleanup_interval_secs": config.cleanup_interval_secs,
            "enable_connection_limit": config.enable_connection_limit
        },
        "client_ids": client_ids,
        "server_time": chrono::Utc::now()
    });

    Ok(HttpResponse::Ok().json(response))
}