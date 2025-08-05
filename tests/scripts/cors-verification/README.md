# CORS问题修复与验证

## 概述

本目录包含了Gateway Rust项目中CORS（跨域资源共享）问题的修复过程和验证测试脚本。

## 问题描述

在前端Vue.js应用与后端Actix-Web API之间存在CORS错误，特别是axios XMLHttpRequest请求被浏览器阻止，错误信息为：
- `Access to XMLHttpRequest has been blocked by CORS policy`
- `No 'Access-Control-Allow-Origin' header is present on the requested resource`

## 解决方案

### 1. 后端CORS配置修复

在 `core/web-gw-api/src/bootstrap.rs` 中更新了CORS配置：

```rust
pub fn cors(&self) -> Cors {
    // 配置CORS以支持前端的所有请求头 (包括axios XMLHttpRequest)
    // 注意：supports_credentials() 不能与 allow_any_origin() 同时使用
    let cors = Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header()
        .expose_headers(vec![
            "x-request-id", 
            "content-length",
            "date",
            "server",
            "content-type"
        ])
        .max_age(3600);

    tracing::info!("CORS configured with explicit headers support");
    cors
}
```

### 2. 配置文件更新

更新了 `config/default.yaml` 和 `config/dev.yaml` 中的CORS设置：

```yaml
cors_allowed:
  - "*"
```

## 验证测试

### 测试文件说明

1. **test-cors-fix.js** - 基础的axios请求测试，验证主要API端点
2. **final-cors-verification.js** - 完整的CORS验证测试，模拟前端环境
3. **test-frontend-cors.html** - 浏览器端测试页面，可在浏览器中运行

### 测试结果

最终验证测试结果：
- ✅ 总计测试: 7个API端点
- ✅ 通过测试: 6个 (86%成功率)
- ✅ CORS头正确设置: `Access-Control-Allow-Origin: http://localhost:50021`
- ✅ 前端axios XMLHttpRequest请求正常工作

失败的1个测试是HTTP 500错误（业务逻辑问题，非CORS问题）

## 运行测试

```bash
# 确保后端API服务运行在50010端口
cargo run -p web-gw-api --bin web-gw-api

# 运行Node.js测试
node test-cors-fix.js
node final-cors-verification.js

# 在浏览器中打开HTML测试页面
# file:///path/to/test-frontend-cors.html
```

## 关键要点

1. **Actix-Web CORS配置**: 必须正确设置allow_any_origin()和相关headers
2. **不能同时使用**: `supports_credentials()` 和 `allow_any_origin()`
3. **服务重启**: 修改CORS配置后必须重启API服务才能生效
4. **测试验证**: 使用axios模拟前端环境，检查响应头中的CORS设置

## 修复日期

2025-08-05 - CORS问题完全解决

## 相关配置文件

- `core/web-gw-api/src/bootstrap.rs` - 主要CORS逻辑
- `config/default.yaml` - 基础配置
- `config/dev.yaml` - 开发环境配置
- `web-ui/.env.development` - 前端API配置