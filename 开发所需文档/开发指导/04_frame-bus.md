# 04 – FrameBus & WAL 设计文档 (v1‑freeze)

## 1. 责任
单进程广播环 + RocksDB WAL；Exactly‑once 保序。

## 2. Frame 结构 (Protobuf)
DataFrame, CmdFrame, Envelope(seq,u64 + kind + payload bytes)。

## 3. API
`init()`, `publish(env)`, `subscribe(Filter)`, `ack(consumer,seq)`.

## 4. Ring & 背压
tokio::broadcast 容量 2^pow；80 % Pause / 60 % Resume。

## 5. WAL
ColumnFamily frames/acks；append flush 100 ms；GC min ack。

## 6. Prom
ring_used, publish_total, drop_total, backlog_lag, wal_bytes。

## 7. 错误
RocksDB failure ⇒ panic + supervisor 重启。

## 8. 测试
replay_after_restart, ring_overflow_pause, ack_gc_threshold, pause_resume_hysteresis。

## 9. 性能
5 k fps P99 < 150 µs；WAL flush < 5 ms。

## 10. 兼容
Protobuf 仅新增字段，保留 tag。

## 11. 里程碑
0.1 broadcast+WAL; 0.2 GC + delta wal; 1.0 API freeze。