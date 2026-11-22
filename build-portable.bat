@echo off
REM Builds Wasted and packages a portable zip. Needs Rust (rustup.rs) + MSVC C++ Build Tools. nasm optional.
cargo build --release || exit /b 1
if exist dist rmdir /s /q dist
mkdir dist\Wasted
copy target\release\wasted.exe dist\Wasted\ >nul
copy installer\portable.txt dist\Wasted\ >nul
copy LICENSE dist\Wasted\ >nul
powershell -NoProfile -Command "Compress-Archive -Path dist\Wasted -DestinationPath dist\Wasted-portable.zip -Force"
echo Done: dist\Wasted-portable.zip
