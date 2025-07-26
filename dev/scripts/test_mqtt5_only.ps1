$env:PROTOC = "C:\tools\protoc\bin\protoc.exe"
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:PATH = $env:PATH + ";C:\tools\protoc\bin;C:\Program Files\LLVM\bin"

Write-Host "Testing mqtt5 connector tests..."
cargo test --package mqtt5