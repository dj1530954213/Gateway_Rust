1.1　深度思考触发条件
复杂度阈值 — 满足任一条件即属复杂场景，必须执行深度思考流程：

预计新增或修改 ≥ 30 行有效代码（不含空行、注释）。

影响两个及以上模块／层（例如 API 层 + Repo 层，或前端 Store + 组件）。

引入并发、异步、性能优化或非幂等逻辑。

涉及前端 ↔ 后端接口契约变更，或新增接口。

流程步骤

💭 Thoughts 注释块：在正式代码之前，编写块级注释，内容包括：

问题拆分与边界。

至少两套候选方案及取舍理由。

潜在异常场景与回滚策略。

影响到的接口（含请求 / 响应示例 JSON）。

Peer Review：如果是协同场景，Thoughts 必须先由人工确认后再进入编码；单人情况下，需自检并在注释后加“Self-Approved”。

编码实现：仅在上述步骤完成后方可开始真正编写逻辑代码和测试。

1.2　编译与测试要求
本地验证：完成修改后必须在本机依次执行：

bash
复制
编辑
cargo check && cargo test --workspace
npm run lint && npm run test:unit
无警告、无失败方视为通过。

禁止自动启动服务：上述命令 不得 启动 docker compose、Actix 二进制或 Vite。若改动影响配置文件，则在提交描述或变更说明中清楚写明新的启动命令或新增环境变量。

新增代码覆盖：

每个 public Rust 函数 / Actix Handler / Repo 方法、新增的 TS 函数 / Pinia Action 必须同步添加测试。

测试应覆盖 正常路径 + 至少一种异常或边界条件。

接口契约验证：如改动 JSON DTO 或前端调用方式，必须：

更新后端 openapi.json（自动生成即可）。

运行前端类型生成脚本（例如 openapi-typescript-codegen），确保 TS 类型同步。

在测试中断言新/变更字段实际存在并符合预期类型和值域。

1.3　文档同步与开发计划更新
docs/dev_plan.md：

在 “### 完成” 小节追加一行：- <功能简述> | <关联模块> | <日期>。

在 “### TODO” 小节勾掉对应任务并补充下一步里程碑。

接口文档：任何新增或变更的接口，必须在 docs/api/README.md 写明：路径、HTTP 方法、请求参数、返回示例、错误码。

配置变更清单：若新增或修改了 .yaml／.env／Compose 文件，需在同一文档的 “### Migration-Guide” 段落列出：

新增键名称、默认值。

修改后的启动命令（例如 WEBGW_HTTP_ADDR=0.0.0.0:9001 make dev）。

1.4　接口正确性强调
单一真源：所有前端调用参数与返回字段必须以后端 DTO 为唯一真源，不得在前端硬编码魔数或字符串常量。

双向验证：

后端：serde 层启用 deny_unknown_fields，拒绝前端多发字段。

前端：Axios 封装开启 responseSchemaValidate()（可用 zod / yup），在开发模式下校验服务端返回结构。

契约测试：对于关键接口（设备 CRUD、实时 WS、历史查询、报警推送），编写契约测试脚本（Rust 或 TS），先请求模拟数据，再断言 JSON Schema；测试失败必须先修复接口再继续其他开发。

### **段落 2：代码目录与模块责任清单（Back-End + Front-End）**

> 本段详细规定 *Gateway Rust* 的文件/包层次、命名约定与跨层调用路径，确保任何新增代码都能快速定位、互不侵入，并且前端-后端接口永远保持 **单一真源**。后续段落将在此基础上继续细化风格与配置。

---

#### 2-A 后端总体结构（Rust workspace）

```
core/                        # Rust workspace 根
├─ web-gw-api/               # Actix-Web HTTP/WS 入口
│   ├─ src/
│   │   ├─ main.rs           # 仅负责 CLI & Server 启动
│   │   ├─ bootstrap.rs      # 读取配置，构建 AppState（PgPool / Influx / DriverMgr / Bus）
│   │   ├─ config.rs         # ApiCfg 结构体 + Default impl
│   │   ├─ dto.rs            # ➡️ 所有外部可见 JSON 结构
│   │   ├─ error.rs          # ApiError → RFC 7807
│   │   └─ routes/           # 领域路由，每个模块一个文件
│   │       ├─ mod.rs
│   │       ├─ devices.rs
│   │       ├─ tags.rs
│   │       ├─ drivers.rs
│   │       ├─ history.rs
│   │       └─ alerts.rs
├─ core-domain/              # 纯领域模型 + 服务接口（❌ 不依赖 IO）
│   ├─ src/models.rs         # Device / Tag / Rule / AlertEvent …
│   └─ src/services.rs       # Trait 定义：DeviceService, AlertService …
├─ infra/                    # 具体基础设施实现
│   ├─ pg_repo/              # sqlx-based Repos：DeviceRepoPg, TagRepoPg …
│   ├─ influx_repo/          # Influx 查询封装
│   └─ frame_bus/            # 环形缓冲总线（已存在）
├─ driver-manager/           # 动态驱动装载与注册
├─ alert-engine/             # 后台报警评估
├─ workflow-worker/          # （可选）流程编排
└─ driver-sdk/               # 第三方驱动开发用宏 & Trait
```

##### 责任边界

| 层                  | 关键特点                        | 禁止事项                |
| ------------------ | --------------------------- | ------------------- |
| **web-gw-api**     | DTO ↔ Service 转换、HTTP/WS 绑定 | 直接 DB 查询、读取 `.yaml` |
| **core-domain**    | 业务规则、实体，零依赖 std+serde       | 调用 Actix、sqlx、tokio |
| **infra**          | 持久化 / 外部 IO                 | 业务判断                |
| **driver-manager** | 加载 `.so`、生命周期管理             | 写 DB、直接处理 HTTP      |
| **alert-engine**   | 定时评估、推事件                    | 暴露 HTTP（除 metrics）  |

---

#### 2-B 模块内部文件粒度

| 路径                      | 强制文件                                                  | 职责说明                                |
| ----------------------- | ----------------------------------------------------- | ----------------------------------- |
| `routes/<feature>.rs`   | `scope()`, 每 Handler 带 `#[utoipa::path]` 注释           | 路由+DTO解析+错误映射                       |
| `services/<feature>.rs` | `trait XxxService`, `struct XxxServiceImpl<R: Repo>`  | 业务用例；不得包含 `web::`                   |
| `pg_repo/<feature>.rs`  | `struct XxxRepo<'a>` 实现对应 Trait                       | **仅** SQL 操作                        |
| `dto.rs`                | `Req`/`Patch`/`Query`/`VO` 定义                         | 字段 `snake_case`, 带 `serde(default)` |
| `models.rs`             | `struct` + `enum`，均派生 `Serialize, Deserialize, Clone` | 持久化同名                               |

---

#### 2-C 前端目录结构（Vue 3）

```
web-ui/
├─ src/
│  ├─ api/                 # Axios 封装 + 由 openapi 生成的客户端
│  ├─ stores/              # Pinia，单文件一个 Store
│  ├─ pages/               # 路由级页面（与后端领域对齐）
│  │   ├─ Device.vue
│  │   ├─ Tag.vue
│  │   ├─ Driver.vue
│  │   ├─ History.vue
│  │   └─ AlertRule.vue
│  ├─ components/          # 可复用原子组件
│  ├─ assets/
│  └─ router.ts            # 路由声明
└─ vite.config.ts
```

##### 关键约束

| 层               | 约束                                                  | 违例示例                           |
| --------------- | --------------------------------------------------- | ------------------------------ |
| **api/**        | 只能调用 `openapi` 生成的 TS 方法；对错误统一拦截                    | 直接写 `axios.get('/api/v1/...')` |
| **stores/**     | 所有异步 Action 返回 `Promise<void>`；Mutation 写在 Action 内 | 在组件中调用 `axios`                 |
| **pages/**      | 只拼装组件 & 调 Action，不做业务计算                             | 在页面直接过滤列表                      |
| **components/** | 纯展示或小逻辑；禁止全局状态                                      | 在组件内部用 `useRouter()` 导航        |

---

#### 2-D 端到端调用路径示例（新增设备）

1. **Device.vue** 调 `deviceStore.create(req)`
2. **Pinia Store** 调 `api.createDevice()`（TS 客户端）
3. **routes/devices.rs::create** 解析 `DeviceCreateReq` → 调 `DeviceService::register`
4. **DeviceServiceImpl** 验证业务规则 → 调 `DeviceRepoPg.create()` → `driver_manager.register()`
5. **DeviceRepoPg** 执行 `INSERT ... RETURNING *`
6. 返回 `DeviceVO` → 前端写入列表，同时 WS 推通知（可选）

---

#### 2-E 命名约定

| 类型          | 结尾                   | 例子                 |
| ----------- | -------------------- | ------------------ |
| 创建请求        | `CreateReq`          | `DeviceCreateReq`  |
| 更新补丁        | `PatchReq`           | `TagPatchReq`      |
| 查询参数        | `Query`              | `HistoryQuery`     |
| 返回 JSON     | `VO` (View Object)   | `DeviceVO`         |
| 服务 Trait    | `XxxService`         | `AlertService`     |
| Repo Trait  | `XxxRepo`            | `DeviceRepo`       |
| Postgres 实现 | `XxxRepoPg`          | `TagRepoPg`        |
| 测试文件        | `<feature>_tests.rs` | `devices_tests.rs` |

---

#### 2-F 接口契约严检流程

1. **后端生成**：`cargo run -p web-gw-api -- gen-openapi docs/api/openapi.json`
2. **前端刷新**：`npx openapi --input docs/api/openapi.json --output src/api`
3. **类型对比**：TS 报错即代表契约不一致，必须修复后端 DTO 或前端调用。
4. **契约测试**：在 `contracts/` 目录编写 Rust 或 TS 测试，对关键接口进行 Schema 校验；任何失败阻断后续开发。

---

#### 2-G 配置文件层级

| 文件                    | 说明                                      |
| --------------------- | --------------------------------------- |
| `config/default.yaml` | 默认配置（不可引用私密）                            |
| `config/dev.yaml`     | 本地开发覆盖                                  |
| `config/prod.yaml`    | 生产覆盖                                    |
| `.env`                | 仅存 token/密码，一律通过环境变量注入                  |
| `docker-compose*.yml` | 不得写死端口，使用 `${WEBGW_HTTP_PORT:-8080}` 占位 |

---

#### 2-H 如何新增一个功能模块（模板）

1. **深度思考**：“💭 Thoughts” 注释 → 明确领域边界 + 接口草案。
2. **创建文件**：

    * 后端：`routes/<xxx>.rs`, `services/<xxx>.rs`, `pg_repo/<xxx>.rs`, DTO 更新。
    * 前端：`pages/<Xxx>.vue`, `stores/<xxx>.ts`, API 调用由 OpenAPI 生成。
3. **写单元测试**：Repo → mock PG；Service → fake Repo；前端组件 → Vitest + dom-testing。
4. **接口契约**：跑 `gen-openapi` + `openapi-typescript`，修正类型冲突。
5. **文档更新**：`docs/dev_plan.md` & `docs/api/README.md`。
6. **本地验证**：`cargo check && cargo test` & `npm run test:unit`。
7. **准备下一段任务**：在 TODO 列刷新剩余工作。

### **段落 3：Rust 代码风格与注释模板（粒度规范）**

> 本段给出 **函数级** 到 **语句级** 的编写与注释规则，保证代码可读、可维护、易调试，同时减少前端-后端接口误差。所有 Rust 源文件（含驱动、Worker、Repo）必须 100 % 遵循以下约定；CI 已启用 `rustfmt` + `clippy -D warnings`，但更重要的是语义与文档约束。

---

#### 3-A 文件与模块头部

```rust
//! devices.rs —— “设备” REST 路由与 Handler
//!
//! - scope(): `/api/v1/devices`
//! - 依赖注入：AppState<DeviceService>
//! - 读取 / 修改 DTO 定义于 dto.rs
//!
//! 更新历史：
//! - 2025-07-26  dujiang  初版
//! - 2025-08-01  bot     加入 pagination
```

* **`//!` doc-comment** 描述文件目的、路径、外部依赖、最近修改记录。
* 每个 `mod.rs` 顶部必须包含 “更新历史” 表；第一行简述职责。

---

#### 3-B 函数头注释模板（所有 `pub` 函数 / impl 方法）

````rust
/// 创建设备并注册至驱动
///
/// # Parameters
/// * `dto` – 设备创建请求体  
///
/// # Returns
/// * `Ok(Device)` – 创建成功  
/// * `Err(DeviceError::DuplicateName)` – 名称已存在  
///
/// # Side Effects
/// * 向 `driver_manager` 注册新协议实例  
///
/// # Errors
/// 详见 [`DeviceError`]。
///
/// # Example
/// ```rust
/// let dto = DeviceCreateReq { .. };
/// let dev = svc.register(dto).await?;
/// assert_eq!(dev.name, "PLC-01");
/// ```
pub async fn register(&self, dto: DeviceCreateReq) -> Result<Device, DeviceError> { … }
````

* **Sections**：`Parameters / Returns / Side Effects / Errors / Example`；缺省则写“None”。
* 示例代码必须 **编译可过**（`cargo test --doc` 自动验证）。

---

#### 3-C 错误处理模式（统一到 `anyhow` + 自定义枚举）

```rust
#[derive(thiserror::Error, Debug)]
pub enum DeviceError {
    #[error("name duplicated")]
    DuplicateName,
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("driver error: {0}")]
    Driver(#[from] DriverError),
}
```

* API 层转换：`impl From<DeviceError> for ApiError` → HTTP 409/500。
* **禁止** `unwrap()/expect()`；必须返回 `Err(_)` 或使用 `map_err`.

---

#### 3-D 日志与追踪

| 场景   | 宏             | 示例                                                                        | 备注                 |
| ---- | ------------- | ------------------------------------------------------------------------- | ------------------ |
| 入口路由 | `info!`       | `info!(target="api.devices", method=%req.method(), "create")`             | 只记录元数据             |
| 业务分支 | `debug!`      | `debug!(device_id=%id, "duplicate name")`                                 | 编译时 `release` 默认关闭 |
| 异常   | `error!`      | `error!(?err, "driver register failed")`                                  | 带 `?err` 展开        |
| 性能   | `trace!` span | `let _span = tracing::span!(Level::TRACE, "sql", query=QUERY).entered();` | 仅调优期启用             |

---

#### 3-E 并发与异步

1. 默认使用 **Tokio**；禁止混用 `async-std`。
2. **不可阻塞**：数据库操作使用 `sqlx::postgres` 异步；文件 IO 用 `tokio::fs`.
3. **Channel**：

   * `mpsc` 用于点对点单生产者；`broadcast` 用于 WS 多播；`frame_bus` 已抽象。
4. **Cancellation**：后台任务需监听 `shutdown_rx`.

---

#### 3-F 测试模板

```rust
/// Given-When-Then
#[tokio::test]
async fn create_device_duplicate_name() {
    // Given
    let repo = MockRepo::new().with_existing_name("PLC-01");
    let svc  = DeviceService::new(repo, FakeDriverMgr::default());

    // When
    let dto = DeviceCreateReq { name:"PLC-01".into(), ..Default::default() };
    let res = svc.register(dto).await;

    // Then
    assert!(matches!(res, Err(DeviceError::DuplicateName)));
}
```

* 函数名采用 **行为描述**；注释首行 `Given-When-Then`。
* Mock 使用 `mockall` 或手写 stub；禁止连接真实 DB。
* 测试数据构造放 `tests/fixtures.rs`。

---

#### 3-G DTO 与模型映射

```rust
impl From<DeviceRow> for DeviceVO { … }
impl TryFrom<DeviceCreateReq> for NewDevice { … }
```

* **单向映射**：DTO → Domain → DB Row；反向仅用于返回值。
* `TryFrom` 可返回验证错误 (`ValidationError`).

---

#### 3-H 跨层调用顺序 (示意)

```
[Route Handler] --> [Service Trait] --> [Repo Trait] --> [sqlx query]
                               |
                     (side-effect)
                               v
                     [driver-manager]
```

* **Service** 只能依赖 **Repo Trait**，不得直接依赖 PG 实现；便于单测注入 mock。
* Handler 只处理 HTTP 细节，不包含业务 if-else。

---

#### 3-I 示例目录条目（devices.rs）

```rust
pub fn scope() -> Scope {
    web::scope("/devices")
        .wrap(span_fn!(|req| tracing::info_span!("req", method=%req.method())))
        .route("", web::post().to(create))
        .route("", web::get().to(list))
        .route("/{id}", web::get().to(get))
        .route("/{id}", web::put().to(update))
        .route("/{id}", web::delete().to(delete))
}
```

* **Span 注入**：使用 `tracing_actix_web::TracingLogger` 或自定义 `span_fn!`.
* 返回统一 `Result<impl Responder, ApiError>`。

### **段落 4：前端组件 / Pinia Store 编写规范与注释模板**

> 本段为 *web-ui* 中 **全部** Vue 3／TypeScript 代码提供文件、函数、注释、测试的细粒度标准。遵守本段可将前端接口对接错误率降至最低，调试成本最小化。任何新组件、新 Store、新 API 包装请 100 % 依据以下约定执行。

---

#### 4-A 文件与目录粒度

| 目录                 | 子结构                                                                | 规则                                                |
| ------------------ | ------------------------------------------------------------------ | ------------------------------------------------- |
| `src/pages/`       | `<Feature>.vue`                                                    | 路由级页面，与后端领域一一对应；不得写业务循环或复杂表单逻辑，**只拼装子组件、调 Store** |
| `src/components/`  | `XxxCard.vue`, `XxxForm.vue`                                       | 复用单元或 Presentational 组件；不得直接调 Axios／Store         |
| `src/stores/`      | `<feature>.ts`                                                     | Pinia 单文件 Store；一个文件只管一类实体（device/tag/alert…）     |
| `src/api/`         | `client/`（自动生成 openapi TS）、`http.ts`（Axios 拦截）、`wrappers.ts`（二次封装） |                                                   |
| `src/composables/` | `useXxx.ts`                                                        | Composition API Hooks（如 useTelemetry）             |
| `tests/`           | `unit/`, `e2e/`                                                    | 单元测试用 Vitest + Vue Test Utils；E2E 用 Playwright    |

---

#### 4-B Vue 组件注释模板

```vue
<script setup lang="ts">
/**
 * DeviceTable —— 设备列表卡片
 *
 * 📝 Responsibilities:
 *  1. 接收 devices Prop 渲染表格
 *  2. 发射 "edit" / "delete" 事件
 *
 * 📦 Dependencies:
 *  - ElementPlus Table
 *
 * 🔄 Update Log:
 *  - 2025-07-27  dujiang  create
 */
defineProps<{ devices: Device[] }>()
const emit = defineEmits<{
  (e:'edit', id:string):void,
  (e:'delete', id:string):void
}>()
</script>
```

* **四段注释**：Responsibilities / Dependencies / Update Log；必要时加 “Known Issues”。
* `<script setup>` 顶部只放 **import/define**，业务逻辑放组合函数。

---

#### 4-C Pinia Store规范

```ts
/**
 * useDeviceStore —— 设备实体集中管理
 *
 * state:
 *  - list       通用列表（分页前后端统一）
 *  - loading    全局加载状态
 *
 * actions:
 *  - fetch(params)      拉取分页
 *  - create(dto)        新建设备并自动刷新列表
 *  - ...
 */
export const useDeviceStore = defineStore('device', {
  state: () => ({
    list: [] as DeviceVO[],
    loading: false,
  }),

  actions: {
    async fetch(p: DeviceQuery) {
      this.loading = true
      const { data } = await api.devicesList({ params: p })
      this.list = data.items
      this.loading = false
    },
    async create(d: DeviceCreateReq) {
      await api.devicesCreate({ data: d })
      await this.fetch({ page: 1, size: 20 })
    },
  },
})
```

**要点**

| 约束           | 说明                                                 |
| ------------ | -------------------------------------------------- |
| **命名**       | Store id = 文件名小写：`device`, `tag`, `alert`          |
| **返回值**      | 所有异步 Action 返回 `Promise<void>`；不得返回 Axios Response |
| **错误处理**     | 依赖 `api/http.ts` 全局拦截；Action 内仅捕获业务校验错误            |
| **Mutation** | 只在 Action 中修改 `state`；组件层只读取不写                     |
| **状态派生**     | 用 `computed` 或 `getter`；Getter 无副作用                |

---

#### 4-D Axios 封装 (`src/api/http.ts`)

```ts
import axios from 'axios'
import { ElMessage } from 'element-plus'
import { z } from 'zod'

export const http = axios.create({
  baseURL: import.meta.env.VITE_API_BASE,
  timeout: 8000,
  headers: { 'Content-Type': 'application/json' },
})

// 请求拦截：补充 Trace-Id
http.interceptors.request.use((cfg) => {
  cfg.headers['X-Trace-Id'] = crypto.randomUUID()
  return cfg
})

// 响应拦截：统一错误弹窗 & Schema 校验
http.interceptors.response.use(
  (res) => {
    const validator = responseSchemaMap[res.config.url!]
    if (validator) validator.parse(res.data)   // zod 校验
    return res
  },
  (err) => {
    ElMessage.error(err?.response?.data?.detail ?? err.message)
    return Promise.reject(err)
  },
)
```

* **SchemaMap**：编译时由脚本根据 `openapi.json` 生成。
* **Trace-Id** 用于后端日志关联。

---

#### 4-E ECharts 统一配置

```ts
// src/composables/useChart.ts
export function useChart() {
  onMounted(() => {
    echarts.init(el).setOption({
      animation: false,
      tooltip: { trigger: 'axis' },
      xAxis: { type: 'time' },
      yAxis: { type: 'value' },
      series: [],
    })
  })
}
```

* 组件仅注入 `series` 数据；样式、轴、动画统一在 Hook。

---

#### 4-F 接口同步步骤（手动执行，不自动 Git）

```bash
# 1. 生成最新 OpenAPI
cargo run -p web-gw-api -- gen-openapi docs/api/openapi.json

# 2. 生成 TS 客户端
npx openapi --input docs/api/openapi.json --output src/api/client --client axios

# 3. 手动更新 responseSchemaMap
npm run gen:schema
```

**⚠ 生成文件只写入本地，不做自动 Git 操作。**

---

#### 4-G 单元与组合测试

| 层          | 框架                      | 约束                     |
| ---------- | ----------------------- | ---------------------- |
| 组件         | Vue Test Utils + Vitest | 断言渲染 & 事件发射，mock Store |
| Store      | Vitest                  | mock `api`，断言 state 变化 |
| Composable | Vitest                  | 调用返回值；测试副作用            |
| 契约         | `zod` parse             | 断言后端响应符合 schema        |

**示例**

```ts
test('device store create then list', async () => {
  const store = useDeviceStore()
  vi.spyOn(api, 'devicesCreate').mockResolvedValue({ data: {} })
  vi.spyOn(api, 'devicesList').mockResolvedValue({ data: { items: [] } })

  await store.create({ name: 'PLC', protocol: 'ModbusTcp' })
  expect(api.devicesCreate).toHaveBeenCalled()
  expect(api.devicesList).toHaveBeenCalledWith({ params: { page:1,size:20 } })
})
```

---

#### 4-H 样式与 UI 规范

| 项    | 规则                                                 |
| ---- | -------------------------------------------------- |
| UI 库 | Element-Plus；主题色 `#1890ff`；禁止自定义暗黑模式               |
| 栅格   | 页面主栅格 `p-6`，卡片 `rounded-xl shadow`                 |
| 统一按钮 | `<PrimaryButton>` `<DangerButton>` 二次封装 Element 按钮 |
| 空状态  | 使用 `<ElEmpty>`；不得留白                                |

---

#### 4-I 前端错误处理大图

```
[Axios Error] -> interceptor -> ElMessage
                            -> if 401 future: redirect login
[API Validation] -> zod schema fail -> Console.warn + Message
[WebSocket] -> 'error' event -> store.events push({level:'CRIT'})
```

---

### **段落 5：配置文件与启动流程管理（多环境 & 手动控制）**

> 本段给出 *Gateway Rust* **从仓库克隆到本地运行** 的完整配置层级、启动顺序、热加载方法及变更通告流程。目标：
>
> 1. **单命令即可在本地→测试→生产启动**；
> 2. **绝不自动帮用户拉起服务**——所有组件只执行 *编译与测试*，最终启动始终由人工输入 `make run-*` 或 `docker compose up`；
> 3. 当配置发生变化时，Claude Code 必须在结果文本内**显式提示新的启动命令**，避免调试时间浪费。

---

#### 5-A 目录与文件层级

```
configs/
├─ default.yaml          # 全环境共用基线（不含密钥）
├─ dev.yaml              # 开发环境覆盖
├─ staging.yaml          # 预发布/测试
├─ prod.yaml             # 生产
├─ examples/             # 示例 .env 文件
│   ├─ dev.env
│   └─ prod.env
docker/
├─ compose.dev.yml       # 本地多容器
├─ compose.staging.yml
└─ compose.prod.yml
Makefile                 # ↗ 调度 compose 与 cargo/npm
```

---

#### 5-B YAML 合并顺序

1. **default.yaml** —— 提供不可或缺的键与合理默认值；**严禁**写入密码、Token、私网地址。
2. `<ENV>.yaml` —— 只覆写差异字段，例如端口、日志级别。
3. **环境变量** —— 最终覆盖层（优先级最高），通过 `.env` 或终端 `export`。
4. **CLI 参数** —— 仅用于一次性 Debug；日常启动不鼓励。

```text
优先级：CLI > ENV > {ENV}.yaml > default.yaml
```

---

#### 5-C 后端配置结构与映射

```rust
#[derive(Deserialize, Clone)]
pub struct ApiCfg {
    pub http_addr:      SocketAddr,               // WEBGW_HTTP_ADDR
    pub cors_allowed:   Vec<String>,              // WEBGW_CORS_ALLOWED
    pub pg_dsn:         String,                   // WEBGW_PG_DSN
    pub influx_url:     String,                   // WEBGW_INFLUX_URL
    pub bus_url:        String,                   // WEBGW_BUS_URL
    pub drivers_dir:    PathBuf,                  // WEBGW_DRIVERS_DIR
    pub log_level:      String,                   // WEBGW_LOG_LEVEL
}
```

* **环境变量映射**：自动通过 `envy` 读取前缀 `WEBGW_`；不存在的键回退到 YAML。
* **启动时打印**：`debug!("{:#?}", cfg.redacted())` —— `redacted()` 把密码字段替换为 `"***"`。

---

#### 5-D 示例 `default.yaml`

```yaml
http_addr: "0.0.0.0:8080"
cors_allowed: ["http://localhost:5173"]
pg_dsn: "postgres://postgres:postgres@localhost:5432/iot"
influx_url: "http://localhost:8086"
bus_url: "nats://localhost:4222"
drivers_dir: "./drivers"
log_level: "info"
```

##### `dev.yaml`

```yaml
cors_allowed:
  - "http://127.0.0.1:5173"
log_level: "debug"
```

##### `.env` (复制 examples/dev.env)

```
WEBGW_PG_DSN=postgres://postgres:postgres@127.0.0.1:5432/iot_dev
WEBGW_INFLUX_TOKEN=my-dev-token
```

---

#### 5-E `Makefile` 关键目标（手动启动）

```makefile
# ⬇⬇⬇ 仅编译与测试，不启动 ⬇⬇⬇
build-backend:
	cargo build -p web-gw-api

build-frontend:
	npm --prefix web-ui run build

test-all:
	cargo test --workspace
	npm --prefix web-ui run test:unit

# ⬇⬇⬇ 用户显式执行以下命令才会真正运行服务 ⬇⬇⬇
run-dev:              ## 本地开发环境
	docker compose -f docker/compose.dev.yml --env-file .env.dev up

run-staging:          ## 预发布
	docker compose -f docker/compose.staging.yml --env-file .env.stg up -d

run-prod:             ## 生产
	docker compose -f docker/compose.prod.yml --env-file .env.prod up -d
```

> **Claude Code 只允许调用 `make build-*`、`make test-*`**；任何 `run-*` 目标必须由人手输入。

---

#### 5-F `docker-compose.dev.yml` 关键片段

```yaml
services:
  postgres:
    image: postgres:15-alpine
    ports: ["5432:5432"]
    volumes: [ "pgdata:/var/lib/postgresql/data" ]
    environment:
      POSTGRES_PASSWORD: postgres

  influxdb:
    image: influxdb:2.7
    ports: ["8086:8086"]
    environment:
      - DOCKER_INFLUXDB_INIT_MODE=setup
      - DOCKER_INFLUXDB_INIT_USERNAME=${INFLUX_USER:-dev}
      - DOCKER_INFLUXDB_INIT_PASSWORD=${INFLUX_PASS:-devpass}
      - DOCKER_INFLUXDB_INIT_ORG=iot
      - DOCKER_INFLUXDB_INIT_BUCKET=telemetry
      - DOCKER_INFLUXDB_INIT_RETENTION=30d
```

* **占位变量**` ${...:-default}` 可被 `.env` 覆盖。
* 同一容器 **不得** 使用 `latest` tag；务必锁定版本。

---

#### 5-G 配置变更流程

| 步骤                     | 描述                                                                |      |                      |
| ---------------------- | ----------------------------------------------------------------- | ---- | -------------------- |
| 1️⃣ **修改 YAML / .env** | Claude Code 编辑文件后，在输出中插入 **📣 Config-Changed** 区段，列出：键名、默认值、为何要加。 |      |                      |
| 2️⃣ **更新 `config.rs`** | 添加字段 + `serde(default)`；升级 `redacted()`.                          |      |                      |
| 3️⃣ **编译 & 测试**        | `cargo check && cargo test`; 不执行 `run-*`.                         |      |                      |
| 4️⃣ **Migration Note** | 在 `docs/config_migration.md` 追加条目：\`<日期>                          | <描述> | <需重新 export 的 ENV>\` |

---

#### 5-H 运行序列图（手动）

```text
┌──────────┐
│ developer│
└────┬─────┘
     │ 1. make run-dev
     ▼
┌───────────────┐
│ docker-compose│  ←─── pulls images if absent
└────┬──────────┘
     │ 2. start Postgres, Influx, NATS
     │
     ▼
┌───────────────┐
│ Postgres      │  ←─── listens 5432
└───────────────┘
┌───────────────┐
│ InfluxDB      │  ←─── listens 8086
└───────────────┘
┌───────────────┐
│ NATS          │  ←─── listens 4222
└───────────────┘
```

> 后端 Actix 与前端 Vite 需用户在另一终端执行：
> `cargo run -p web-gw-api` 与 `npm run dev`。

---

#### 5-I 热加载策略

| 组件               | 方法                                             | 注意                     |
| ---------------- | ---------------------------------------------- | ---------------------- |
| **Rust 后端**      | 安装 `cargo-watch`; 由用户显式运行：`cargo watch -x run` | Claude 仅提示             |
| **Vue**          | Vite HMR (`npm run dev`)                       | 默认 5173                |
| **Alert-Engine** | `SIGHUP` 触发重新加载规则                              | CLI: `kill -HUP <pid>` |
| **Driver**       | 前端上传 `.so` → API 调 DriverManager.reload        | 立即生效，无需重启              |

---

#### 5-J 示范配置变更通告（模板）

```
📣 Config-Changed
新增: WEBGW_ALERT_SMTP_HOST (string) – 报警邮件 SMTP 服务器地址
默认: "localhost"
启动示例:
  WEBGW_ALERT_SMTP_HOST=smtp.example.com make run-dev
文档已更新: docs/config_migration.md 2025-08-02
```

Claude Code 必须在修改配置时按此模板输出。

---
### **段落 6：监控与日志收集部署（Prometheus + Grafana + Loki）**

> 本段为 *Gateway Rust* 提供**可观测性基座**：统一的性能指标、报警指标、日志检索和仪表盘。目标是让工程师能够在本地或服务器环境通过手动启动的监控栈快速定位性能瓶颈、接口错误或驱动异常。遵守本段后，所有核心组件均暴露 Prometheus 指标及结构化日志，Grafana 一键导入仪表盘，Loki 支撑全文检索。

---

#### 6-A 组件与版本锁定

| 组件             | 镜像 Tag                   | 端口     | 作用            |
| -------------- | ------------------------ | ------ | ------------- |
| **Prometheus** | `prom/prometheus:2.52.0` | `9090` | 指标采集与聚合       |
| **Grafana**    | `grafana/grafana:10.4.0` | `3000` | 可视化仪表盘        |
| **Loki**       | `grafana/loki:3.0.0`     | `3100` | 日志聚合          |
| **Promtail**   | `grafana/promtail:3.0.0` | –      | 将容器日志推送至 Loki |

*所有版本写死，不得使用 `latest`*。

---

#### 6-B 新增 Compose 文件 `docker/compose.monitor.yml`

```yaml
services:
  prometheus:
    image: prom/prometheus:2.52.0
    volumes:
      - ./monitor/prometheus.yml:/etc/prometheus/prometheus.yml
    ports: ["9090:9090"]

  grafana:
    image: grafana/grafana:10.4.0
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitor/grafana_dashboards:/var/lib/grafana/dashboards
    environment:
      - GF_DASHBOARDS_DEFAULT_HOME_DASHBOARD_PATH=/var/lib/grafana/dashboards/overview.json
    ports: ["3000:3000"]

  loki:
    image: grafana/loki:3.0.0
    command: -config.file=/etc/loki/local-config.yaml
    ports: ["3100:3100"]

  promtail:
    image: grafana/promtail:3.0.0
    volumes:
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - ./monitor/promtail-config.yml:/etc/promtail/config.yml
    command: -config.file=/etc/promtail/config.yml

volumes:
  grafana_data:
```

> 手动启动命令：
>
> ```bash
> docker compose -f docker/compose.monitor.yml up
> ```

---

#### 6-C Prometheus 抓取配置 `monitor/prometheus.yml`

```yaml
global:
  scrape_interval: 10s
scrape_configs:
  - job_name: api
    static_configs:
      - targets: ['host.docker.internal:8080']   # Actix API
  - job_name: alert_engine
    static_configs:
      - targets: ['host.docker.internal:9500']
  - job_name: postgres
    static_configs:
      - targets: ['postgres:9187']              # pg_exporter（可选）
  - job_name: influxdb
    metrics_path: /metrics
    static_configs:
      - targets: ['influxdb:8086']
  - job_name: docker
    static_configs:
      - targets: ['host.docker.internal:9323']   # cAdvisor / node-exporter
```

*☑ TODO MON-1* —— Claude Code 需在 API、Alert-Engine 二进制启动时自动开放 `/metrics`（见 6-E）。

---

#### 6-D 日志统一：`tracing` → Loki

1. **JSON 输出**：所有 Rust 服务设置

   ```rust
   tracing_subscriber::fmt()
       .json()
       .with_max_level(cfg.log_level.parse()?)
       .with_current_span(true)
       .with_span_events(FmtSpan::CLOSE)
       .init();
   ```
2. **日志路由**：Docker 通过 Promtail 读取容器 stdout；本地裸运行建议使用 `RUST_LOG=json` + file 路径，并调整 `promtail-config.yml` 文件中的 `__path__`.

---

#### 6-E 指标导出（Rust 端）

```rust
// Cargo.toml
metrics = "0.21"
metrics-exporter-prometheus = "0.14"

// main.rs
let recorder = PrometheusBuilder::new()
    .with_http_listener(cfg.metrics_addr)   // e.g., 0.0.0.0:8081
    .install_recorder()?;
```

| 模块                | 指标                                                              | 说明                          |
| ----------------- | --------------------------------------------------------------- | --------------------------- |
| **API**           | `http_requests_total`, `http_response_time_seconds` (Histogram) | label: method, path, status |
| **DriverManager** | `drivers_loaded_total`, `driver_reload_duration_seconds`        | label: protocol             |
| **Alert-Engine**  | `alert_evaluations_total`, `alert_trigger_total`                | label: result=ok/fired      |
| **DB**            | `sqlx_query_time_seconds`                                       | label: query\_hash          |

*☑ TODO MON-2* —— 在各 crate 引入 `#[instrument]` + `metrics::increment_counter! / histogram!`。

---

#### 6-F Grafana Dashboards

1. **overview\.json** —— 系统概览

   * API QPS / P95 / Error Rate
   * Alert 触发数
   * Influx write & query QPS
2. **driver.json** —— 驱动加载与热重载耗时
3. **alert.json** —— 每小时报警统计 + Top N 设备

放置于 `monitor/grafana_dashboards/`。Grafana 启动时自动导入。

*☑ TODO MON-3* —— Claude Code 需提交 `overview.json` 初版模板（可导入即显）。

---

#### 6-G 常见手工操作指南（写入 `docs/monitoring.md`）

| 目标           | 命令                                                                    |                              |
| ------------ | --------------------------------------------------------------------- | ---------------------------- |
| 查看 API 指标    | \`curl [http://localhost:8081/metrics](http://localhost:8081/metrics) | grep http\_requests\_total\` |
| Grafana 默认账户 | `admin / admin`；首次登录强制修改                                              |                              |
| Loki 查询示例    | \`{container="web-gw-api"}                                            | = "error"\`                  |
| 清理监控栈        | `docker compose -f docker/compose.monitor.yml down -v`                |                              |

*☑ TODO MON-4* —— 在文档新增章节并填充上述表格。

---

#### 6-H 变更通告模板

若 Claude Code 调整或新增指标路径、端口、dashboard，需附：

```
📣 Monitoring-Update
新增指标: drivers_loaded_total{protocol}
Grafana 面板: driver.json v0.2
手动刷新: docker compose -f docker/compose.monitor.yml up -d prometheus grafana
```

---

#### 6-I 综合 TODO 列表（段落 6 专属）

| 编号     | 内容                                                     |
| ------ | ------------------------------------------------------ |
| MON-1  | 在 `web-gw-api`, `alert-engine` 暴露 `/metrics` listener  |
| MON-2  | 插入 `metrics` 宏到 API 中间件、驱动重载、报警评估逻辑                    |
| MON-3  | 提交 Grafana `overview.json` 初版                          |
| MON-4  | 更新 `docs/monitoring.md` 手册                             |
| *手动操作* | 用户自行 `docker compose -f docker/compose.monitor.yml up` |





