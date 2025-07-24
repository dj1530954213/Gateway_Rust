# 09 – Observability & OTA 设计文档 (v1‑draft)

## 1. 监控栈
* Prometheus exporter (`:9100/metrics`)  
* Loki JSON 日志 (`tracing`)  
* Grafana Dashboards JSON

## 2. 核心指标总表
| 子系统 | 关键 metric |
| EndpointKit | endpoint_acquire_latency_us, _pool_size |
| Modbus | modbus_pdu_total, _exception_total |
| FrameBus | framebus_ring_used, _drop_total |
| MQTT | mqtt_out_bytes_total, _rtt_ms |
| Config | cfg_revision_current, _apply_total |

## 3. 日志字段
`{ts, level, msg, subsys, driver_id?, endpoint?, err?}`

## 4. OTA 策略
* 双目录 `/opt/edge/A` `/opt/edge/B`  
* 新版拉到 **非当前** 目录 → health check → symlink switch → systemctl restart  
* 驱动 .so/.wasm 支持热替无需主程序重启

## 5. 签名验证
* `.bin` / `.so` / `.wasm` 文件附 `.sig` (ed25519)  
* 公钥白名单在 `config/tuf_pubkeys.json`

## 6. Dashboards
* `dashboards/framebus.json`  
* `dashboards/modbus.json`  
* `dashboards/mqtt.json`

## 7. 里程碑
0.1 Prom+Grafana+Loki docker‑compose; 0.2 OTA shell script; 1.0 OTA Rust daemon