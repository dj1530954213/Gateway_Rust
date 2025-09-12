# 后端统一方案（Actix web-gw-api）

## 现状与问题

- Actix `core/web-gw-api/` 功能更全（REST+WS+OpenAPI+CORS），但存在编译错误未清理
- Warp `core/rest-api/` 可用但覆盖有限，无法作为长期统一面

## 目标

- 以 Actix 为唯一对外 API 实现，暴露：
  - `/system/*` 系列（健康、信息、指标）
  - `/api/v1/*`（设备/点位/驱动/驱动配置/连接器/报警/历史）
  - `/ws/*`（订阅、状态、批量、管理）
  - `/docs/*`（OpenAPI + Swagger UI）

## 迁移步骤

1. 修复编译错误（详见迁移计划）
   - `routes/driver_configs.rs` 受管实例字段与真实结构对齐
   - `routes/websocket.rs` 移除未定义扩展字段（已处理基础版），补足管理能力
   - `bootstrap.rs` CORS 配置与健康检查返回类型（已修复）
   - `routes/mod.rs` 暂禁 `database` 路由，后续补齐
   - `dto.rs` 调整 `ApiResponse::paginated` 针对 `Vec<T>` 实现，避免类型冲突（已处理）
2. 独立运行 Actix：绑定 `0.0.0.0:8080`，指标 `9091`
3. 将 Actix 嵌入 `edge-gateway` 主进程，Warp 移到 `8180` 并仅保留只读路由（或直接下线）
4. 更新 `docker-compose`、前端代理与 `.env`，统一到 8080
5. E2E 验收后，移除 Warp 依赖与代码路径

## 端口与指标

- HTTP：8080
- Metrics（Prometheus）：9091（Actix），9090（metrics-server 抓取整合）

## 安全与 CORS

- 严格 CORS 白名单：`CORS_ALLOWED_ORIGINS` 环境变量
- 认证：预留 API Key/Bearer 验证中间件，OpenAPI 中声明

## 代码参考

- Actix 路由根：`core/web-gw-api/src/routes/mod.rs`
- OpenAPI：`core/web-gw-api/src/openapi.rs`
- 启动装配：`core/web-gw-api/src/bootstrap.rs`
