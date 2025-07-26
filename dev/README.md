# 开发文件目录

本目录包含项目开发相关的文档和脚本文件。

## 📁 目录结构

### docs/
开发文档和指南：
- `FRONTEND_DEVELOPMENT_PLAN.md` - 前端开发计划
- `MENU_NAVIGATION_FIX.md` - 菜单导航修复文档
- `PROJECT_COMPLETION_SUMMARY.md` - 项目完成总结
- `quick_start.md` - 快速开始指南
- `startup_guide.md` - 启动指南
- `开发记录.md` - 开发记录

### scripts/
开发和测试脚本：

#### 构建脚本
- `build.ps1` - 构建脚本
- `build_all.ps1` - 全量构建脚本
- `cargo_test.bat` - Cargo测试批处理

#### 环境设置脚本
- `fix_env.ps1` - 环境修复脚本
- `set_env.ps1` - 环境设置脚本
- `setup_env.ps1` - 环境初始化脚本

#### 测试脚本
- `test.ps1` - 基础测试脚本
- `test_all.ps1` - 全量测试脚本
- `test_*_lifecycle.ps1` - 各种生命周期测试脚本
- `test_complete_*.ps1` - 完整性测试脚本
- `test_*_modules.ps1` - 模块测试脚本

#### 启动脚本
- `start-mock-api.bat` - 启动模拟API服务
- `final_test.ps1` - 最终测试脚本

## 💡 使用说明

1. **开发文档**：查看 `docs/` 目录下的相关文档
2. **构建项目**：运行 `scripts/build_all.ps1`
3. **运行测试**：运行 `scripts/test_all.ps1`
4. **环境设置**：首次使用时运行 `scripts/setup_env.ps1`

## 📋 注意事项

- 所有PowerShell脚本需要在Windows环境下运行
- 批处理文件(.bat)适用于Windows命令提示符
- 运行脚本前请确保已安装必要的依赖