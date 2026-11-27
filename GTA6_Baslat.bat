@echo off
setlocal
title clawz - GTA VI (RX 7600 8GB - %100 Saf Shader)
set "REPO=%~dp0"
set "PS1=%REPO%scripts\play_rx7600_8gb.ps1"
set "EXE=%REPO%target\release\victory.exe"
set "PROF=%~1"
if "%PROF%"=="" set "PROF=2"

if not exist "%EXE%" (
    echo [HATA] "%EXE%" bulunamadi!
    echo Once proje klasorunde "cargo build --release -p victory" komutuyla derleyin.
    pause
    exit /b 1
)

powershell -NoProfile -ExecutionPolicy Bypass -NoExit -File "%PS1%" -Profile %PROF%
endlocal
