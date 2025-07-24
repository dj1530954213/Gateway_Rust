Write-Host "=== 设置环境变量 ==="

# 设置PROTOC
[Environment]::SetEnvironmentVariable("PROTOC", "C:\tools\protoc\bin\protoc.exe", "User")
Write-Host "✓ PROTOC 已设置"

# 设置LIBCLANG_PATH  
[Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "User")
Write-Host "✓ LIBCLANG_PATH 已设置"

# 获取当前PATH
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$newPath = $currentPath + ";C:\tools\protoc\bin;C:\Program Files\LLVM\bin"
[Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
Write-Host "✓ PATH 已更新"

Write-Host "`n=== 验证 ==="
Write-Host "PROTOC:" ([Environment]::GetEnvironmentVariable("PROTOC", "User"))
Write-Host "LIBCLANG_PATH:" ([Environment]::GetEnvironmentVariable("LIBCLANG_PATH", "User"))

Write-Host "`n⚠️ 请关闭并重新打开 PowerShell 窗口，然后运行 'cargo test'"