@echo off
echo ========================================
echo Gateway Rust 开发调试环境 - 第三方依赖启动
echo 端口范围: 50000+
echo ========================================

echo.
echo 1. 检查Docker环境...
docker --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] Docker未安装或未启动
    pause
    exit /b 1
)

echo [OK] Docker环境正常

echo.
echo 2. 清理可能冲突的容器...
echo 停止相关容器...
docker stop emqx prometheus grafana influxdb >nul 2>&1
docker rm emqx prometheus grafana influxdb >nul 2>&1

echo [OK] 容器清理完成

echo.
echo 3. 启动第三方依赖服务（仅Docker服务）...
echo 启动服务: EMQX, Prometheus, Grafana, InfluxDB 3.2-core

docker-compose up -d emqx prometheus grafana influxdb

echo.
echo 4. 等待服务启动...
timeout /t 15 /nobreak >nul

echo.
echo 5. 验证服务状态...
docker-compose ps

echo.
echo ========================================
echo 第三方依赖服务启动完成！
echo ========================================
echo 服务访问地址（50000+端口）：
echo - InfluxDB 3.2-core: http://localhost:50001 (admin/password123)
echo - Grafana监控:      http://localhost:50002 (admin/admin)
echo - Prometheus:       http://localhost:50003
echo - EMQX Dashboard:   http://localhost:50004 (admin/public)
echo - MQTT Broker:      mqtt://localhost:50005
echo ========================================

echo.
echo 接下来请手动启动：
echo 1. 后端服务（新终端）: 
echo    cargo run --bin edge-gateway -- --config config/dev-debug.yaml
echo.
echo 2. 前端服务（新终端）:
echo    cd web-ui
echo    npm run dev:debug
echo.
echo 按任意键查看服务日志...
pause >nul

echo.
echo 显示第三方服务日志...
docker-compose logs -f emqx prometheus grafana influxdb