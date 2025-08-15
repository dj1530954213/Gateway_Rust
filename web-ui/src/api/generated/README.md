# Gateway Rust - OpenAPI TypeScript 客户端

这个目录包含从 Gateway Rust 后端 OpenAPI 规范自动生成的 TypeScript 客户端代码。

## 生成客户端

### 前提条件

1. 确保 Gateway Rust 后端服务正在运行：

```bash
# 在项目根目录执行
cargo run -p web-gw-api
```

2. 确认 API 文档可访问：

```bash
curl http://localhost:8080/docs/openapi.json
```

### 生成命令

```bash
# 生成一次
npm run gen:api

# 监听变化并自动重新生成（开发模式）
npm run gen:api:watch
```

## 使用生成的客户端

### 基本用法

```typescript
import { GatewayRustApi } from '@/api/generated'

// 使用生成的服务
const devices = await GatewayRustApi.listDevices({
  page: 1,
  size: 20,
})

console.log('设备列表:', devices)
```

### 与现有 API 封装集成

生成的客户端会自动使用我们在 `@/api/http.ts` 中配置的 HTTP 客户端，包括：

- 自动添加认证 token
- 统一的错误处理
- 请求/响应拦截器
- 加载状态管理

### 类型安全

生成的客户端提供完整的 TypeScript 类型定义：

```typescript
import type { DeviceVO, DeviceCreateReq, TagDataType } from '@/api/generated'

// 类型安全的 API 调用
const createDevice = async (data: DeviceCreateReq): Promise<DeviceVO> => {
  return await GatewayRustApi.createDevice({ requestBody: data })
}
```

## 生成的文件结构

```
src/api/generated/
├── index.ts          # 统一导出
├── models/           # 类型定义
│   ├── DeviceVO.ts
│   ├── TagCreateReq.ts
│   └── ...
├── services/         # API 服务类
│   └── DefaultService.ts
└── core/            # 核心工具类
    ├── ApiError.ts
    ├── ApiResult.ts
    └── ...
```

## 开发工作流

1. **修改后端 API** - 在 Gateway Rust 中修改 API 接口
2. **重启服务** - 重启后端服务以更新 OpenAPI 规范
3. **重新生成** - 运行 `npm run gen:api` 更新前端客户端
4. **更新代码** - 使用新的类型和接口更新前端代码

## 注意事项

- **不要手动修改** 生成的文件，它们会在重新生成时被覆盖
- **版本控制** - 建议将生成的文件提交到 Git，确保团队同步
- **CI/CD** - 在构建流程中自动生成客户端，确保与后端同步

## 故障排除

### 生成失败

1. 检查后端服务是否运行
2. 确认 OpenAPI 端点可访问
3. 查看控制台错误信息

### 类型错误

1. 确保使用最新生成的客户端
2. 检查后端 OpenAPI 规范是否正确
3. 重新生成客户端

### 网络问题

1. 检查防火墙设置
2. 确认 API 地址配置正确
3. 验证 CORS 设置
