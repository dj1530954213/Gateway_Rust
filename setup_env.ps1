# 永久设置环境变量脚本
Write-Host "=== 设置开发环境变量 ==="

# 设置PROTOC环境变量
$protoc_path = "C:\tools\protoc\bin\protoc.exe"
[Environment]::SetEnvironmentVariable("PROTOC", $protoc_path, "User")
Write-Host "PROTOC 设置为: $protoc_path"

# 设置LIBCLANG_PATH环境变量  
$libclang_path = "C:\Program Files\LLVM\bin"
[Environment]::SetEnvironmentVariable("LIBCLANG_PATH", $libclang_path, "User")
Write-Host "LIBCLANG_PATH 设置为: $libclang_path"

# 更新PATH环境变量
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$protoc_bin = "C:\tools\protoc\bin"
$llvm_bin = "C:\Program Files\LLVM\bin"

if ($currentPath -notlike "*protoc*") {
    $newPath = $currentPath + ";$protoc_bin;$llvm_bin"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "PATH 已更新，添加了 protoc 和 LLVM 路径"
} else {
    Write-Host "PATH 已包含所需路径"
}

# 验证设置
Write-Host "`n=== 验证环境变量 ==="
Write-Host "PROTOC: " -NoNewline
Write-Host ([Environment]::GetEnvironmentVariable("PROTOC", "User"))
Write-Host "LIBCLANG_PATH: " -NoNewline  
Write-Host ([Environment]::GetEnvironmentVariable("LIBCLANG_PATH", "User"))

# 测试protoc可用性
if (Test-Path $protoc_path) {
    Write-Host "`n✅ protoc 可用:" -ForegroundColor Green
    & $protoc_path --version
} else {
    Write-Host "`n❌ protoc 不可用" -ForegroundColor Red
}

# 测试libclang可用性
if (Test-Path "$libclang_path\libclang.dll") {
    Write-Host "✅ libclang 可用" -ForegroundColor Green
} else {
    Write-Host "❌ libclang 不可用" -ForegroundColor Red
}

Write-Host "`n⚠️  重要提示: 请重启 PowerShell 或命令行窗口以使环境变量生效!" -ForegroundColor Yellow