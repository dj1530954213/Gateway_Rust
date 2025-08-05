# 推荐开发命令

## 环境设置 (Windows)
```powershell
$env:PROTOC = "C:\tools\protoc\bin\protoc.exe"
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:PATH = $env:PATH + ";C:\tools\protoc\bin;C:\Program Files\LLVM\bin"
```

## 后端开发命令

### 构建和测试
```bash
# 构建整个工作空间
cargo build --workspace

# 运行所有测试
cargo test --workspace

# 代码检查 (必须无警告)
cargo check && cargo test --workspace

# 格式化和lint检查
cargo fmt && cargo clippy
```

### 启动服务
```bash
# 启动主网关服务 (推荐)
cargo run -p edge-gateway
# 端口: REST API (50013), Web界面 (50014), 监控 (50015)

# 或单独运行API服务
cargo run -p web-gw-api  # 端口: 50010
```

## 前端开发命令

```bash
# 进入前端目录
cd web-ui

# 安装依赖
npm install

# 开发模式启动 (端口: 50020)
npm run dev

# 生产构建
npm run build

# 代码检查 (必须通过)
npm run lint && npm run type-check

# 单元测试
npm run test:unit
```

## 快速启动流程
1. **后端**: `cargo run -p edge-gateway`
2. **前端**: `cd web-ui && npm run dev`  
3. **访问**: http://localhost:50020 (前端) → http://localhost:50013 (API)

## 健康检查
```bash
curl http://localhost:50013/health
```

## Docker部署
```bash
# 开发环境
docker-compose up -d

# 生产环境  
docker-compose -f docker-compose.prod.yml up -d
```

## 系统命令 (Windows)
- **列出文件**: `dir` 或 `ls` (如果安装了Git Bash)
- **搜索文件**: `findstr` 或 `Select-String` (PowerShell)
- **文件查找**: `where` 或 `Get-ChildItem` (PowerShell)
- **进程管理**: `tasklist`, `taskkill`
- **网络检查**: `netstat`, `ping`, `telnet`