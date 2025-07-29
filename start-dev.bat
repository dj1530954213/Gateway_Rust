@echo off
echo ========================================
echo Gateway Rust 开发环境启动脚本
echo InfluxDB 3.2 + 开发调试模式
echo ========================================

echo.
echo 1. 检查开发环境依赖...
docker --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] Docker未安装或未启动
    pause
    exit /b 1
)

cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] Rust/Cargo未安装或未配置PATH
    pause
    exit /b 1
)

node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] Node.js未安装或未配置PATH
    pause
    exit /b 1
)

echo [OK] 开发环境依赖检查完成

echo.
echo 2. 清理现有容器...
echo 停止相关容器...
docker stop emqx prometheus grafana influxdb edge-gateway >nul 2>&1
docker rm emqx prometheus grafana influxdb edge-gateway >nul 2>&1

echo 清理开发volumes...
docker volume rm gateway_rust_emqx-data gateway_rust_prometheus-data gateway_rust_grafana-data gateway_rust_influxdb-data >nul 2>&1

echo [OK] 容器清理完成

echo.
echo 3. 启动基础服务容器...
echo 使用InfluxDB 3.2-core镜像...
echo InfluxDB 3.2-core已存在，跳过拉取

echo 启动EMQX, Prometheus, Grafana, InfluxDB...
docker-compose up -d emqx prometheus grafana influxdb

echo 等待服务启动...
timeout /t 15 /nobreak >nul

echo.
echo 4. 编译Rust后端...
echo 检查代码编译...
cargo check
if %errorlevel% neq 0 (
    echo [警告] 代码编译有问题，但继续启动...
)

echo.
echo 5. 安装前端依赖...
cd web-ui
if not exist "node_modules" (
    echo 安装npm依赖...
    npm install
)
cd ..

echo.
echo ========================================
echo 开发环境基础服务启动完成！
echo ========================================
echo 服务访问地址：
echo EMQX Dashboard: http://localhost:20006 (admin/public)
echo Grafana监控:   http://localhost:20008 (admin/admin)
echo InfluxDB 3.2:  http://localhost:20012 (admin/password123)
echo Prometheus:    http://localhost:20007
echo ========================================

echo.
echo 接下来请手动启动：
echo 1. 后端服务: cargo run -p edge-gateway
echo 2. 前端服务: cd web-ui ^&^& npm run dev
echo.
echo 或者按任意键自动启动后端和前端服务...
pause >nul

echo.
echo 启动后端服务...
start "Gateway Backend" cmd /k "cargo run -p edge-gateway"

echo 等待后端启动...
timeout /t 5 /nobreak >nul

echo 启动前端开发服务器...
start "Gateway Frontend" cmd /k "cd web-ui && npm run dev"

echo.
echo 完整开发环境启动完成！
echo 前端开发地址: http://localhost:5173
echo 后端API地址:  http://localhost:8080 (如果成功启动)
echo.
pause