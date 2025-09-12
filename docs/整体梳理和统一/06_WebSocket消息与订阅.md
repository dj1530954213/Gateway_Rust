# WebSocket 消息与订阅

## 路由

- `/ws/telemetry` 建立连接并通过客户端消息进行订阅/退订
- `/ws/status` 获取服务端状态

## 消息模型（`core/web-gw-api/src/routes/websocket.rs`）

- `WsMessage::{Telemetry, TelemetryBatch, Alert, Status, Subscription, Error, AdminMessage, Ping, Pong}`
- 客户端消息：`ClientMessage::{Subscribe(ClientSubscription), Unsubscribe, GetStatus, Ping}`
- 订阅：设备/点位过滤、可选采样间隔、报警开关

## 可靠性与治理

- 心跳：Ping/Pong + 心跳超时清理
- 广播限速与采样：避免风暴
- 管理消息：维护窗口、全局告警广播

## 安全

- 连接鉴权（后续）：Token/ApiKey -> 限制订阅范围
