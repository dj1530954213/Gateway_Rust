# 工程对齐与整改计划（Engineering Alignment Plan)

本文档统一收敛当前项目需要做与需要调整的全部事项，明确变更范围、决策基线、实施步骤、验收标准与风险控制，作为后续推进与评审的唯一事实依据（SSOT）。

更新时间：2025-09-09 19:01 +08:00
适用仓库：`/home/dj/CODE/Gateway_Rust`

---

## 1. 背景与目标

- 现状：仓库整体架构良好，但在端口/健康检查、Dockerfile、默认配置、文档与脚本、CI、OpenAPI 契约等方面存在不一致与阻断运行的问题。
- 目标：完成一轮“工程对齐”，确保开发/容器/生产三环境的一致性与可用性，建立稳定最小可运行路径（MVP），再逐步收敛技术栈与提升安全与可运维性。
- 成果：
  - 可本地启动（cargo 与 docker-compose 均可）
  - 可观测（/healthz 与 /metrics 正常）
  - 文档准确（README 与 docs 与脚本一致）
  - CI 基本通过（格式、构建、单测、基础集成）

---

## 2. 决策基线（Design Decisions）

- 端口统一（开发/容器/生产一致）：
  - REST API: 8080
  - Web 管理（如有单独进程）: 8090
  - Metrics/Prometheus: 9090
- 健康检查路径：`GET /healthz`（由主 REST 服务提供）
- 指标路径：`GET /metrics`（由 metrics 服务提供）
- 默认配置文件：`config/dev.yaml`（本地开发）；生产用 `config/default.yaml` 或通过 `--config` 指定。
- 配置文件后缀统一：`.yaml`（弃用 `.yml`）。
- HTTP 框架收敛：Axum 0.7（P1 执行，逐步移除 actix/warp/hyper 的直接使用）。
- OpenAPI 契约：统一由网关 REST 服务在 `http://localhost:8080/docs/openapi.json` 暴露（P1）。
- CI 策略：优先稳定最小集，将高负载与不稳定的步骤移至手动触发或夜间定时（P1）。

---

## 3. 分阶段实施计划

### P0（阻断项修复，确保可运行）

1) 统一端口 / 健康检查
- 现状：代码、Dockerfile、docker-compose、文档端口与路径不一致；健康检查路径错误。
- 变更：
  - 将 REST 监听端口统一为 8080，将 metrics 端口统一为 9090（若 Web 服务独立则为 8090）。
  - 健康检查使用 `GET http://127.0.0.1:8080/healthz`。
- 涉及文件：
  - `edge-gateway/src/main.rs`（服务绑定端口、健康检查路由）
  - `config/default.yaml`, `config/dev.yaml`, `config/dev-debug.yaml`（端口项）
  - `Dockerfile`（EXPOSE、HEALTHCHECK）
  - `docker-compose.yml`, `docker-compose.prod.yml`（端口映射与健康检查）
  - `README.md`, `dev/docs/*.md`（端口说明）
- 验收标准：
  - `cargo run -p edge-gateway -- --debug` 后，`curl -fsS http://127.0.0.1:8080/healthz` 返回 200；`curl -fsS http://127.0.0.1:9090/metrics` 可见指标。
  - `docker-compose up -d` 后，edge-gateway 容器状态 `healthy`。

2) 修复 Dockerfile
- 现状：基础镜像标签不可用、拷贝路径错误、健康检查与启动命令引用不存在文件、遗漏 `.yaml` 文件复制等。
- 变更：
  - 基础镜像改为稳定存在的 tag（示例：`rust:1.79-bullseye` 或项目要求的稳定版）。
  - 移除错误的 `COPY web/`；若不在该镜像内托管前端，则不构建/拷贝前端；需要时另立前端镜像或后续 P1 再整合。
  - 同时复制 `config/*.yaml` 与 `config/*.yml`（过渡期），并逐步迁移到 `.yaml`。
  - `EXPOSE 8080 8090 9090` 与服务一致。
  - `HEALTHCHECK CMD curl -fsS http://127.0.0.1:8080/healthz || exit 1`。
  - `CMD` 使用实际存在的配置文件（示例：`config/dev.yaml` 或通过环境变量指定）。
- 涉及文件：`Dockerfile`
- 验收标准：
  - `docker build` 成功；
  - `docker run` 后健康检查通过；
  - 日志显示 REST/metrics 端口按预期开启。

3) 修复编译错误与默认配置
- 现状：`warn!` 宏未导入导致编译失败；默认 CLI 指向 `config/gateway.yaml` 但文件不存在。
- 变更：
  - 在 `edge-gateway/src/main.rs` 顶部导入 `tracing::warn` 或使用 `tracing::warn!` 全限定调用。
  - 将默认配置调整为 `config/dev.yaml`，或在 README 明确先运行 `scripts/generate-config.sh` 产出统一命名的文件，再用 `--config` 指向。
- 涉及文件：`edge-gateway/src/main.rs`、`README.md`、`scripts/generate-config.sh`
- 验收标准：
  - `cargo check --workspace` 通过；`cargo run -p edge-gateway` 可启动；
  - 文档指引下的最小路径可复现启动。

4) 文档与脚本对齐
- 现状：README、开发文档、脚本输出名/端口/步骤不一致。
- 变更：
  - 统一文档中的端口说明为 8080/8090/9090；
  - `scripts/generate-config.sh` 默认输出 `.yaml` 后缀，并在 README 中明确使用方式；
  - 移除文档中不存在的服务或替换为当前可用组件；
  - `web-ui` 开发端口以 `package.json` 为准（默认 50020），文档据此更新。
- 涉及文件：`README.md`、`dev/docs/*.md`、`scripts/generate-config.sh`、`web-ui/README 或文档段落`
- 验收标准：
  - 按 README 步骤即可成功跑通本地与容器环境；
  - 无死链或不可用端口的描述。

---

### P1（一致性与可维护性提升）

1) 收敛 HTTP 框架到 Axum 0.7
- 变更：
  - 新功能均基于 Axum；
  - 渐进移除 actix/warp/hyper 的直接依赖与实现；
  - 提供统一中间件（日志、CORS、鉴权、错误处理、限流）。
- 验收：
  - 仅保留 Axum 相关依赖；核心 REST/健康检查/metrics 工作正常。

2) 打通 OpenAPI 契约与前端 SDK 生成
- 变更：
  - 后端统一对外 `GET /docs/openapi.json`；
  - `web-ui/scripts/generate-api.cjs` 指向 `http://localhost:8080/docs/openapi.json`；
  - 在 CI 中增加“后端起—生成 SDK—前端构建”验证链。
- 验收：
  - `npm run gen:api` 可在本地与 CI 成功生成；
  - 前端 TS SDK 调用对齐后端接口。

3) CI 精简与路径修复
- 变更：
  - 将容易不稳定或超时的步骤（`cargo bench`、E2E、大型扫描）移至手动或夜间任务；
  - 修复 `tests/e2e` 路径引用（仓库无此目录时移除该步骤或补齐用例）；
  - `sqlx migrate run` 指向 `schema/migrations/`；
  - 基线 CI 保留：格式化、clippy、单元/集成测试、基础安全扫描。
- 验收：
  - 主干 PR CI 稳定绿；
  - 定时任务产出安全/质量报告。

4) 配置加载策略统一
- 变更：
  - 后缀统一为 `.yaml`；
  - 明确 `config` crate 的合并顺序（`default.yaml` -> env -> `--config`）；
  - 明确环境变量覆盖策略（不依赖 YAML `${VAR}` 内插，使用 `config` 与 `env` 约定）。
- 验收：
  - 本地、容器、生产加载配置一致；
  - 文档清晰描述覆盖优先级。

---

### P2（安全/可运维/一致性自动化）

1) 安全基线
- 变更：
  - 默认凭据集中在 `.env.example`，文档强调必须修改；
  - Dev CORS 放宽、Prod CORS 白名单化；
  - 引入鉴权/限流/审计日志中间件（按需）。
- 验收：
  - 生产环境安全扫描通过；
  - 默认部署不暴露弱口令与 unrestricted CORS。

2) 可观测性与优雅停机
- 变更：
  - 健康探针 `/healthz` 与指标 `/metrics`；
  - 进程 SIGTERM 优雅关闭；
  - 暴露版本/构建信息端点（/version）。
- 验收：
  - k8s/compose 下健康检查与滚动更新表现良好；
  - 观测面板能看到关键指标。

3) 文档一致性自动化
- 变更：
  - 在 README 顶部放置“端口与服务一览”表；
  - 增加简单脚本/CI 检查，校验文档端口与配置文件/compose 是否一致（关键字符串比对）。
- 验收：
  - 文档长期与实现保持一致，漂移自动报警。

---

## 4. 里程碑与时间计划（建议）

- W1（截至 2025-09-12）：完成 P0 全部项，输出“可运行基线报告”。
- W2（2025-09-15 ~ 2025-09-19）：完成 P1，提交“契约与 CI 稳定报告”。
- W3（2025-09-22 ~ 2025-09-26）：推进 P2，大版本文档与安全基线完成。

> 注：如遇上游依赖或环境变更，计划将滚动调整。

---

## 5. 变更清单（按文件）

- `edge-gateway/src/main.rs`
  - 统一端口绑定（8080/9090），新增/修正 `/healthz`；
  - 修复 `warn!` 宏导入；
  - 默认配置路径指向 `config/dev.yaml`（或文档明确生成步骤）。

- `config/default.yaml`, `config/dev.yaml`, `config/dev-debug.yaml`
  - 端口项与格式统一为 `.yaml`；
  - 明确可被环境变量与 `--config` 覆盖。

- `Dockerfile`
  - 稳定基础镜像、复制 `.yaml/.yml`（过渡期）、EXPOSE/HEALTHCHECK/启动命令修复。

- `docker-compose.yml`, `docker-compose.prod.yml`
  - 端口映射、健康检查与服务依赖修正。

- `README.md`, `dev/docs/*`, `scripts/generate-config.sh`
  - 端口、步骤、一键最小路径对齐；
  - 生成脚本产物统一 `.yaml`。

- `web-ui/scripts/generate-api.cjs`（P1）
  - OpenAPI 地址统一为 `http://localhost:8080/docs/openapi.json`。

- `.github/workflows/*.yml`（P1）
  - 精简步骤、修复迁移路径与 E2E 引用。

---

## 6. 风险与回滚

- 端口统一可能影响现有集成：
  - 应对：提供端口映射兼容期（compose 中临时保留旧映射但标注弃用），并在发布说明中强调变更。
- Dockerfile 变更可能影响构建缓存与镜像大小：
  - 应对：先在预生产分支验证；保留上一版本镜像以便回滚。
- 框架收敛（P1）涉及较大重构：
  - 应对：先以适配层形式兼容，逐步迁移；保持接口不变。

---

## 7. 验收清单（Checklists）

- 本地（cargo）
  - [ ] `cargo check --workspace` 通过
  - [ ] `cargo run -p edge-gateway` 启动成功
  - [ ] `curl http://127.0.0.1:8080/healthz` 返回 200
  - [ ] `curl http://127.0.0.1:9090/metrics` 返回指标

- 容器（compose）
  - [ ] `docker-compose up -d` 成功
  - [ ] `docker ps` 中 edge-gateway 为 `healthy`
  - [ ] 访问 `http://localhost:8080/healthz` 与 `http://localhost:9090/metrics`

- 文档与脚本
  - [ ] README 步骤可直接复现
  - [ ] 端口与 compose/配置一致
  - [ ] 生成脚本产物为 `.yaml`

- CI（P1）
  - [ ] PR CI 稳定绿
  - [ ] 夜间任务产出质量/安全报告

---

## 8. 推进与沟通机制

- 合并策略：P0 优先单独 PR，确保随时可回滚；P1、P2 分批。
- 评审要点：仅评审计划内改动，严格限制“顺手改”。
- 发布说明：每阶段完结产出“执行报告 + 变更摘要 + 风险提示”。

---

（完）
