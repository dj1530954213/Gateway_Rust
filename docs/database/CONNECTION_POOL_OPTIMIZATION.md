# 数据库连接池优化指南

## 概述

本文档描述了 Gateway_Rust 项目的数据库连接池优化方案，包括配置参数、监控指标、健康检查机制和生产环境调优建议。

## 优化特性

### 1. 高级连接池配置
- ✅ 可配置的最大/最小连接数
- ✅ 连接获取超时控制
- ✅ 连接空闲超时管理
- ✅ 连接生命周期管理
- ✅ 连接健康检查
- ✅ 智能重试机制

### 2. 性能监控
- ✅ Prometheus 指标集成
- ✅ 连接池利用率监控
- ✅ 查询性能追踪
- ✅ 慢查询检测
- ✅ 连接超时统计

### 3. 故障恢复
- ✅ 自动重试和指数退避
- ✅ 连接池健康检查
- ✅ 优雅降级机制
- ✅ 连接泄漏防护

## 配置参数详解

### 基础连接池配置

```yaml
database_pool:
  # 最大连接数（生产环境推荐：50-100）
  max_connections: 50
  # 最小连接数（保持的最少连接，推荐：max_connections * 0.1-0.2）
  min_connections: 5
  # 获取连接超时时间（秒，推荐：5-10秒）
  acquire_timeout_secs: 10
  # 连接空闲超时时间（秒，推荐：300-600秒）
  idle_timeout_secs: 600
  # 连接最大生存时间（秒，推荐：1800-3600秒）
  max_lifetime_secs: 3600
  # 获取连接前进行健康检查
  test_before_acquire: true
  # 连接池健康检查间隔（秒）
  health_check_interval_secs: 30
  # 启用连接池监控指标
  enable_metrics: true
  # 慢查询阈值（毫秒）
  slow_query_threshold_ms: 1000
```

### 重试配置

```yaml
retry_config:
  # 最大重试次数（推荐：3-5次）
  max_retries: 3
  # 初始重试间隔（毫秒，推荐：50-200ms）
  initial_retry_delay_ms: 100
  # 重试间隔递增因子（推荐：1.5-2.0）
  backoff_multiplier: 2.0
  # 最大重试间隔（毫秒，推荐：2000-5000ms）
  max_retry_delay_ms: 5000
  # 启用抖动（避免惊群效应）
  enable_jitter: true
```

## 环境变量覆盖

可以通过以下环境变量覆盖配置文件中的设置：

```bash
# 数据库连接池配置
export WEBGW_DB_MAX_CONNECTIONS=100
export WEBGW_DB_MIN_CONNECTIONS=20
export WEBGW_DB_ACQUIRE_TIMEOUT_SECS=5
export WEBGW_DB_IDLE_TIMEOUT_SECS=300
export WEBGW_DB_ENABLE_METRICS=true

# 数据库 DSN
export DATABASE_URL="postgres://user:password@host:5432/database"
```

## 不同环境的推荐配置

### 开发环境（config/dev.yaml）

```yaml
database_pool:
  max_connections: 20        # 较少连接数
  min_connections: 2         # 最少连接数
  health_check_interval_secs: 60     # 较低频率检查
  slow_query_threshold_ms: 2000      # 放宽慢查询阈值
```

### 生产环境（config/prod.yaml）

```yaml
database_pool:
  max_connections: 100       # 高并发支持
  min_connections: 20        # 保持足够连接
  acquire_timeout_secs: 5    # 快速失败
  idle_timeout_secs: 300     # 较短空闲时间
  max_lifetime_secs: 1800    # 定期刷新连接
  health_check_interval_secs: 15     # 频繁健康检查
  slow_query_threshold_ms: 500       # 严格慢查询阈值
```

## 监控指标

### Prometheus 指标

```promql
# 连接池利用率
db_pool_active_connections / db_pool_max_connections * 100

# 平均查询时间
rate(db_query_duration_ms_sum[5m]) / rate(db_query_duration_ms_count[5m])

# 慢查询比例
rate(db_slow_queries[5m]) / rate(db_queries_total[5m]) * 100

# 连接获取成功率
rate(db_pool_connection_acquired[5m]) / (rate(db_pool_connection_acquired[5m]) + rate(db_pool_connection_acquire_failed[5m])) * 100
```

### 关键指标阈值

| 指标 | 正常范围 | 警告阈值 | 严重阈值 |
|------|---------|----------|----------|
| 连接池利用率 | < 70% | 70-85% | > 85% |
| 平均查询时间 | < 100ms | 100-500ms | > 500ms |
| 慢查询比例 | < 1% | 1-5% | > 5% |
| 连接获取成功率 | > 99% | 95-99% | < 95% |

## API 接口

### 连接池状态查询

```bash
# 获取连接池状态
curl http://localhost:50013/api/v1/database/pool/status

# 获取性能指标
curl http://localhost:50013/api/v1/database/metrics

# 执行健康检查
curl -X POST http://localhost:50013/api/v1/database/pool/health-check

# 获取配置信息
curl http://localhost:50013/api/v1/database/pool/config
```

### 响应示例

```json
{
  "stats": {
    "active_connections": 15,
    "idle_connections": 5,
    "total_connections": 20,
    "max_connections": 50,
    "acquire_count": 12500,
    "release_count": 12450,
    "acquire_time_avg_ms": 2.5,
    "utilization_percent": 40.0
  },
  "health": {
    "last_check": "2025-01-27T10:30:00Z",
    "is_healthy": true,
    "consecutive_failures": 0,
    "last_error": null
  },
  "config": {
    "max_connections": 50,
    "min_connections": 5,
    "acquire_timeout_secs": 10,
    "slow_query_threshold_ms": 1000
  }
}
```

## 性能调优建议

### 1. 连接数调优

**计算公式**：
```
max_connections = (CPU 核心数 × 2) + 磁盘数量
```

**实际建议**：
- 开发环境：10-20 个连接
- 测试环境：20-50 个连接
- 生产环境：50-100 个连接（根据并发量调整）

### 2. 超时配置

```yaml
# 快速失败策略（适合高并发）
acquire_timeout_secs: 5
idle_timeout_secs: 300
max_lifetime_secs: 1800

# 容错策略（适合批处理）
acquire_timeout_secs: 15
idle_timeout_secs: 900
max_lifetime_secs: 7200
```

### 3. 监控告警

```yaml
# Grafana 告警规则
groups:
  - name: database_pool
    rules:
    - alert: HighConnectionPoolUtilization
      expr: db_pool_active_connections / db_pool_max_connections > 0.85
      for: 2m
      labels:
        severity: warning
      annotations:
        summary: "数据库连接池利用率过高"
        
    - alert: SlowDatabaseQueries
      expr: rate(db_slow_queries[5m]) / rate(db_queries_total[5m]) > 0.05
      for: 1m
      labels:
        severity: critical
      annotations:
        summary: "慢查询比例过高"
```

## 故障排查

### 常见问题

1. **连接池耗尽**
   ```bash
   # 检查连接数配置
   curl http://localhost:50013/api/v1/database/pool/config
   
   # 查看当前连接状态
   curl http://localhost:50013/api/v1/database/pool/status
   ```

2. **连接超时**
   ```bash
   # 调整超时配置
   export WEBGW_DB_ACQUIRE_TIMEOUT_SECS=15
   
   # 检查数据库性能
   curl http://localhost:50013/api/v1/database/metrics
   ```

3. **慢查询过多**
   ```bash
   # 降低慢查询阈值进行调试
   export WEBGW_DB_SLOW_QUERY_THRESHOLD_MS=500
   
   # 查看具体的慢查询日志
   grep "Slow query detected" logs/app.log
   ```

### 日志分析

```bash
# 查看连接池相关日志
grep -E "(连接池|connection pool|database)" logs/app.log | tail -100

# 分析慢查询
grep "Slow query" logs/app.log | awk '{print $NF}' | sort | uniq -c | sort -nr

# 监控连接获取失败
grep "连接获取失败" logs/app.log | wc -l
```

## 测试验证

### 负载测试

```bash
# 使用 wrk 进行负载测试
wrk -t12 -c400 -d30s --timeout 10s http://localhost:50013/api/v1/devices

# 并发查询测试
for i in {1..100}; do
  curl -s http://localhost:50013/api/v1/devices > /dev/null &
done
wait

# 检查连接池状态
curl http://localhost:50013/api/v1/database/pool/status
```

### 故障模拟

```bash
# 模拟数据库连接中断
sudo iptables -A OUTPUT -p tcp --dport 5432 -j DROP

# 等待一段时间后恢复
sudo iptables -D OUTPUT -p tcp --dport 5432 -j DROP

# 检查自动恢复情况
curl http://localhost:50013/api/v1/database/pool/health-check
```

## 最佳实践

1. **连接池大小**：根据业务并发量和数据库性能设置，避免过大或过小
2. **超时配置**：平衡响应速度和容错能力，生产环境建议快速失败
3. **监控告警**：设置合理的告警阈值，及时发现和处理问题
4. **定期检查**：定期查看连接池状态和性能指标，进行调优
5. **故障演练**：定期进行故障模拟，验证恢复机制

## 升级指南

从现有系统升级到新的连接池管理器：

1. **配置迁移**：将现有的连接池配置迁移到新的配置结构
2. **监控集成**：集成 Prometheus 指标收集
3. **健康检查**：启用连接池健康检查服务
4. **逐步部署**：在测试环境验证后再部署到生产环境

## 相关文档

- [PostgreSQL 连接池最佳实践](https://wiki.postgresql.org/wiki/Number_Of_Database_Connections)
- [SQLx 连接池配置](https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html)
- [Prometheus 监控指南](../monitoring/prometheus.md)
- [Grafana 仪表板配置](../monitoring/grafana.md)