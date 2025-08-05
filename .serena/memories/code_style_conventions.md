# 代码风格和约定

## Rust后端规范

### 代码风格
- 使用 `rustfmt` 自动格式化
- 使用 `clippy` 进行代码检查，必须无警告
- 所有public函数必须有文档注释
- 错误处理使用 `thiserror` + 自定义枚举

### 命名约定
- 模块名: `snake_case` (如 `driver_manager`)
- 结构体: `PascalCase` (如 `FrameBus`)
- 函数和变量: `snake_case` (如 `read_data`)
- 常量: `SCREAMING_SNAKE_CASE` (如 `DEFAULT_PORT`)

### 项目结构约定
- `core/`: 核心框架模块
- `drivers/`: 设备驱动
- `connectors/`: 云端连接器
- `infra/`: 基础设施层
- `tests/`: 测试代码

## 前端规范

### 代码风格
- 遵循 ESLint + TypeScript严格模式
- 使用 Prettier 格式化
- 组件使用 `PascalCase` 命名
- 文件名使用 `PascalCase.vue`

### Vue组件约定
- 组件必须使用 `<script setup lang="ts">`
- 使用 Composition API
- Props 必须定义TypeScript类型
- 事件使用 `defineEmits`

### 目录结构
```
src/
├── api/          # API接口层
├── components/   # 可复用组件
├── views/        # 页面视图
├── stores/       # Pinia状态管理
├── types/        # TypeScript类型定义
├── router/       # 路由配置
└── utils/        # 工具函数
```

## 通用规范

### 深度思考要求
遇到以下情况必须使用深度思考注释：
- 新增/修改 ≥30 行代码
- 影响多个模块/层
- 涉及并发、异步、性能优化
- 前后端接口变更

### 文档要求
- 每个模块必须有README说明
- API接口必须有OpenAPI文档
- 复杂算法必须有实现说明

### 测试要求
- 新增public函数必须添加单元测试
- 关键功能必须有集成测试
- 测试覆盖率应当合理

### 环境变量约定
使用 `WEBGW_` 前缀：
```bash
WEBGW_HTTP_ADDR=0.0.0.0:8080
WEBGW_PG_DSN=postgres://user:pass@localhost:5432/db
```