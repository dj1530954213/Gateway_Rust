@echo off
echo ========================================
echo Gateway Rust 生产环境启动脚本
echo InfluxDB 3.2 + 完整监控栈
echo ========================================

echo.
echo 1. 检查Docker服务状态...
docker --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] Docker未安装或未启动
    pause
    exit /b 1
)

echo [OK] Docker服务正常

echo.
echo 2. 清理现有容器和镜像...
echo 停止所有容器...
for /f "tokens=*" %%i in ('docker ps -aq') do docker stop %%i >nul 2>&1

echo 删除所有容器...
for /f "tokens=*" %%i in ('docker ps -aq') do docker rm %%i >nul 2>&1

echo 删除所有镜像...
for /f "tokens=*" %%i in ('docker images -q') do docker rmi %%i -f >nul 2>&1

echo 清理volumes和网络...
docker volume prune -f >nul 2>&1
docker network prune -f >nul 2>&1
docker system prune -a --volumes -f >nul 2>&1

echo [OK] Docker环境清理完成

echo.
echo 3. 检查配置文件...
if not exist ".env.prod" (
    echo 创建默认环境配置文件...
    echo INFLUXDB_ADMIN_PASSWORD=influx_secure_2024>.env.prod
    echo GRAFANA_ADMIN_PASSWORD=grafana_secure_2024>>.env.prod
    echo GRAFANA_SECRET_KEY=gateway-rust-secret-key-2024>>.env.prod
    echo [提醒] 请编辑 .env.prod 文件设置安全密码
)

echo [OK] 配置文件检查完成

echo.
echo 4. 启动生产环境服务...
echo 使用InfluxDB 3.2-core镜像...
echo InfluxDB 3.2-core已存在，跳过拉取

echo 启动所有服务...
docker-compose -f docker-compose.prod.yml --env-file .env.prod up -d

echo.
echo 5. 等待服务启动...
timeout /t 10 /nobreak >nul

echo.
echo 6. 检查服务状态...
docker-compose -f docker-compose.prod.yml ps

echo.
echo ========================================
echo 启动完成！服务访问地址：
echo ========================================
echo Web管理界面:    http://localhost:8090
echo API接口:       http://localhost:8080
echo Grafana监控:   http://localhost:3000
echo InfluxDB 3.2:  http://localhost:8086
echo Prometheus:    http://localhost:9091
echo EMQX Dashboard: http://localhost:18083
echo ========================================

echo.
echo 默认账户信息（请尽快修改）：
echo Grafana:  admin / grafana_secure_2024
echo InfluxDB: admin / influx_secure_2024
echo EMQX:     admin / emqx_admin_2024

echo.
echo 按任意键查看服务日志，或关闭窗口...
pause >nul

echo.
echo 显示服务日志 (Ctrl+C 退出日志)...
docker-compose -f docker-compose.prod.yml logs -f