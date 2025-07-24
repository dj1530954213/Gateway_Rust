Write-Host "Setting environment variables..."

[Environment]::SetEnvironmentVariable("PROTOC", "C:\tools\protoc\bin\protoc.exe", "User")
[Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "User")

$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$newPath = $currentPath + ";C:\tools\protoc\bin;C:\Program Files\LLVM\bin"
[Environment]::SetEnvironmentVariable("PATH", $newPath, "User")

Write-Host "Environment variables set successfully!"
Write-Host "PROTOC: C:\tools\protoc\bin\protoc.exe"
Write-Host "LIBCLANG_PATH: C:\Program Files\LLVM\bin"
Write-Host ""
Write-Host "Please restart PowerShell and run: cargo test"