# 08 – Protocol Bridge 设计文档 (v1‑draft)

## 1. 范围
* 将 FrameBus 数据 **回放成现场旧系统协议**  
* OPC‑UA Server、Modbus‑Slave、IEC‑104 Server …  
* 支持反向写 (旧系统 → CmdFrame)

## 2. Bridge Trait
```rust
#[async_trait]
pub trait Bridge: Send + Sync {
    async fn listen(&mut self, cfg:&Value) -> Result<()>;
}
```
Bridge 驱动放 `plugins/bridge/*.so`.

## 3. OPC‑UA Server Bridge
* open62541 C 库 FFI  
* AddressSpace 生成策略：Tag `plant.flow_m3h` → UA Node  
* 写回 → CmdFrame(origin="opcua")

## 4. Modbus‑Slave Bridge
* tokio‑modbus server mode  
* Tag ↔ reg map YAML (`bridge_tags.yml`)  
* Coil/HR 写回 → CmdFrame

## 5. Security
UA 支持 Anonymous / Username / X509；RateLimiter 10 cmd/s

## 6. 指标
bridge_srv_subs_total, bridge_write_req_total{result}

## 7. 测试
UA Browse & Write, Modbus Function 16 WriteMultiRegisters

## 8. 里程碑
0.1 OPC‑UA browse/read；0.2 Write; 0.3 Modbus‑Slave