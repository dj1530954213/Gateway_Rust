# 06 – Config‑Center & 热更新 设计文档 (v1‑freeze)

## 1. 三张表
endpoints.yml / drivers.yml / variables.yml — see § 样例。

## 2. JSON‑Schema
各表 schema+ driver extra schema merged at build.rs。

## 3. Snapshot
HashMaps + sha256 + rev。

## 4. Diff → Impact
Endpoint / Driver / Variables bool；算法 O(N)。

## 5. Apply 流水
Drain Endpoint → Restart Driver → Reload Mapper；失败自动 rollback。

## 6. etcd 布局
/edge/config/v1/endpoints/{id}; drivers/{id}; variables/vSN; revision。

## 7. CLI
lint / push / diff / rollback / watch。

## 8. Prom
cfg_revision_current, cfg_apply_total, cfg_invalid_total, cfg_rollback_total。

## 9. 安全
etcd TLS & token；变更需签名；驱动 .so ed25519 verify。

## 10. 测试
schema_reject, partial_driver_restart, endpoint_drain, rollback。

## 11. 里程碑
0.1 local YAML+inotify; 0.2 etcd backend; 0.3 Web UI; 1.0 multi‑tenant。