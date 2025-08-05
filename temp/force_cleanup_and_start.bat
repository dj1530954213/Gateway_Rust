@echo off
echo ============================================
echo     Gateway Rust 强制清理和启动脚本
echo ============================================
echo.

echo [1/4] 强制停止所有相关进程...
taskkill /f /im cargo.exe 2>nul
taskkill /f /im rustc.exe 2>nul  
taskkill /f /im web-gw-api.exe 2>nul
taskkill /f /im edge-gateway.exe 2>nul
taskkill /f /im python.exe 2>nul
taskkill /f /im node.exe 2>nul
wmic process where "name like '%%cargo%%' or name like '%%rustc%%'" delete 2>nul
echo 进程清理完成!

echo.
echo [2/4] 等待文件解锁...
timeout /t 3 /nobreak >nul

echo.
echo [3/4] 清理编译文件...
del /f /q "target\debug\web-gw-api.exe" 2>nul
del /f /q "target\debug\edge-gateway.exe" 2>nul
echo 文件清理完成!

echo.
echo [4/4] 清理编译缓存...
cargo clean -p web-gw-api 2>nul
echo 编译缓存清理完成!

echo.
echo ============================================
echo              清理完成！
echo ============================================
echo.
echo 现在您可以安全地运行以下命令启动后端：
echo.
echo   选项1 - Python API服务器（推荐）:
echo   cd temp
echo   python quick_test_api.py
echo.
echo   选项2 - Rust API服务器:
echo   cargo run -p web-gw-api --bin web-gw-api
echo.
echo   选项3 - 边缘网关服务:
echo   cargo run -p edge-gateway
echo.
echo ============================================
pause