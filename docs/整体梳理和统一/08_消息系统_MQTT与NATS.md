# 消息系统：MQTT 与 NATS

## MQTT（EMQX）

- 北向数据发布：主题命名规范 `gateway/data/{device}/{tag}`
- QoS 策略与遗嘱消息（选）
- 接入第三方系统的主通道

## NATS

- 控制/事件总线，支持分布式架构中多个边缘节点的编排
- 主题/Subject 设计与权限

## 安全与限流

- 认证/鉴权（Broker 侧）、主题授权、带宽与速率限制
