@echo off
title Installing Boombox RX-505...
cd /d "%~dp0"
echo ========================================================
echo   Launching Boombox RX-505 Windows Installer...
echo ========================================================
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Installation encountered an issue.
    pause
)
