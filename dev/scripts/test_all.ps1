#!/usr/bin/env powershell

# 设置必要的环境变量
$env:PROTOC = "C:\tools\protoc\bin\protoc.exe"
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:PATH = $env:PATH + ";C:\tools\protoc\bin;C:\Program Files\LLVM\bin"

Write-Host "=== 运行完整项目测试 ===" -ForegroundColor Green
Write-Host "环境变量已设置：" -ForegroundColor Yellow
Write-Host "PROTOC = $env:PROTOC" -ForegroundColor Gray
Write-Host "LIBCLANG_PATH = $env:LIBCLANG_PATH" -ForegroundColor Gray
Write-Host ""

Write-Host "开始运行所有测试..." -ForegroundColor Cyan
cargo test

if ($LASTEXITCODE -eq 0) {
    Write-Host "=== 所有测试通过！ ===" -ForegroundColor Green
} else {
    Write-Host "=== 测试失败，请检查错误信息 ===" -ForegroundColor Red
    exit $LASTEXITCODE
}