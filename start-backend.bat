@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"

echo [QuantPilot] Starting backend API on http://127.0.0.1:3000 ...
cargo run

if errorlevel 1 (
  echo.
  echo [QuantPilot] Backend failed to start.
  pause
)