@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"

if not defined VITE_BACKEND_ORIGIN (
  set "VITE_BACKEND_ORIGIN=http://127.0.0.1:3000"
)

if not exist node_modules (
  echo [QuantPilot] node_modules not found, installing dependencies...
  call npm install
  if errorlevel 1 (
    echo.
    echo [QuantPilot] npm install failed.
    pause
    exit /b %errorlevel%
  )
)

echo [QuantPilot] Starting frontend dev server...
call npm run dev -- --host 0.0.0.0

if errorlevel 1 (
  echo.
  echo [QuantPilot] Frontend failed to start.
  pause
)
