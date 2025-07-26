@echo off
echo Setting environment variables...
set PROTOC=C:\tools\protoc\bin\protoc.exe
set LIBCLANG_PATH=C:\Program Files\LLVM\bin
set PATH=%PATH%;C:\tools\protoc\bin;C:\Program Files\LLVM\bin

echo PROTOC: %PROTOC%
echo LIBCLANG_PATH: %LIBCLANG_PATH%
echo.

echo Running cargo test...
cargo test
pause