$env:PROTOC = "C:\tools\protoc\bin\protoc.exe"
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:PATH = $env:PATH + ";C:\tools\protoc\bin;C:\Program Files\LLVM\bin"

Write-Host "Testing driver lifecycle timing fix..."
cargo test --package driver-manager --test lifecycle_tests test_driver_lifecycle_timing