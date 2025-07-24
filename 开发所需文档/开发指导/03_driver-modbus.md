# 03 – Static Modbus‑TCP Driver 设计文档 (v1‑freeze)

## 1. 范围
功能码覆盖 0x01..0x06,0x0F,0x10,0x16；MVP‑0 实现 0x03/0x04/0x06 读/写。

## 2. 公共接口
实现 `Driver` trait：`init`, `connect`, `read_loop`, `write`.

## 3. 配置字段
unit_id, polling, max_regs_per_req≤125, retry, endian(big/little), enable_write。

## 4. 数据结构
`RegPoint`, `PollBatch`, `DrvCfg`, `DriverError`.

## 5. 批量算法
按功能码+连续地址聚合；max_regs_per_req 截断。

## 6. PDU 编解码
基于 tokio‑modbus；异常码映射 DriverError::Exception(u8)。

## 7. 缩放
Lua expr 预编译 (mlua) 对 `value`.

## 8. 重试
I/O timeout / Exception 重试 `retry` 次，否则标 Fault。

## 9. Prom 指标
`modbus_pdu_total`, `modbus_point_latency_ms`, `modbus_exception_total`, `modbus_reconnect_total`.

## 10. 测试
PDU Encode/Decode、Little‑endian Float、断线重连、异常 0x02。

## 11. 基准
解码 120 寄存器 ≤ 25 µs。

## 12. 兼容
cfg 字段只能追加；寄存器地址 u16，不扩大。

## 13. 里程碑
0.1 读 Holding/Input；0.2 写单寄存器；0.3 0x10 批写；1.0 完备码集。