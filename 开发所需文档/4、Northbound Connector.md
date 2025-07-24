> **职责**：把 Frame Bus 中的 DataFrame/CmdFrame **推送到云端或分公司**，并把云端的 Cmd 请求注入 Frame Bus。  
> ★ 先聚焦 **MQTT 5**（MVP-0 必需）——其余协议全部插件化，按需追加。

---

## 1 设计目标 & 范畴

|目标|说明|
|---|---|
|**稳**|断网/掉 Broker 自动续传；云侧重复 ACK 不丢不重|
|**简**|内核与协议插件分离：`trait Connector` + 动态加载|
|**可观测**|吞吐、重试、延迟统一 Prom 指标|
|**扩展**|MQTT5 → Kafka → gRPC-stream → REST Batch —— 同一加载流程|

---

## 2 统一 Connector Trait

```rust
#[async_trait]
pub trait Connector: Send + Sync {
    fn meta(&self) -> ConnectorMeta;        // id, name, api, proto

    async fn start(
        &mut self,
        rx: FrameReceiver,                  // subscribe end
        cmd_tx: FrameSender                 // cloud cmd -> FrameBus
    ) -> Result<()>;
}

```

- **start** 在内部创建独立 Task：
    
    - 订阅 Bus → 批量发送
        
    - 订阅云端 Cmd → 转 `CmdFrame`
        

> 若未来换 Kafka/gRPC，只实现同 Trait，不改上层。

---

## 3 初版重点：MQTT 5 Connector

### 3.1 MQTT Topic & Payload

|类型|Topic 模板|Payload|
|---|---|---|
|**DataFrame**|`/edge/{tenant}/{device}/{tag}`|JSON (`Frame` 字段)|
|**CmdFrame Ack**|`/edge/ack/{seq}`|`{ result:"ok/fail", ts }`|
|**Cloud→Cmd**|`/edge/cmd/{tag}`|JSON `{ value, meta }`|

### 3.2 批量压缩

```scss
┌────────────┐    32 frames   ┌──── MQTT PUB QoS2 ───┐
│ Bus batch  │ ─────────────► │ zstd(json array)     │
└────────────┘                └──────────────────────┘

```

- Batch = min(32 Frame, 1 MiB, 1 s)
    
- 每 Frame `seq_no` 保留原顺序，用于重传对齐
    

### 3.3 断网续传

|阶段|行为|
|---|---|
|断线检测|rust-mqtt client emits `ConnectionLost`|
|重试策略|backoff 0.5 s→1 s→2 s→… 最多 2 min|
|缓存|写入 FrameBus WAL；MQTT Session-Expiry=1 day|
|恢复|读取上次 `PubRec` ack seq_no → 从 WAL seek 继续|

### 3.4 QoS 与可用性

- **QoS2**（Exactly-Once）——云端 Broker 需支持；
    
- 如果 Broker 只支持 QoS1，Connector 降级但在 WAL 记 `dup=possible`；
    
- Prometheus `edge_uplink_retry_total` 报警。
    

---

## 4 其他协议插件（计划）

|插件|适用场景|关键实现点|状态|
|---|---|---|---|
|**Kafka**|分公司用 Confluent|idempotent producer; PartKey=tag|待阶段 2|
|**gRPC-stream**|SaaS 服务|`stream FrameProto` + gzip; client ack seq|待阶段 3|
|**REST Batch**|一次性导入|`POST /frames` JSON array; Range-POST 续传|待阶段 3|
|**Arrow Flight**|大批量历史补录|Flight DoPut; Columnar Arrow|长期|

所有插件放 `plugins/connectors/`.

---

## 5 配置示例

```yaml
# connector.yml
mqtt_cloud:
  kind: mqtt5
  endpoint: tls://broker.iot.com:8883
  topic: "/edge/{tenant}/{device}/{tag}"
  qos: 2
  batch: 32
  timeout: 5s
  tls:
    ca: /certs/ca.pem
    client_cert: /certs/edge.crt
    client_key:  /certs/edge.key

```

- Manager 注入 `FrameReceiver`；Connector 自行维护 MQTT client与重试。
    

---

## 6 Metrics & Logging

|Metric|维度(label)|解释|
|---|---|---|
|`connector_out_bytes_total`|connector_id|实际发送字节|
|`connector_batch_size`|connector_id|每批 Frame 数|
|`connector_retry_total`|connector_id|重试次数|
|`connector_latency_ms`|connector_id|send→ack RTT|

**Logs**：`msg="mqtt_pub",seq=1234,status=ok/err`.

---

## 7 单元 & 集成测试

|测试|条件|断言|
|---|---|---|
|**batch_pub_ok**|docker EMQX，本地 100 Frame|收到 100 帧，顺序正确|
|**broker_down_30s**|30 s 禁用 EMQX|WAL 增长；恢复后全部补发|
|**dup_ack**|Broker 重发 PUBREC|Connector 不重发已 ack 帧|
|**QoS1 downgrade**|服务端限制 QoS1|本地标记 `dup`;消费端仍可识别|

---

## 8 版本演进

|版本|新功能|
|---|---|
|**0.1**|MQTT5 QoS2 + batch|
|**0.2**|TLS 双向、断网续传|
|**0.3**|Kafka connector dyn|
|**0.4**|gRPC-stream & REST|
|**1.0**|插件 ABI freeze|

---

## 9 开发分解（MVP-0 Scope）

Week 1-2

- rust-mqtt client Demo → QoS2 Publish/Subscribe
    
- Encode/Decode Frame JSON → Batch zstd
    

Week 3

- WAL seek + Session-Resume
    
- Prom metrics + unit tests
    

Week 4

- Compose-dev：EMQX + edge, k6 压测
    
- Blog：《用 Rust 写工业 MQTT 批量上传》
    

> 完成即满足 MVP-0 “采集→云”，剩余协议留待后续插件。