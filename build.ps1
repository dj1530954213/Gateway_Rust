$env:PROTOC = "C:\tools\protoc\bin\protoc.exe"
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:PATH = $env:PATH + ";C:\tools\protoc\bin;C:\Program Files\LLVM\bin"

# 验证环境变量
Write-Host "PROTOC: $env:PROTOC"
Write-Host "LIBCLANG_PATH: $env:LIBCLANG_PATH"

# 验证protoc可用
if (Test-Path $env:PROTOC) {
    Write-Host "protoc found!"
    & $env:PROTOC --version
} else {
    Write-Host "protoc not found!"
}

# 验证libclang可用
if (Test-Path "$env:LIBCLANG_PATH\libclang.dll") {
    Write-Host "libclang found!"
} else {
    Write-Host "libclang not found!"
}

# 编译
Write-Host "Starting build..."
cargo build --package frame-bus