@echo off
echo ========================================
echo Gateway Rust 开发调试环境测试脚本
echo 端口50000+ | InfluxDB 3.2-core修复版
echo ========================================

echo.
echo 1. 清理旧环境...
docker stop emqx prometheus grafana influxdb 2>nul
docker rm emqx prometheus grafana influxdb 2>nul
docker volume rm gateway_rust_influxdb-data gateway_rust_influxdb-config gateway_rust_influxdb-plugins 2>nul

echo [OK] 环境清理完成

echo.
echo 2. 启动修复后的第三方服务...
echo 启动服务: EMQX, Prometheus, Grafana, InfluxDB 3.2-core (修复版)

docker-compose up -d emqx prometheus grafana influxdb

echo.
echo 3. 等待服务启动 (60秒)...
timeout /t 60 /nobreak >nul

echo.
echo 4. 检查服务状态...
docker-compose ps

echo.
echo 5. 测试服务连通性...

echo 测试 InfluxDB 3.2-core (端口50001)...
curl -s http://localhost:50001/health >nul 2>&1
if %errorlevel% equ 0 (
    echo [✓] InfluxDB: http://localhost:50001 - 连接成功
) else (
    echo [✗] InfluxDB: http://localhost:50001 - 连接失败
    echo 查看InfluxDB日志:
    docker logs influxdb --tail 20
)

echo 测试 Grafana (端口50002)...
curl -s http://localhost:50002 >nul 2>&1
if %errorlevel% equ 0 (
    echo [✓] Grafana: http://localhost:50002 - 连接成功
) else (
    echo [✗] Grafana: http://localhost:50002 - 连接失败
)

echo 测试 Prometheus (端口50003)...
curl -s http://localhost:50003 >nul 2>&1
if %errorlevel% equ 0 (
    echo [✓] Prometheus: http://localhost:50003 - 连接成功
) else (
    echo [✗] Prometheus: http://localhost:50003 - 连接失败
)

echo 测试 EMQX Dashboard (端口50004)...
curl -s http://localhost:50004 >nul 2>&1
if %errorlevel% equ 0 (
    echo [✓] EMQX Dashboard: http://localhost:50004 - 连接成功
) else (
    echo [✗] EMQX Dashboard: http://localhost:50004 - 连接失败
)

echo.
echo 6. 测试前端编译...
cd web-ui
echo 检查前端API导入是否修复...
npm run build 2>error.log
if %errorlevel% equ 0 (
    echo [✓] 前端编译成功 - API导入错误已修复
) else (
    echo [✗] 前端编译失败 - 查看错误:
    type error.log
)
cd ..

echo.
echo ========================================
echo 测试完成！
echo ========================================
echo 如果所有测试通过，可以继续启动：
echo 1. 后端: cargo run --bin edge-gateway -- --config config/dev-debug.yaml
echo 2. 前端: cd web-ui ^&^& npm run dev:debug
echo ========================================

pause