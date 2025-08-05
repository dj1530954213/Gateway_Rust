@echo off
echo ========================================
echo 启动 Gateway Rust 服务 - 新端口配置
echo ========================================
echo.
echo 后端端口: 18080
echo 前端端口: 58020
echo.
echo 请在两个不同的终端窗口中分别运行：
echo.
echo 1. 启动后端服务：
echo    cargo run -p edge-gateway
echo.
echo 2. 启动前端服务：
echo    cd web-ui
echo    npm run dev
echo.
echo 然后访问: http://localhost:58020
echo.
pause