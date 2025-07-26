# 模拟新的PowerShell会话
Write-Host "Testing in new PowerShell session..."
Write-Host "Current PROTOC: $env:PROTOC"
Write-Host "Current LIBCLANG_PATH: $env:LIBCLANG_PATH"

# 刷新环境变量
$env:PROTOC = [Environment]::GetEnvironmentVariable("PROTOC", "User")
$env:LIBCLANG_PATH = [Environment]::GetEnvironmentVariable("LIBCLANG_PATH", "User")
$env:PATH = [Environment]::GetEnvironmentVariable("PATH", "User")

Write-Host "Updated PROTOC: $env:PROTOC"
Write-Host "Updated LIBCLANG_PATH: $env:LIBCLANG_PATH"

# 测试protoc可用性
if (Test-Path $env:PROTOC) {
    Write-Host "protoc is available!"
    & $env:PROTOC --version
} else {
    Write-Host "protoc NOT found!"
}

# 运行cargo test
Write-Host "`nRunning cargo test..."
cargo test --package frame-bus