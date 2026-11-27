@echo off
setlocal
title GTA VI - RX 7600 8GB Baslatici (target\release\victory.exe)
set "REPO=%~dp0"
set "PS1=%REPO%scripts\play_rx7600_8gb.ps1"
set "EXE=%REPO%target\release\victory.exe"

if not exist "%EXE%" (
    echo [HATA] "%EXE%" bulunamadi!
    echo Once proje klasorunde "cargo build --release -p victory" komutuyla derleyin.
    pause
    exit /b 1
)

rem Korunan ayarlar + tam model yukleme + sessiz log
set "VICTORY_AMBIENT=constant:0.18"
set "VICTORY_RELAY=2"
set "VICTORY_CLAMP_WAITS=ON"
set "VICTORY_PORT_CBV=1"
set "VICTORY_PORT_DESCRIPTORS=1"
set "RUST_LOG=error"

powershell -NoProfile -ExecutionPolicy Bypass -File "%PS1%" %*
endlocal
