# 05 – MQTT 5 Connector 设计文档 (v1‑freeze)

## 1. 范围
FrameBus ↔ MQTT5 QoS2 uplink / cmd downlink；批量压缩 zstd。

## 2. 配置字段
endpoint, client_id, topic, qos, batch(max_frames/bytes/interval), keepalive, session_expiry, tls, backoff。

## 3. 批量规则
flush 条件 = frames≥N ∨ bytes≥M ∨ tick。

## 4. QoS2 State
Inflight map pkid→seqs；PubRec→PubRel→PubComp→ack(seq)。

## 5. Downlink
订阅 `/edge/cmd/#` JSON {seq,tag,value} → CmdFrame(origin="mqtt").

## 6. 重连
指数退避 500 ms→2 min；Session‑Expiry 1d。

## 7. Prom
mqtt_out_bytes_total, batch_frames, retry_total, rtt_ms。

## 8. 测试
batch_flush_size, qos2_dup_rec, broker_disconnect_replay, downlink_cmdframe。

## 9. 性能
5 k fps 连续 10 min 0 drop；平均 RTT < 20 ms。

## 10. 里程碑
0.1 uplink; 0.2 downlink; 0.3 TLS/mTLS; 1.0 插件 ABI freeze。