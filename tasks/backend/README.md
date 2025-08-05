# 后端开发任务

本文件夹存放Rust后端相关的开发任务。

## 任务类型

### API开发
- 新增REST API端点
- WebSocket功能实现
- OpenAPI文档生成

### 服务架构
- 微服务重构
- 依赖注入优化
- 错误处理标准化

### 数据处理
- Frame Bus集成
- 数据流处理
- 消息队列管理

### 驱动系统
- 静态驱动开发
- 动态驱动加载
- 协议桥接实现

### 性能优化
- 异步处理优化
- 内存使用优化
- 并发性能调优

## 相关模块

- `core/web-gw-api/` - HTTP API服务
- `core/frame-bus/` - 消息总线
- `core/driver-manager/` - 驱动管理
- `core/protocol-bridge/` - 协议桥接
- `infra/pg-repo/` - 数据库访问层