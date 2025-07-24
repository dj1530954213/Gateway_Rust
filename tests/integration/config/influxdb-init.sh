#!/bin/bash

# InfluxDB 初始化脚本 - 集成测试专用

set -e

echo "Initializing InfluxDB for Gateway Integration Tests..."

# 等待InfluxDB启动
sleep 5

# 创建组织和用户（InfluxDB 2.x）
influx setup \
    --org gateway-test \
    --bucket gateway-test-bucket \
    --username admin \
    --password testpass123 \
    --token gateway-test-token \
    --force

# 创建测试用的bucket
influx bucket create \
    --name gateway-metrics \
    --org gateway-test \
    --retention 1h

influx bucket create \
    --name gateway-data \
    --org gateway-test \
    --retention 24h

echo "InfluxDB initialization completed!"