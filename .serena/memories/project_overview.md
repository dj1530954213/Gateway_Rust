# 工控物联网边缘网关项目概述

## 项目目的
这是一个高性能、安全的工业物联网边缘网关，使用Rust语言实现，具备完整的生产级功能。实现了从基础MVP到高级分析引擎的全套特性，支持动态驱动加载、云端命令下发、实时数据分析、机器学习推理等企业级功能。

## 技术栈
### 后端
- **语言**: Rust 1.70+
- **Web框架**: Actix-Web 
- **数据库**: PostgreSQL (元数据) + InfluxDB (时序数据)
- **消息总线**: 自研Frame Bus + RocksDB WAL持久化
- **协议支持**: Modbus、OPC-UA、MQTT5
- **监控**: Prometheus + Grafana + Loki

### 前端  
- **框架**: Vue 3 + TypeScript
- **UI库**: Element Plus + Pinia
- **构建工具**: Vite
- **开发端口**: 50020

## 核心架构
**事件驱动微服务架构**，围绕中央**Frame Bus**构建：

```
[设备] → [驱动] → [Frame Bus] → [协议桥接] → [连接器] → [云端/MQTT]
                      ↓
              [Web API] ← [前端]
                      ↓
          [InfluxDB时序] + [PostgreSQL元数据]
```

## 关键组件
- **Frame Bus** (`core/frame-bus`): 中央消息总线，RocksDB WAL持久化
- **驱动系统** (`core/driver-manager`): 静态/动态驱动加载，热插拔支持  
- **Web API** (`core/web-gw-api`): Actix-Web REST API + WebSocket
- **协议桥接** (`core/protocol-bridge`): Modbus/OPC-UA协议转换
- **配置管理** (`core/config-manager`): 多层配置，热重载

## 服务端口
- REST API: 50013
- Web界面: 50014
- 监控: 50015
- 单独API服务: 50010
- 前端开发: 50020

## 项目状态
项目已完成100%，生产就绪状态。严禁使用任何模拟数据，必须连接真实PLC设备和工业协议。