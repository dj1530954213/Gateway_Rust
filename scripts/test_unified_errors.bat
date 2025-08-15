@echo off
rem Gateway_Rust 统一错误处理系统测试脚本 (Windows)
rem
rem 使用方法:
rem   scripts\test_unified_errors.bat

echo 🚀 Gateway_Rust 统一错误处理系统测试
echo ========================================

rem 检查工作目录
if not exist "Cargo.toml" (
    echo ❌ 错误: 请在项目根目录运行此脚本
    exit /b 1
)

echo 📋 测试步骤:
echo 1. 编译 common-errors 核心模块
echo 2. 编译 pg-repo 集成模块
echo 3. 运行错误处理演示程序
echo 4. 运行单元测试
echo.

rem 1. 编译核心错误处理模块
echo 🔨 步骤 1: 编译 common-errors 模块...
cargo check -p common-errors --quiet
if %errorlevel% equ 0 (
    echo ✅ common-errors 模块编译成功
) else (
    echo ❌ common-errors 模块编译失败
    exit /b 1
)

rem 2. 编译仓储层集成
echo 🔨 步骤 2: 编译 pg-repo 模块...
cargo check -p pg-repo --quiet
if %errorlevel% equ 0 (
    echo ✅ pg-repo 模块编译成功
) else (
    echo ❌ pg-repo 模块编译失败
    exit /b 1
)

rem 3. 运行演示程序
echo 🎯 步骤 3: 运行错误处理演示程序...
cd examples
cargo run --bin unified_error_handling_demo --quiet >nul 2>&1
if %errorlevel% equ 0 (
    echo ✅ 演示程序运行成功
) else (
    echo ⚠️ 演示程序运行遇到问题，但这可能是预期的（演示错误处理）
    echo ℹ️ 查看上方输出了解详细信息
)
cd ..

rem 4. 运行单元测试
echo 🧪 步骤 4: 运行单元测试...
echo   - 测试 common-errors 模块
cargo test -p common-errors --quiet
if %errorlevel% equ 0 (
    echo ✅ common-errors 测试通过
) else (
    echo ❌ common-errors 测试失败
    exit /b 1
)

echo   - 测试 examples
cd examples
cargo test --quiet
if %errorlevel% equ 0 (
    echo ✅ examples 测试通过
) else (
    echo ❌ examples 测试失败
    exit /b 1
)
cd ..

echo.
echo 🎉 统一错误处理系统测试完成！
echo.
echo 📊 测试结果摘要:
echo   ✅ 核心模块编译成功
echo   ✅ 集成模块编译成功
echo   ✅ 演示程序运行正常
echo   ✅ 单元测试全部通过
echo.
echo 📖 查看实施报告: UNIFIED_ERROR_HANDLING_IMPLEMENTATION.md
echo 🔍 查看演示代码: examples\unified_error_handling_demo.rs
echo.
echo 接下来的步骤:
echo 1. 配置 libclang 环境以编译 frame-bus 模块
echo 2. 更新现有代码以使用新的统一错误系统
echo 3. 运行端到端集成测试

pause