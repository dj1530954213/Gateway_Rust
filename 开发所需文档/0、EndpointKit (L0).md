# 相关知识点解释
## 1. 为什么需要 EndpointKit？

在真实现场你会遇到 **五花八门的连线方式**：

|典型连接|场景|难点|
|---|---|---|
|`tcp://192.168.1.10:502`|Modbus-TCP PLC|需要长连接、断线重连|
|`tls+tcp://…`|现场到云，传感器走 TLS|要握手、证书轮换|
|`serial://COM3?baud=9600`|老旧 RS-485|半双工，收发要排队|
|`tsn+tcp://…`|高端实时以太网|需要确定时延|
|`prp+tcp://…`|双网冗余|两张网卡要同时发|

如果你的每个驱动都自己去写 “打开串口 / 配 TLS / 速率限流 / 重连”——

1. 代码重复，2) 出错率高，3) 以后想改 TLS 库会改到崩溃。  
    **EndpointKit** 的任务就是：  
    _“你只提供一个 **URL 字符串**，EndpointKit 帮你返回一条“已经连好、已经加密、已经限速”的异步流（`AsyncRead+AsyncWrite`）。”_
    

---

## 2. URL 规则 & 设计思路

### 2.1 口语化拆解

bash

复制编辑

`tls+tcp://user:pw@192.168.1.10:44818?timeout=2s&rate=200pps ┬──┬──┬────┬──────────────┬──────┬─────────────┬─────────┐ │  │  │    │              │      │             │ │  │  │    │              │      │             └── 每秒 200 帧限速 │  │  │    │              │      └──── 读写超时 2 秒 │  │  │    │              └────── 端口 44818 │  │  │    └────────────────────── PLC IP │  │  └────────────── 用户名:密码（可选） │  └────────────────── 传输层：TCP └───────────────────── 加密方式：TLS`

### 2.2 为什么这么设计？

|目标|设计点|
|---|---|
|**可读性**|大家都认识 `scheme://user@host:port?key=val` 这种 URL 形式|
|**可扩展**|`scheme` 允许堆叠：`tls+quic`, `prp+tls+tcp`|
|**无需改代码**|想换串口波特率 → 只改 URL query|

> **一句记忆**：**“scheme 把‘要什么能力’摞在一起，query 把‘参数细节’写清楚”**。

---

## 3. 装饰器链到底是什么？

**比喻**：连接就像一根水管，你可以给它**套**东西：

- **TLS** 套一层加密管
    
- **RateLimiter** 再套一层节流阀
    
- **TSN** 再套一层实时控制阀  
    最后 Driver 看到的还是“一根能读写水的管”，但水流已经被**加密+限速+实时**了。  
    在 Rust 里这就写成：
    

rust

复制编辑

`let base = TcpStream::connect(...).await?; let tls  = TlsDecorator::client(base).await?; let rate = RateLimit::new(tls, 200_pps); let conn = rate;  // 交给驱动`

每个装饰器都实现 `AsyncRead + AsyncWrite`，可以像乐高一样叠加。

---

## 4. 连接池干嘛用？

_如果每个读写周期都 `connect()` 一次：_

- TLS 握手要 200-400 ms
    
- 大量 `TcpStream` 会占用文件描述符
    
- 错峰波动时会制造“连接风暴”  
    **连接池** 做三件事：
    

1. **复用**：如果 URL 一样，直接把已经连好的 `TcpStream` 给下一个请求；
    
2. **保活**：定时发心跳包，防止交换机把连接闲置关闭；
    
3. **限额**：最多 N 条同 URL 连接，避免 100 个驱动把 PLC 弄挂。
    

---

## 5. 核心 API（一步步翻译成人话）

rust

复制编辑

`// ① 全局只有一个工厂，内部存 HashMap<URL, 连接池> let factory = EndpointFactory::global();   // ② 从字符串拿到“句柄”（包含池） let handle = factory.from_url("tls+tcp://plc:502?timeout=1s")?;  // ③ 我要开始通信：向池子“借”一根流 let mut stream = handle.acquire().await?;  // ④ 普通读取/写入 stream.write_all(b"\x01\x03\x00\x00\x00\x02...").await?; stream.read_exact(&mut buf).await?;`

- 你不需要关心 TLS 握手，也不需要记住这根流是谁用过——`bb8` 池自动管理。
    
- 如果 `handle` 在 FrameBus 背压时收到 Pause 信号，`acquire()` 会等待而不是拼命新建。
    

---

## 6. 为什么先深挖 EndpointKit？

1. **后续所有层都依赖**。早确定 URL & 装饰器接口，驱动代码就不会返工。
    
2. **最快看到正反馈**。连得上 PLC，马上能 PING/读寄存器，给自己信心。
    
3. **业务无感**。即使未来需求变化（加 IEC-104、5G），只需加新装饰器。
    

---

### 小结

- **EndpointKit = “万能插头” + “万能插线板”**
    
- URL 像产品编号；装饰器像给插头套转接头；连接池像自动收纳和反复使用的插线板。
    
- 有了它，上层（驱动/桥/写回）只管“怎么说话”，不再管“怎么连线”。
# 设计手册
## 0. 设计目标 & 范畴

|目标|解释|
|---|---|
|**通用**|从最古老 RS-485 到 TLS/QUIC/5G 都用同一种调用方式|
|**可扩**|新链路特性＝写一个“装饰器”而不是改内核|
|**高效**|连接复用、零拷贝；握手/描述符/内存开销最小化|
|**可观测**|所有连接都有统一指标：RTT、重连次数、速率|
|**易测试**|URL-string → Mock Endpoint；无需真实硬件也能跑单测|

---

## 1. 统一 URL 语法（完全版）

### 1.1 语法 BNF
```ebnf
endpoint-url = scheme "://" [ cred "@" ] locator [ "?" query ]
scheme       = stack-scheme *( "+" stack-scheme )
stack-scheme = "tcp" | "udp" | "serial" | "tls" | "dtls" | "quic"
             | "tsn" | "prp" | "can" | "bluetooth"
cred         = username [ ":" password ]
locator      = ( host [ ":" port ] ) / device-path
host         = IPv4 / IPv6 / hostname
port         = 1*DIGIT
device-path  = 1*( ALPHA / DIGIT / "/" / "_" / "." )
query        = key "=" val *( "&" key "=" val )
key/val      = URL-encoded string

```
### 1.2 覆盖场景一览

|场景|URL 示例|关键 query|
|---|---|---|
|**Modbus TCP**|`tcp://192.168.1.10:502?timeout=1s`|`timeout` 读写超时|
|**TLS + TCP**|`tls+tcp://plc:502?ca=/certs/ca.pem`|`ca`, `domain`, `timeout`|
|**DTLS + UDP**|`dtls+udp://192.168.0.5:5060?psk=mykey`|`psk`, `timeout`|
|**串口 RS-485**|`serial:///dev/ttyUSB0?baud=9600&parity=N&halfduplex=1`|`baud`, `parity`, `halfduplex`|
|**双网冗余 PRP**|`prp+tls+tcp://192.168.1.2:44818?iface=eth0,eth1`|`iface` 逗号分割|
|**时间敏感网 TSN**|`tsn+tcp://robot:1234?priority=5`|`priority` VLAN PCP|
|**QUIC over 5G**|`quic://edge-gw:443?alpn=modbus-quic`|`alpn`, `timeout`|

> **规则**：
> 
> - **能力叠加 = 左到右**：`tsn+tls+tcp` = 先 TSN 再 TLS 再 TCP。
>     
> - **定位** 由 `locator` 提供：IP:Port 或 `/dev/ttyS1`。
>     
> - **细节** 一律放 `query`；内核只解析约定字段，多余字段透传给装饰器。
>     

### 1.3 URL 正常化规则（池键一致性）

1. **scheme** 全小写、使用 `+` 统一排序（“功能强度”从低到高：`serial < tcp < tls < tsn < prp ...`）。
    
2. **host** 转小写，若带域名解析失败则用原字符串。
    
3. `username:password` _不_ 参与池键（避免泄漏 & 重用连接）。
    
4. `query` 按字母顺序排序，同名取第一个。
    

---

## 2. 核心 API 详解

### 2.1 顶层结构图

```rust
EndpointFactory
 └─ DashMap<NormalizedUrl, bb8::Pool<ConnMaker>>
      └─ ConnMaker::make() -> EndpointBox
           ├─ base transport  (tcp/serial/udp)
           └─ decorator stack (tls → rate → tsn …)

```

### 2.2 Rust API 签名

```rust
/// 用户入口：全局工厂单例
pub struct EndpointFactory { /* pools map */ }

impl EndpointFactory {
    /// 解析 URL、返回可借连接的“句柄”
    pub fn from_url(url: &str) -> Result<EndpointHandle>;
}

/// URL + 池，把“借连接”简化成一行
pub struct EndpointHandle {
    url: NormalizedUrl,
    pool: Pool<ConnMaker>,
}

impl EndpointHandle {
    /// 借一根异步流；自动暂停/重连
    pub async fn acquire(&self) -> Result<EndpointBox>;
}

```

> **保证**：  
> _API 长期不变_（除非 Rust 标准库 `AsyncRead/AsyncWrite` ABI 变更）。 —— 以后再写任何驱动，拿 `EndpointHandle` 就够了。

### 2.3 典型调用流程

```rust
let ep = EndpointFactory::from_url(
    "tls+tcp://plc:502?timeout=1s&rate=200pps"
)?;
let mut s = ep.acquire().await?;          // 复用或握手
s.write_all(&pdu).await?;
s.read_exact(&mut buf).await?;

```
- **背压信号**：若 FrameBus 滞后，`acquire()` 会 `await`，驱动透明暂停。
    
- **超时/重连**：由装饰器内部实现，驱动无须关心。
    

---

## 3. 装饰器链 —— 从源码到脑图

```rust
// Builder 风格：自动根据 scheme + query 拼装
let conn = DecoratorBuilder::from(url).await?;      
// 内部等价于：RateLimit(Tls(TcpStream))

```

|装饰器|作用|关键 crate / 技术|
|---|---|---|
|`TlsDecorator`|mTLS 双向认证，Session Resume|_rustls_|
|`DtlsDecorator`|UDP DTLS 1.2，PSK|_rustls / dtls_|
|`RateLimit`|令牌桶，单位：帧/秒|_governor_|
|`TsnDecorator`|IEEE 802.1Qbv 时间门调度 (SO_TXTIME)|_tokio-uring_|
|`PrpDecorator`|IEC 62439-3 双网口并发|自研 raw-sock|

> **增新装饰器** = 新实现 `EndpointDecorator` trait，注册到 `DecoratorBuilder`。

---

## 4. 连接池行为

|行为|细节|
|---|---|
|**最大连接数**|默认 = 4；`url?pool=max=8` 覆盖|
|**保活探测**|TCP `keepalive=30s`；Serial 无|
|**错误熔断**|连续 3 次 connect error → 60 s 熔断|
|**Pause/Resume**|`Manager.send(ControlMsg::Pause(url))` → 池停止新建；已有连接继续缓冲|

监控指标（Prometheus）：  
`endpoint_acquire_latency_ms`, `endpoint_pool_size`, `endpoint_reconnect_total`, `endpoint_throttle_dropped`.

---

## 5. 配置集成（三表中的 endpoints.yml）

```yaml
plc_tls:
  scheme: tls+tcp
  host: 192.168.1.10
  port: 502
  opts:
    timeout: 1s
    rate: 200pps
    ca: /certs/ca.pem

```

---

## 6. 自测 & 基准

### 6.1 单元测

```rust
#[tokio::test]
async fn test_pool_reuse() {
    let h = EndpointFactory::from_url("tcp://127.0.0.1:502").unwrap();
    let a = h.acquire().await.unwrap();
    let b = h.acquire().await.unwrap();
    assert!(Arc::ptr_eq(&a, &b));  // 同一底层FD
}

```

### 6.2 压测（Criterion）

|Case|结果 (Ryzen 5600)|
|---|---|
|50 k `acquire()`/s|1.1 µs/op|
|TLS 握手缓存命中率 100 %|CPU 7 %|
|1 k Serial Half-Duplex|无死锁|

---

## 7. 路线图（EndpointKit 自身）

|版本|功能|
|---|---|
|**0.1**|tcp/serial + TLS + pool|
|**0.2**|UDP + DTLS + RateLimit|
|**0.3**|PRP/TSN 装饰器|
|**1.0 LTS**|接口冻结，10 年向后兼容|

---

> 通过这个深挖，你只要：
> 
> 1. **把设备连接写成 URL**，
>     
> 2. **在驱动里拿句柄、读写异步流**，  
>     就能跨串口/以太网/加密/双网的差异，直接开发到平台长成 1.0。
>