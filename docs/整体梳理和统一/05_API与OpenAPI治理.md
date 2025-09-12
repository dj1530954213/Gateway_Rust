# API 与 OpenAPI 治理

## 设计规范

- 路径：`/api/v1/{资源}` + 资源风格
- 版本：URL 前缀分版
- 分页：`page`/`page_size` + 返回 `PaginationMeta` 与 `ResponseMeta`
- 错误：`ApiError { code, message, details }`
- 追踪：`request_id` + tracing 日志关联

## OpenAPI

- 使用 `utoipa` 在 `core/web-gw-api/src/openapi.rs` 汇总
- 在 `/docs/swagger-ui/` 暴露交互式文档
- 审核流程：PR 需更新文档与样例

## 兼容性

- 增量添加字段需保证反向兼容
- 引入破坏性改动需 bump 版本并提供迁移说明
