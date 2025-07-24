# 02 – EndpointKit 设计文档 (v1‑freeze)

> L0 · 统一连接 / 装饰器 / 连接池  
> 目标：同一 URL 复用；装饰链可扩；Acquire 热路径 ≤ 1.5 µs

## 1. 目标 / 非目标
|Must|• TLS、Rate、HalfDuplex 等装饰器链<br>• Pause/Resume 背压<br>• 端点热复用|
|Won’t|• 不跨进程；不负责 Modbus/OPC 协议握手|

## 2. API
```rust
fn init(cfg: BusCfg) -> Result<()>;
struct EndpointHandle;
impl EndpointHandle { async fn acquire(&self)->Result<EndpointBox>; }
struct EndpointFactory;
impl EndpointFactory { fn from_url(s:&str)->Result<EndpointHandle>; }
```

## 3. URL 语法
BNF 及示例参见章节 3。支持 tcp、tls、serial、udp、dtls、tsn、prp。

## 4. 核心数据结构
* `Scheme`, `EndpointUrl`, `NormalizedUrl` struct 描述  
* Hash = host+port+sorted query 去掉凭证

## 5. 装饰器链
倒序包裹：Base→TLS→PRP→Rate→HalfDuplex。Query key—装饰器映射表。

## 6. 连接池
* bb8 池大小默认 4 (`pool=max=` 覆盖)  
* IoFail 3 次 ⇒熔断 60 s

## 7. 背压 / Control
`Control::Pause/Resume/Drain` broadcast; 80 % → Pause, 60 % → Resume。

## 8. Prom 指标
`endpoint_acquire_latency_us`, `endpoint_pool_size`, `endpoint_pause_total`, `endpoint_reconnect_total`

## 9. 错误模型
`EndpointError::{Io,Timeout,UnsupportedScheme,PoolExhausted}`

## 10. 测试矩阵
|用例|断言|
|url_roundtrip|parse→display 恒等|
|tls_handshake|对 openssl 成功|
|rate_limit|Acquire 速率≤设置|
|pause_resume|收到 Pause 停 3 s|

## 11. 基准
Acquire 热 ≤ 1.5 µs；TLS 握手冷 ≤ 3 ms

## 12. 向前兼容
枚举只能新增；新字段 `serde(default)`。

## 13. 里程碑
0.1 TCP/TLS/Serial; 0.2 UDP/DTLS; 0.3 PRP; 1.0 API freeze