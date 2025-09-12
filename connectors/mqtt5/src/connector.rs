//! MQTT5连接器实现

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::sleep;
use anyhow::Result;
use serde_json::Value as JsonValue;
use uuid::Uuid;
use base64::engine::Engine;

use frame_bus::{FrameReceiver, Filter, DataFrame, Value};
use crate::config::{MqttCfg, MqttMessage, DataPoint};
use crate::metrics::METRICS;

use rumqttc::{MqttOptions, AsyncClient, QoS, Event, Packet};

/// MQTT5连接器
pub struct MqttConnector {
    cfg: MqttCfg,
    client: Option<AsyncClient>,
    buffer: Arc<Mutex<Vec<MqttMessage>>>,
    device_id: String,
}

impl MqttConnector {
    pub fn new(cfg: MqttCfg) -> Self {
        let device_id = if cfg.client_id.is_empty() {
            format!("gateway-{}", Uuid::new_v4())
        } else {
            cfg.client_id.clone()
        };

        Self {
            cfg,
            client: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            device_id,
        }
    }

    /// 初始化连接器
    pub async fn init(&mut self) -> Result<()> {
        tracing::info!("Initializing MQTT5 connector to {}", self.cfg.broker);

        let mut mqttoptions = MqttOptions::new(&self.device_id, &self.parse_broker_host()?, self.parse_broker_port()?);
        
        // 配置MQTT选项
        mqttoptions.set_keep_alive(self.cfg.keep_alive);
        // Note: rumqttc may not have set_connection_timeout method in this version
        // mqttoptions.set_connection_timeout(self.cfg.timeout);
        
        if !self.cfg.username.is_empty() {
            mqttoptions.set_credentials(&self.cfg.username, &self.cfg.password);
        }

        // TLS配置
        if self.cfg.tls.enabled {
            self.configure_tls(&mut mqttoptions).await?;
        }

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        self.client = Some(client);

        // 启动事件循环处理
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        METRICS.connect_total.inc();
                        tracing::info!("MQTT connected");
                    }
                    Ok(Event::Incoming(Packet::Disconnect)) => {
                        METRICS.disconnect_total.inc();
                        tracing::warn!("MQTT disconnected");
                    }
                    Err(e) => {
                        tracing::error!("MQTT connection error: {}", e);
                        METRICS.reconnect_total.inc();
                        sleep(Duration::from_secs(5)).await;
                    }
                    _ => {}
                }
            }
        });

        tracing::info!("MQTT5 connector initialized with device_id: {}", self.device_id);
        Ok(())
    }

    /// 启动连接器
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting MQTT5 connector");

        // 订阅DataFrame
        let rx = frame_bus::subscribe(Filter::data_only())?;
        let buffer1 = self.buffer.clone();
        let buffer2 = self.buffer.clone();
        let cfg1 = self.cfg.clone();
        let cfg2 = self.cfg.clone();
        let device_id = self.device_id.clone();
        let client = self.client.as_ref().unwrap().clone();

        // 启动数据处理任务
        let data_task = tokio::spawn(async move {
            Self::process_frames(rx, buffer1, device_id, cfg1.batch).await
        });

        // 启动批量发送任务
        let send_task = tokio::spawn(async move {
            Self::batch_sender(client, buffer2, cfg2).await
        });

        // 等待任务完成（实际上会一直运行）
        tokio::select! {
            result = data_task => {
                tracing::error!("Data processing task ended: {:?}", result);
            }
            result = send_task => {
                tracing::error!("Batch sender task ended: {:?}", result);
            }
        }

        Ok(())
    }

    /// 处理接收到的DataFrame
    async fn process_frames(
        mut rx: FrameReceiver,
        buffer: Arc<Mutex<Vec<MqttMessage>>>,
        device_id: String,
        batch_cfg: crate::config::BatchCfg,
    ) {
        let mut current_batch = Vec::new();
        let mut last_send = Instant::now();

        while let Ok(envelope) = rx.recv().await {
            // 解码 DataFrame；失败则跳过当前包
            let frame = match envelope.into_data() {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!("Failed to decode DataFrame from envelope: {}", e);
                    continue;
                }
            };

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let point = DataPoint {
                tag: frame.tag,
                value: Self::frame_value_to_json(frame.value.as_ref().unwrap_or(&Value::int(0))),
                quality: frame.qos as u8,
                meta: frame.meta,
            };

            current_batch.push(point);

            // 检查是否需要发送批次
            let should_send = current_batch.len() >= batch_cfg.size
                || last_send.elapsed() >= batch_cfg.timeout;

            if should_send && !current_batch.is_empty() {
                let message = MqttMessage {
                    device_id: device_id.clone(),
                    timestamp,
                    points: current_batch.drain(..).collect(),
                };

                let mut buffer_guard = buffer.lock().await;
                buffer_guard.push(message);
                METRICS.buffer_used.set(buffer_guard.len() as i64);
                drop(buffer_guard);

                last_send = Instant::now();
            }
        }
    }

    /// 批量发送器
    async fn batch_sender(
        client: AsyncClient,
        buffer: Arc<Mutex<Vec<MqttMessage>>>,
        cfg: MqttCfg,
    ) {
        loop {
            sleep(cfg.batch.timeout / 2).await;

            let messages = {
                let mut buffer_guard = buffer.lock().await;
                if buffer_guard.is_empty() {
                    continue;
                }
                let messages = buffer_guard.drain(..).collect::<Vec<_>>();
                METRICS.buffer_used.set(0);
                messages
            };

            for message in messages {
                if let Err(e) = Self::send_message(&client, &message, &cfg).await {
                    tracing::error!("Failed to send message: {}", e);
                    METRICS.publish_error_total.inc();
                    
                    // 重新放回缓冲区
                    let mut buffer_guard = buffer.lock().await;
                    buffer_guard.push(message);
                    METRICS.buffer_used.set(buffer_guard.len() as i64);
                } else {
                    METRICS.publish_total.inc();
                }
            }
        }
    }

    /// 发送单个消息
    async fn send_message(
        client: &AsyncClient,
        message: &MqttMessage,
        cfg: &MqttCfg,
    ) -> Result<()> {
        let start = Instant::now();
        
        let json_data = serde_json::to_vec(message)?;
        let payload = if cfg.compression.enabled && json_data.len() > cfg.compression.threshold {
            let compressed = zstd::encode_all(&json_data[..], cfg.compression.level)?;
            let ratio = compressed.len() as f64 / json_data.len() as f64;
            METRICS.compression_ratio.observe(ratio);
            compressed
        } else {
            json_data
        };

        let topic = format!("{}/data/{}", cfg.topic_prefix, message.device_id);
        let qos = match cfg.qos {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::ExactlyOnce,
        };

        METRICS.message_size.observe(payload.len() as f64);
        METRICS.batch_size.observe(message.points.len() as f64);

        client.publish(topic, qos, false, payload).await?;

        let latency = start.elapsed().as_millis() as f64;
        METRICS.publish_latency.observe(latency);

        Ok(())
    }

    /// 配置TLS
    async fn configure_tls(&self, _opts: &mut MqttOptions) -> Result<()> {
        if !self.cfg.tls.server_name.is_empty() {
            // TODO: 实现完整的TLS配置
            tracing::warn!("TLS configuration not fully implemented for MVP-0");
        }
        Ok(())
    }

    /// 解析broker主机
    fn parse_broker_host(&self) -> Result<String> {
        let url = url::Url::parse(&self.cfg.broker)?;
        Ok(url.host_str().unwrap_or("localhost").to_string())
    }

    /// 解析broker端口
    fn parse_broker_port(&self) -> Result<u16> {
        let url = url::Url::parse(&self.cfg.broker)?;
        Ok(url.port().unwrap_or(1883))
    }

    /// 转换Frame值为JSON
    fn frame_value_to_json(value: &Value) -> JsonValue {
        use frame_bus::envelope::value::Value as ValueEnum;
        match &value.value {
            Some(ValueEnum::BoolV(b)) => JsonValue::Bool(*b),
            Some(ValueEnum::IntV(i)) => JsonValue::Number((*i).into()),
            Some(ValueEnum::FloatV(f)) => {
                JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into()))
            }
            Some(ValueEnum::StrV(s)) => JsonValue::String(s.clone()),
            Some(ValueEnum::BinV(b)) => {
                JsonValue::String(base64::engine::general_purpose::STANDARD.encode(b))
            }
            None => JsonValue::Null,
        }
    }
}

