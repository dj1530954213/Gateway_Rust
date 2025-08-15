# Gateway_Rust 自动化测试环境配置指南

## 概述

本文档提供Gateway_Rust项目完整的自动化测试工具链配置，包括端到端测试、单元测试、集成测试和性能测试的环境设置。

## 🛠️ 工具链架构

```
测试工具链架构
├── 🎭 Playwright          # E2E测试框架
├── 🦀 Cargo Test          # Rust单元测试
├── 📊 Tarpaulin           # 代码覆盖率
├── 🗄️ PostgreSQL          # 测试数据库
├── 📈 InfluxDB            # 时序数据库
├── 📡 Modbus Simulator    # 模拟PLC设备
└── 🐋 Docker             # 容器化环境
```

## 📋 环境要求

### 系统要求
- **操作系统**: Windows 10/11, Linux, macOS
- **Node.js**: >= 18.0.0
- **Rust**: >= 1.70.0
- **Docker**: >= 20.10.0
- **内存**: >= 8GB (推荐16GB)
- **磁盘**: >= 5GB可用空间

### 网络端口分配
```yaml
端口分配表:
  50010: REST API 服务 (开发)
  50013: Edge Gateway API (生产)
  50014: Web 管理界面
  50015: 监控指标端点
  50020: Vue.js 前端开发
  5432:  PostgreSQL 数据库
  8086:  InfluxDB 数据库
  502:   ModbusTCP 测试服务
  9090:  Prometheus 监控
  3000:  Grafana 仪表板
```

## 🔧 安装配置步骤

### 1. 核心依赖安装

#### Windows 环境
```powershell
# 安装 Node.js 和 npm
winget install OpenJS.NodeJS

# 安装 Rust 工具链
winget install Rustlang.Rustup

# 安装 Docker Desktop
winget install Docker.DockerDesktop

# 设置环境变量
$env:PROTOC = "C:\tools\protoc\bin\protoc.exe"
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:PATH = $env:PATH + ";C:\tools\protoc\bin;C:\Program Files\LLVM\bin"

# 安装 Protocol Buffers
choco install protoc

# 安装 LLVM (用于代码覆盖率)
choco install llvm
```

#### Linux 环境
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y nodejs npm curl build-essential libssl-dev pkg-config
sudo apt install -y postgresql-client docker.io docker-compose
sudo apt install -y protobuf-compiler clang llvm

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 安装 InfluxDB CLI
wget -qO- https://repos.influxdata.com/influxdb.key | sudo apt-key add -
echo "deb https://repos.influxdata.com/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/influxdb.list
sudo apt update && sudo apt install influxdb2-cli
```

### 2. 项目依赖安装

```bash
# 克隆并进入项目目录
git clone <repository-url>
cd Gateway_Rust

# 安装 Rust 依赖
cargo build --workspace

# 安装前端依赖
cd web-ui
npm install
cd ..

# 安装测试工具
npm install

# 安装 Playwright 浏览器
npx playwright install
```

### 3. 测试专用工具安装

#### 代码覆盖率工具
```bash
# 安装 tarpaulin (代码覆盖率)
cargo install cargo-tarpaulin

# 验证安装
cargo tarpaulin --version
```

#### 性能测试工具
```bash
# 安装 criterion (基准测试)
cargo install cargo-criterion

# 安装 flamegraph (性能分析)
cargo install flamegraph
```

#### 数据库工具
```bash
# 安装 sqlx-cli (数据库迁移)
cargo install sqlx-cli --no-default-features --features rustls,postgres

# 验证安装
sqlx --version
```

## 🗄️ 数据库环境配置

### PostgreSQL 配置
```yaml
# docker-compose.test.yml
version: '3.8'
services:
  postgres-test:
    image: postgres:15
    environment:
      POSTGRES_DB: gateway_test
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
    ports:
      - "5432:5432"
    volumes:
      - postgres_test_data:/var/lib/postgresql/data
      - ./schema/migrations:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

  influxdb-test:
    image: influxdb:2.7
    environment:
      DOCKER_INFLUXDB_INIT_MODE: setup
      DOCKER_INFLUXDB_INIT_USERNAME: admin
      DOCKER_INFLUXDB_INIT_PASSWORD: password
      DOCKER_INFLUXDB_INIT_ORG: gateway
      DOCKER_INFLUXDB_INIT_BUCKET: gateway_test
    ports:
      - "8086:8086"
    volumes:
      - influxdb_test_data:/var/lib/influxdb2

volumes:
  postgres_test_data:
  influxdb_test_data:
```

### 数据库初始化脚本
```bash
#!/bin/bash
# scripts/init-test-db.sh

echo "🗄️ 初始化测试数据库..."

# 启动数据库服务
docker-compose -f docker-compose.test.yml up -d

# 等待数据库就绪
echo "⏳ 等待PostgreSQL启动..."
until docker-compose -f docker-compose.test.yml exec postgres-test pg_isready -U postgres; do
  sleep 2
done

# 运行数据库迁移
echo "📊 运行数据库迁移..."
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/gateway_test"
sqlx migrate run --source ./schema/migrations

# 验证 InfluxDB 连接
echo "📈 验证InfluxDB连接..."
until curl -f http://localhost:8086/health > /dev/null 2>&1; do
  echo "等待InfluxDB..."
  sleep 2
done

echo "✅ 测试数据库初始化完成"
```

## 🏭 ModbusTCP 测试服务配置

### Modbus 模拟器配置
```python
# tests/scripts/modbus-simulator.py
#!/usr/bin/env python3
"""
真实ModbusTCP服务器模拟器
运行在 127.0.0.1:502
"""

from pymodbus.server.sync import StartTcpServer
from pymodbus.device import ModbusDeviceIdentification
from pymodbus.datastore import ModbusSequentialDataBlock
from pymodbus.datastore import ModbusSlaveContext, ModbusServerContext
import logging
import threading
import time
import random

class ModbusTestServer:
    def __init__(self, host='127.0.0.1', port=502):
        self.host = host
        self.port = port
        self.server_thread = None
        self.running = False
        
    def setup_datastore(self):
        """设置Modbus数据存储"""
        # 初始化数据块
        store = ModbusSlaveContext(
            di=ModbusSequentialDataBlock(0, [0] * 100),  # 离散输入
            co=ModbusSequentialDataBlock(0, [0] * 100),  # 线圈
            hr=ModbusSequentialDataBlock(0, [0] * 100),  # 保持寄存器
            ir=ModbusSequentialDataBlock(0, [0] * 100)   # 输入寄存器
        )
        
        # 设置初始测试数据
        store.setValues(3, 0, [1234, 5678, 9012, 3456])  # 保持寄存器
        store.setValues(4, 0, [100, 200, 300, 400])       # 输入寄存器
        
        context = ModbusServerContext(slaves=store, single=True)
        return context
    
    def start_data_simulator(self, context):
        """启动数据模拟线程"""
        def simulate_data():
            while self.running:
                try:
                    # 模拟传感器数据变化
                    slave = context[0]
                    
                    # 温度传感器 (地址0-3)
                    temperatures = [random.randint(200, 300) for _ in range(4)]
                    slave.setValues(4, 0, temperatures)
                    
                    # 压力传感器 (地址10-13)  
                    pressures = [random.randint(1000, 2000) for _ in range(4)]
                    slave.setValues(4, 10, pressures)
                    
                    # 流量传感器 (地址20-23)
                    flows = [random.randint(50, 150) for _ in range(4)]
                    slave.setValues(4, 20, flows)
                    
                    time.sleep(1)  # 每秒更新一次
                except Exception as e:
                    logging.error(f"数据模拟错误: {e}")
                    
        thread = threading.Thread(target=simulate_data, daemon=True)
        thread.start()
    
    def start(self):
        """启动Modbus服务器"""
        if self.running:
            return
            
        print(f"🚀 启动ModbusTCP测试服务器 {self.host}:{self.port}")
        
        # 设置日志
        logging.basicConfig(level=logging.INFO)
        
        # 设置设备标识
        identity = ModbusDeviceIdentification()
        identity.VendorName = 'Gateway Rust Test'
        identity.ProductCode = 'MODBUS-TEST'
        identity.VendorUrl = 'https://github.com/gateway-rust'
        identity.ProductName = 'Modbus Test Server'
        identity.ModelName = 'Test Server'
        identity.MajorMinorRevision = '1.0'
        
        # 创建数据存储
        context = self.setup_datastore()
        
        # 启动数据模拟
        self.running = True
        self.start_data_simulator(context)
        
        # 启动服务器
        def run_server():
            StartTcpServer(
                context,
                identity=identity,
                address=(self.host, self.port)
            )
            
        self.server_thread = threading.Thread(target=run_server, daemon=True)
        self.server_thread.start()
        
        print(f"✅ ModbusTCP服务器运行中: {self.host}:{self.port}")
        print("📊 可用数据点:")
        print("  - 温度传感器: 地址0-3 (200-300)")
        print("  - 压力传感器: 地址10-13 (1000-2000)")  
        print("  - 流量传感器: 地址20-23 (50-150)")
        
    def stop(self):
        """停止服务器"""
        self.running = False
        print("🛑 ModbusTCP测试服务器已停止")

if __name__ == "__main__":
    server = ModbusTestServer()
    try:
        server.start()
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        server.stop()
```

## 🧪 测试环境变量配置

### 环境变量文件
```bash
# .env.test
# 测试环境配置

# 数据库配置
DATABASE_URL=postgres://postgres:postgres@localhost:5432/gateway_test
INFLUX_URL=http://localhost:8086
INFLUX_TOKEN=test-token-123456
INFLUX_ORG=gateway
INFLUX_BUCKET=gateway_test

# 服务端点配置
GATEWAY_API_URL=http://localhost:50013
WEB_UI_URL=http://localhost:50020
METRICS_URL=http://localhost:50015

# Modbus测试配置
MODBUS_HOST=127.0.0.1
MODBUS_PORT=502

# 测试配置
RUST_LOG=debug
RUST_BACKTRACE=1
TEST_TIMEOUT=30000
PARALLEL_TESTS=2

# Playwright配置
HEADLESS=false
SCREENSHOT_MODE=only-on-failure
VIDEO_MODE=retain-on-failure
```

### PowerShell 环境设置脚本
```powershell
# scripts/setup-test-env.ps1
param(
    [string]$Environment = "test"
)

Write-Host "🔧 设置Gateway Rust测试环境 ($Environment)" -ForegroundColor Green

# 设置环境变量
$env:DATABASE_URL = "postgres://postgres:postgres@localhost:5432/gateway_test"
$env:INFLUX_URL = "http://localhost:8086"
$env:GATEWAY_API_URL = "http://localhost:50013"
$env:WEB_UI_URL = "http://localhost:50020"
$env:MODBUS_HOST = "127.0.0.1"
$env:MODBUS_PORT = "502"
$env:RUST_LOG = "debug"
$env:RUST_BACKTRACE = "1"

# 检查必要的工具
Write-Host "🔍 检查工具可用性..." -ForegroundColor Yellow

$tools = @(
    @{Name="cargo"; Command="cargo --version"},
    @{Name="node"; Command="node --version"},
    @{Name="npm"; Command="npm --version"},
    @{Name="docker"; Command="docker --version"},
    @{Name="playwright"; Command="npx playwright --version"}
)

foreach ($tool in $tools) {
    try {
        $version = Invoke-Expression $tool.Command 2>$null
        Write-Host "✅ $($tool.Name): $version" -ForegroundColor Green
    } catch {
        Write-Host "❌ $($tool.Name): 未安装或不可用" -ForegroundColor Red
    }
}

# 检查端口可用性
Write-Host "🔌 检查端口可用性..." -ForegroundColor Yellow

$ports = @(5432, 8086, 502, 50013, 50020)
foreach ($port in $ports) {
    try {
        $connection = Test-NetConnection -ComputerName localhost -Port $port -WarningAction SilentlyContinue
        if ($connection.TcpTestSucceeded) {
            Write-Host "✅ 端口 $port: 可用" -ForegroundColor Green
        } else {
            Write-Host "⚠️ 端口 $port: 未使用" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "⚠️ 端口 $port: 检查失败" -ForegroundColor Yellow
    }
}

Write-Host "🎯 测试环境设置完成!" -ForegroundColor Green
Write-Host "💡 使用以下命令启动测试:" -ForegroundColor Cyan
Write-Host "   cargo test --workspace" -ForegroundColor White
Write-Host "   npm test" -ForegroundColor White
Write-Host "   cargo tarpaulin" -ForegroundColor White
```

## 🚀 测试执行配置

### 测试执行脚本
```bash
#!/bin/bash
# scripts/run-all-tests.sh

set -e

echo "🧪 Gateway Rust 全面测试执行"
echo "================================"

# 检查环境
source scripts/setup-test-env.sh

# 1. 启动测试服务
echo "🚀 启动测试服务..."
docker-compose -f docker-compose.test.yml up -d
python3 tests/scripts/modbus-simulator.py &
MODBUS_PID=$!

# 等待服务就绪
sleep 10

# 2. 运行Rust单元测试
echo "🦀 执行Rust单元测试..."
cargo test --workspace --verbose

# 3. 运行代码覆盖率测试
echo "📊 执行代码覆盖率分析..."
cargo tarpaulin --out Html --out Xml --output-dir coverage/

# 4. 启动应用服务
echo "🌐 启动应用服务..."
cargo run -p edge-gateway &
GATEWAY_PID=$!

cd web-ui
npm run dev &
FRONTEND_PID=$!
cd ..

# 等待服务启动
sleep 20

# 5. 运行E2E测试
echo "🎭 执行Playwright E2E测试..."
npx playwright test --reporter=html,junit,json-summary

# 6. 运行性能测试
echo "⚡执行性能基准测试..."
cargo bench

# 7. 生成测试报告
echo "📋 生成测试报告..."
node scripts/generate-test-report.js

# 清理
echo "🧹 清理测试环境..."
kill $MODBUS_PID $GATEWAY_PID $FRONTEND_PID 2>/dev/null || true
docker-compose -f docker-compose.test.yml down

echo "✅ 所有测试执行完成!"
echo "📄 查看报告: coverage/index.html"
echo "🎭 E2E报告: tests/e2e/reports/html/index.html"
```

### 测试报告生成器
```javascript
// scripts/generate-test-report.js
const fs = require('fs');
const path = require('path');

class TestReportGenerator {
  constructor() {
    this.reportData = {
      timestamp: new Date().toISOString(),
      rustTests: {},
      coverage: {},
      e2eTests: {},
      performance: {}
    };
  }

  async generateReport() {
    console.log('📊 生成综合测试报告...');

    // 收集Rust测试结果
    await this.collectRustTestResults();
    
    // 收集覆盖率数据
    await this.collectCoverageData();
    
    // 收集E2E测试结果
    await this.collectE2EResults();
    
    // 收集性能测试数据
    await this.collectPerformanceData();
    
    // 生成HTML报告
    await this.generateHtmlReport();
    
    // 生成JSON摘要
    await this.generateJsonSummary();
    
    console.log('✅ 测试报告生成完成');
  }

  async collectRustTestResults() {
    // 解析cargo test输出
    try {
      const testOutput = fs.readFileSync('tests/results/cargo-test-output.txt', 'utf8');
      const lines = testOutput.split('\n');
      
      let passed = 0, failed = 0, ignored = 0;
      
      lines.forEach(line => {
        if (line.includes('test result:')) {
          const match = line.match(/(\d+) passed; (\d+) failed; (\d+) ignored/);
          if (match) {
            passed = parseInt(match[1]);
            failed = parseInt(match[2]);
            ignored = parseInt(match[3]);
          }
        }
      });
      
      this.reportData.rustTests = { passed, failed, ignored, total: passed + failed + ignored };
    } catch (error) {
      console.warn('⚠️ 无法收集Rust测试结果:', error.message);
    }
  }

  async collectCoverageData() {
    // 解析tarpaulin覆盖率报告
    try {
      const coverageJson = JSON.parse(fs.readFileSync('coverage/tarpaulin-report.json', 'utf8'));
      
      this.reportData.coverage = {
        lines: coverageJson.files.reduce((acc, file) => {
          acc.covered += file.summary.lines.covered;
          acc.total += file.summary.lines.count;
          return acc;
        }, { covered: 0, total: 0 }),
        branches: coverageJson.files.reduce((acc, file) => {
          acc.covered += file.summary.branches.covered;
          acc.total += file.summary.branches.count;
          return acc;
        }, { covered: 0, total: 0 })
      };
      
      // 计算百分比
      this.reportData.coverage.linePercentage = 
        (this.reportData.coverage.lines.covered / this.reportData.coverage.lines.total * 100).toFixed(2);
      this.reportData.coverage.branchPercentage = 
        (this.reportData.coverage.branches.covered / this.reportData.coverage.branches.total * 100).toFixed(2);
        
    } catch (error) {
      console.warn('⚠️ 无法收集覆盖率数据:', error.message);
    }
  }

  async collectE2EResults() {
    // 解析Playwright测试结果
    try {
      const e2eResults = JSON.parse(fs.readFileSync('tests/e2e/reports/results.json', 'utf8'));
      
      this.reportData.e2eTests = {
        passed: e2eResults.suites.reduce((acc, suite) => 
          acc + suite.specs.filter(spec => spec.tests.every(test => test.status === 'passed')).length, 0),
        failed: e2eResults.suites.reduce((acc, suite) => 
          acc + suite.specs.filter(spec => spec.tests.some(test => test.status === 'failed')).length, 0),
        duration: e2eResults.stats.duration
      };
      
      this.reportData.e2eTests.total = this.reportData.e2eTests.passed + this.reportData.e2eTests.failed;
      
    } catch (error) {
      console.warn('⚠️ 无法收集E2E测试结果:', error.message);
    }
  }

  async generateHtmlReport() {
    const html = `
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Gateway Rust 测试报告</title>
    <style>
        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .header { text-align: center; margin-bottom: 40px; }
        .header h1 { color: #2c3e50; margin-bottom: 10px; }
        .timestamp { color: #7f8c8d; font-size: 14px; }
        .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin-bottom: 40px; }
        .metric-card { background: #ecf0f1; padding: 20px; border-radius: 8px; text-align: center; }
        .metric-card h3 { margin-top: 0; color: #34495e; }
        .metric-value { font-size: 2em; font-weight: bold; margin: 10px 0; }
        .passed { color: #27ae60; }
        .failed { color: #e74c3c; }
        .coverage { color: #3498db; }
        .progress-bar { background: #ecf0f1; height: 20px; border-radius: 10px; overflow: hidden; margin: 10px 0; }
        .progress-fill { height: 100%; background: linear-gradient(90deg, #27ae60, #2ecc71); transition: width 0.3s ease; }
        .details { margin-top: 30px; }
        .details table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        .details th, .details td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }
        .details th { background: #34495e; color: white; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 Gateway Rust 自动化测试报告</h1>
            <div class="timestamp">生成时间: ${this.reportData.timestamp}</div>
        </div>
        
        <div class="metrics">
            <div class="metric-card">
                <h3>🦀 Rust单元测试</h3>
                <div class="metric-value passed">${this.reportData.rustTests.passed || 0}</div>
                <div>通过 / ${this.reportData.rustTests.total || 0} 总计</div>
                ${this.reportData.rustTests.failed > 0 ? `<div class="failed">失败: ${this.reportData.rustTests.failed}</div>` : ''}
            </div>
            
            <div class="metric-card">
                <h3>📊 代码覆盖率</h3>
                <div class="metric-value coverage">${this.reportData.coverage.linePercentage || '0'}%</div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: ${this.reportData.coverage.linePercentage || 0}%"></div>
                </div>
                <div>行覆盖率</div>
            </div>
            
            <div class="metric-card">
                <h3>🎭 E2E测试</h3>
                <div class="metric-value passed">${this.reportData.e2eTests.passed || 0}</div>
                <div>通过 / ${this.reportData.e2eTests.total || 0} 总计</div>
                ${this.reportData.e2eTests.duration ? `<div>耗时: ${Math.round(this.reportData.e2eTests.duration/1000)}s</div>` : ''}
            </div>
        </div>
        
        <div class="details">
            <h2>📋 详细信息</h2>
            <table>
                <tr><th>测试类型</th><th>通过</th><th>失败</th><th>总计</th><th>覆盖率</th></tr>
                <tr>
                    <td>Rust单元测试</td>
                    <td class="passed">${this.reportData.rustTests.passed || 0}</td>
                    <td class="failed">${this.reportData.rustTests.failed || 0}</td>
                    <td>${this.reportData.rustTests.total || 0}</td>
                    <td>${this.reportData.coverage.linePercentage || 0}%</td>
                </tr>
                <tr>
                    <td>E2E集成测试</td>
                    <td class="passed">${this.reportData.e2eTests.passed || 0}</td>
                    <td class="failed">${this.reportData.e2eTests.failed || 0}</td>
                    <td>${this.reportData.e2eTests.total || 0}</td>
                    <td>N/A</td>
                </tr>
            </table>
        </div>
        
        <div class="details" style="margin-top: 30px;">
            <h2>🔗 相关链接</h2>
            <ul>
                <li><a href="./coverage/index.html" target="_blank">详细代码覆盖率报告</a></li>
                <li><a href="../e2e/reports/html/index.html" target="_blank">Playwright E2E测试报告</a></li>
                <li><a href="./junit.xml" target="_blank">JUnit格式测试报告</a></li>
            </ul>
        </div>
    </div>
</body>
</html>
    `;
    
    fs.writeFileSync('tests/results/comprehensive-report.html', html);
    console.log('📄 HTML报告: tests/results/comprehensive-report.html');
  }

  async generateJsonSummary() {
    const summary = {
      ...this.reportData,
      summary: {
        totalTests: (this.reportData.rustTests.total || 0) + (this.reportData.e2eTests.total || 0),
        totalPassed: (this.reportData.rustTests.passed || 0) + (this.reportData.e2eTests.passed || 0),
        totalFailed: (this.reportData.rustTests.failed || 0) + (this.reportData.e2eTests.failed || 0),
        overallSuccessRate: 0
      }
    };
    
    summary.summary.overallSuccessRate = 
      (summary.summary.totalPassed / summary.summary.totalTests * 100).toFixed(2);
    
    fs.writeFileSync('tests/results/test-summary.json', JSON.stringify(summary, null, 2));
    console.log('📄 JSON摘要: tests/results/test-summary.json');
  }
}

// 执行报告生成
const generator = new TestReportGenerator();
generator.generateReport().catch(console.error);
```

## 🔍 测试验证和故障排除

### 健康检查脚本
```bash
#!/bin/bash
# scripts/health-check.sh

echo "🔍 Gateway Rust 测试环境健康检查"
echo "================================"

# 检查服务可用性
services=(
    "PostgreSQL:localhost:5432"
    "InfluxDB:localhost:8086"
    "ModbusTCP:localhost:502"
    "Gateway API:localhost:50013"
    "Web UI:localhost:50020"
)

for service in "${services[@]}"; do
    name=$(echo $service | cut -d: -f1)
    host=$(echo $service | cut -d: -f2)
    port=$(echo $service | cut -d: -f3)
    
    if timeout 5 bash -c "</dev/tcp/$host/$port"; then
        echo "✅ $name: 正常"
    else
        echo "❌ $name: 不可用"
    fi
done

# 检查文件系统
echo ""
echo "📁 检查目录结构..."
dirs=("tests/results" "coverage" "screenshots" "tests/e2e/reports")
for dir in "${dirs[@]}"; do
    if [ -d "$dir" ]; then
        echo "✅ $dir: 存在"
    else
        echo "⚠️ $dir: 缺失"
        mkdir -p "$dir"
        echo "📁 已创建: $dir"
    fi
done

# 检查环境变量
echo ""
echo "🔧 检查环境变量..."
env_vars=("DATABASE_URL" "INFLUX_URL" "RUST_LOG")
for var in "${env_vars[@]}"; do
    if [ -n "${!var}" ]; then
        echo "✅ $var: 已设置"
    else
        echo "⚠️ $var: 未设置"
    fi
done

echo ""
echo "✅ 健康检查完成"
```

## 📚 使用说明

### 快速开始
1. **环境准备**:
   ```bash
   # Windows
   .\scripts\setup-test-env.ps1
   
   # Linux/macOS  
   ./scripts/setup-test-env.sh
   ```

2. **启动测试环境**:
   ```bash
   # 启动数据库和服务
   docker-compose -f docker-compose.test.yml up -d
   
   # 启动Modbus模拟器
   python tests/scripts/modbus-simulator.py
   ```

3. **执行测试**:
   ```bash
   # 全套测试
   ./scripts/run-all-tests.sh
   
   # 单独测试类型
   cargo test --workspace          # Rust单元测试
   cargo tarpaulin                 # 代码覆盖率
   npm test                        # E2E测试
   ```

### 常见问题排解

#### 问题1: 端口冲突
```bash
# 检查端口占用
netstat -an | grep :502
netstat -an | grep :5432

# 解决方案: 修改配置文件中的端口号
```

#### 问题2: 数据库连接失败
```bash
# 检查数据库状态
docker-compose -f docker-compose.test.yml ps

# 重启数据库
docker-compose -f docker-compose.test.yml restart postgres-test
```

#### 问题3: Playwright浏览器问题
```bash
# 重新安装浏览器
npx playwright install --force

# 检查浏览器
npx playwright install-deps
```

## 🎯 最佳实践

### 测试数据管理
- 使用真实的传感器数据模式
- 避免硬编码的测试数据  
- 实现数据清理和重置机制

### 性能优化
- 并行执行非冲突的测试
- 使用测试数据缓存
- 优化数据库查询性能

### 持续集成
- 设置测试覆盖率门禁(>75%)
- 实现失败测试的自动重试
- 生成详细的测试报告

## 📞 技术支持

如遇问题，请查看：
1. **日志文件**: `tests/results/test.log`
2. **错误截图**: `screenshots/`
3. **测试报告**: `tests/results/comprehensive-report.html`

联系信息：
- GitHub Issues: 项目仓库
- 文档wiki: 测试相关说明

---

*本配置文档确保Gateway_Rust项目具备生产级的自动化测试能力，支持全方位的质量保证。*