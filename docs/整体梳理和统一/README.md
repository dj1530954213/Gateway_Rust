# 整体梳理和统一（长期方案）

本目录用于承载“统一后端到 Actix（web-gw-api）”的长期方案与项目级重构计划，覆盖架构、技术路线、迁移步骤、验收标准与风险应对。

- 面向对象：后端开发、驱动与数据面开发、前端开发、DevOps、SRE
- 配套代码位置：
  - 后端（Actix 目标）：`core/web-gw-api/`
  - 现用（Warp 托底）：`core/rest-api/`
  - 网关主进程：`edge-gateway/`
  - 驱动与采集：`drivers/`, `core/driver-manager/`, `core/dynamic-driver/`
  - 连接与总线：`core/frame-bus/`, `connectors/mqtt5/`
  - 存储与仓库：`infra/pg-repo/`, `schema/migrations/`
  - 前端：`web-ui/`
  - 监控：`core/metrics-server/`, `docker/prometheus/`, `docker/grafana/`

## 文档索引

- 01_总体架构与原则.md
- 02_后端统一方案_Actix_web-gw-api.md
- 03_核心数据面_驱动与总线.md
- 04_前端对齐与接口规范.md
- 05_API与OpenAPI治理.md
- 06_WebSocket消息与订阅.md
- 07_数据模型_存储与迁移.md
- 08_消息系统_MQTT与NATS.md
- 09_可观测性_指标日志追踪.md
- 10_部署拓扑_本地_边缘_集中.md
- 11_迁移计划_里程碑与风险.md
- 12_开发流程与代码规范.md

## 目标端口与服务（目标态）

- Actix web-gw-api（主后端）：HTTP 8080
- Warp（迁移期备用）：HTTP 8180（迁移完成后淘汰或关闭）
- Prometheus 指标：9090（metrics-server），Actix 本身 9091（可选独立 Exporter）
- Web 管理界面：8090
- 外部组件：Postgres, InfluxDB, EMQX（MQTT Broker）, NATS, Grafana

## 验收基线（统一后）

- `GET /system/health` 200 且包含关键子系统健康状态
- `/api/v1` 下设备/点位/驱动/驱动配置/连接器等端点齐全且与前端契约一致
- WebSocket `/ws` 支持订阅/状态/批量/管理消息，前端实时视图稳定
- OpenAPI 文档 `/docs/swagger-ui/` 可用，契约生成与前端类型一致
- 端到端 E2E：页面无降级/告警，基础操作全链路通过
