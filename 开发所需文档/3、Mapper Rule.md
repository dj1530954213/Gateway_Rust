## 1. 角色定位：站在 L2 与 L4 之间

```arduino
Driver → FrameBus ──► ① Mapper ② RuleEngine ──► FrameBus → Connector/Bridge
                                   │
                                   └──► Alert / Local Actuator (旁路)

```

- **Mapper** = **静态点位映射 & 单位换算**
    
- **RuleEngine** = **在线计算 / 告警 / 聚合**  
    二者共享同一 DSL 与热加载机制，但执行优先级：Mapper → RuleEngine。
    

---

## 2. 配置 DSL — 三层文件

|文件|粒度|写的人|版本控制|
|---|---|---|---|
|`variables.yml`|每个 Tag 的**静态属性**|工艺工程师|Git (随 PLC 点表)|
|`mapping.yml`|原始 Tag → 业务 Tag 的**转换**|OT&IT 协作|Git / etcd|
|`rules.yml`|阈值、窗口、派生计算|工艺 / 数据工程师|etcd 热加载|

### 2.1 `variables.yml`（核心列）

```yaml
- tag: plc.flow_raw
  driver_id: modbus1
  address: {type: holding, addr: 40001, len: 2}
  datatype: uint16
  scale: "value / 10.0"
  unit: m3_h
  access: R

```

- 解析后做三件事：
    
    1. 给驱动生成读/写表
        
    2. 把 `scale` 预编译为 Lua 函数
        
    3. 存入 Tag → Metadata 哈希
        

### 2.2 `mapping.yml`（重命名 / 合并）

```yaml
- from: plc.flow_raw
  to:   plantA.flow_m3h

- from: plc.temp_f
  to:   plantA.temp_c
  expr: "(value - 32) * 5 / 9"        # 覆盖 variables.yml 中 scale

```

- 顺序重要；后者可覆盖前者。
    
- 若 `expr` 缺省 → 默认 `scale` 已在 variables 中处理。
    

### 2.3 `rules.yml`（实时逻辑）

```yaml
# 阈值告警
- when: data
  match: tag == "plantA.press_bar" and value > 4.0
  action:
    alert: "高压 >4 bar"
    severity: critical
    notify: ["sms:+861380...", "mqtt:alarm/high"]

# 窗口平均派生
- when: data
  match: tag == "plantA.flow_m3h"
  action:
    emit_data:
      tag: "plantA.flow_avg_1m"
      expr: "avg(value, window='1m', step='5s')"

```

---

## 3. 执行引擎架构

```pgsql
───────── FrameBus (input channel)
        │
        ▼
┌──────────────────────────────┐
│  TagHasher (fast path)       │
│    - Tag → MappingEntry*     │
└──────────────────────────────┘
        │ hit?
       yes
        ▼
┌──────────────────────────────┐
│  Mapper Step                 │
│    - Apply scale / expr      │
│    - Rename tag              │
└──────────────────────────────┘
        │
        ▼
┌──────────────────────────────┐
│  RuleEngine Step             │
│    - LuaJIT / Wasm eval      │
│    - Window primitives       │
│    - emit_data / emit_cmd /  │
│      alert / side-effect     │
└──────────────────────────────┘
        │
        ▼
───────── FrameBus (output channel)

```

### 3.1 关键技术点

|主题|实现|
|---|---|
|**高速哈希**|`ahash` + `once_cell::Lazy` 构建 `HashMap<&'static str, MappingEntry>`|
|**LuaJIT vs Wasm**|默认 LuaJIT (embedding ≈ <100 µs per call)  <br>启 `--feature wasm_rule` 开 Wasmtime|
|**窗口函数**|内置 `RingBuf<f64,N>` per Tag；调用如 `avg, max, min, sum, std`|
|**热加载零丢帧**|新解析结果放 `ArcSwap<HashMap>`；切换时读侧瞬间可见，旧引用自动 drop|
|**并发与背压**|单 tokio Task，内循环 `while let Ok(f)=rx.recv(){...}`；无锁数据结构保障 50 k fps|

---

## 4. 自定义窗口函数语法

|函数|例|说明|
|---|---|---|
|`avg`|`avg(value,'1m','5s')`|窗口 1 min 滚动，5 s 步长|
|`max`|`"max(value,'30s')"`|仅窗口，步长=采样周期|
|`deriv`|`deriv(value,'10s')`|(v_t - v_t-10s) / 10 s|
|`delay`|`delay(value,'1m')`|取 1 min 之前的值|

解析为 **AST**；窗口缓冲依赖 Tag+函数+window 长度哈希。

---

## 5. 错误与安全

|类别|处理方式|
|---|---|
|Lua 运行时错误|捕获 → 写 `edge_rule_error_total{rule_id}` Prom 指标；帧丢弃|
|表达式无限循环|`debug.sethook` 限 100k 指令|
|Wasm 越界 / OOM|Wasmtime `StoreLimits` CPU+Mem|
|用户脚本加载失败|回滚旧规则；发告警 Frame (`rule_load_error`)|

---

## 6. 性能基准

|场景|条件|P95 延迟|
|---|---|---|
|**纯映射**|10 k Tag，50 k fps|110 µs|
|**+Lua scale**|add 5 k expr|180 µs|
|**+窗口 avg**|1 k 1 min window|240 µs|
|**Lua panic 1 %**|异常捕捉|无帧丢失，P99 < 2 ms|

Testbed：Ryzen 5600，Rust 1.78，LuaJIT 2.1.0.

---

## 7. 开发与运维工具

|工具|用法|
|---|---|
|`edge map-test variables.yml mapping.yml`|Dry-run，输出新 Tag & Value|
|`edge rule-repl`|Lua REPL，内置 `avg()` mock，方便调试|
|Grafana Dashboard|面板 `Mapper latency`, `Rule error/sec`|
|CI Linter|`yamllint + jsonschema` 校验表格格式|

---

## 8. 路线图

|版本|计划|
|---|---|
|**0.1**|variables+mapping YAML，Lua `scale`，静态函数|
|**0.2**|rules.yml，LuaJIT engine，alert/emit_data|
|**0.3**|窗口函数 + 环形缓存|
|**0.4**|Optional Wasm expr，ONNX 插件接口|
|**1.0**|DSL 冻结，向后兼容保证 5 年|