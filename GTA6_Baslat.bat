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

powershell -NoProfile -ExecutionPolicy Bypass -File "%PS1%" %*
endlocal
