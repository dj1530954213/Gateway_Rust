# 核心模块依赖分析与优化建议

## 当前核心模块结构

### API服务层 (存在重复)
- **web-gw-api** (Actix-Web) - 主要网关API，包含数据库访问
- **rest-api** (Warp) - 通用REST API框架，包含认证和RBAC  
- **web-server** (Axum) - HTTP服务器，依赖于rest-api

### 监控指标层 (存在重复)
- **metrics-server** - Prometheus指标服务器
- **monitoring** - 系统监控和健康检查

### 驱动与协议层
- **driver-manager** - 驱动管理器 ✅
- **driver-sdk** - 驱动开发SDK ✅
- **dynamic-driver** - 动态驱动加载 ✅
- **protocol-bridge** - 协议桥接 ✅

### 核心通信层
- **frame-bus** - 中央消息总线 ✅
- **endpoint-kit** - 端点连接工具包 ✅

### 业务功能层
- **alert-engine** - 告警引擎 ✅
- **advanced-features** - 高级功能 ✅

## 问题识别

### 1. API服务重复
```
问题: 三个不同的HTTP框架并存
- web-gw-api: Actix-Web (主要使用)
- rest-api: Warp (认证相关)  
- web-server: Axum (包装器)

影响: 
- 维护成本高
- 依赖复杂  
- 功能重复
```

### 2. 监控功能重复
```
问题: 监控功能分散在两个模块
- metrics-server: 指标收集和暴露
- monitoring: 健康检查和诊断

影响:
- 功能重叠
- 配置复杂
- 资源浪费
```

### 3. 模块边界不清晰
```
问题: 一些模块职责模糊
- 某些功能可能归属不明确
- 依赖关系过于复杂
```

## 优化建议

### 阶段1: API服务整合 (中优先级)

#### 方案A: 保留web-gw-api为主要API (推荐)
```
行动:
1. 将rest-api的认证功能迁移到web-gw-api
2. 废弃rest-api和web-server模块
3. 统一使用Actix-Web框架

优点:
- 减少依赖复杂度
- 统一技术栈
- 简化部署
```

#### 方案B: 明确职责分工
```
行动:
1. web-gw-api: 业务API (设备、标签、数据)
2. rest-api: 系统API (认证、配置、管理)
3. web-server: 静态资源服务

优点:
- 职责清晰
- 可独立部署
- 技术栈多样化
```

### 阶段2: 监控功能整合 (高优先级)

```
建议行动:
1. 将metrics-server功能并入monitoring模块
2. 创建统一的监控服务
3. 提供统一的指标和健康检查接口

新模块结构:
monitoring/
├── src/
│   ├── metrics/     # Prometheus指标
│   ├── health/      # 健康检查
│   ├── alerts/      # 告警集成
│   └── diagnostics/ # 系统诊断
```

### 阶段3: 目录结构重组 (低优先级)

```
建议新结构:
core/
├── api/           # API服务层
│   └── web-gw-api/
├── communication/ # 通信层
│   ├── frame-bus/
│   └── endpoint-kit/
├── drivers/       # 驱动层
│   ├── driver-manager/
│   ├── driver-sdk/
│   └── dynamic-driver/
├── protocols/     # 协议层
│   └── protocol-bridge/
├── services/      # 业务服务层
│   ├── alert-engine/
│   ├── monitoring/    # 整合后的监控
│   └── advanced-features/
```

## 实施优先级

### 高优先级 ✅
1. **整合监控功能** - metrics-server + monitoring
   - 影响: 减少资源占用，简化配置
   - 风险: 低
   - 工作量: 中等

### 中优先级 ⚠️
2. **API服务整合** - 统一API框架
   - 影响: 简化技术栈，降低维护成本
   - 风险: 中等 (需要迁移认证逻辑)
   - 工作量: 大

### 低优先级 📋
3. **目录结构重组** - 按功能域分组
   - 影响: 提高代码可维护性
   - 风险: 低
   - 工作量: 大 (需要更新所有导入)

## 实施计划

### 第1周: 监控功能整合
- [ ] 分析metrics-server和monitoring的功能重叠
- [ ] 设计统一监控模块架构
- [ ] 实施代码合并和测试

### 第2-3周: API服务评估
- [ ] 分析rest-api的认证功能使用情况
- [ ] 评估迁移到web-gw-api的可行性
- [ ] 制定详细迁移计划

### 第4周及以后: 根据评估结果执行
- [ ] 执行选定的API整合方案
- [ ] 更新文档和测试
- [ ] 考虑目录重组的必要性

## 成功指标

1. **模块数量减少**: 从当前15个核心模块减少到12个以下
2. **依赖复杂度降低**: 减少循环依赖和不必要的依赖关系
3. **构建时间优化**: 减少并行编译数，提高构建效率
4. **维护成本降低**: 统一技术栈，减少学习成本

## 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 功能回归 | 中 | 高 | 完善测试覆盖，分阶段部署 |
| 依赖冲突 | 低 | 中 | 仔细分析依赖树，渐进式迁移 |
| 性能下降 | 低 | 中 | 性能测试，基准对比 |
| 开发延期 | 中 | 中 | 合理评估工作量，预留buffer |