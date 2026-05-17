@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"
set QUANTPILOT_DEV=true

echo QuantPilot Starting...
echo.

set EXE=quantpilot.exe
if not exist "%EXE%" (
    echo ERROR: %EXE% not found
    pause
    exit /b
)

echo Starting backend...
start "qp" "%EXE%"

echo Waiting 8 seconds for startup...
choice /t 8 /d y /n >nul 2>&1

echo Opening browser to http://127.0.0.1:3000
start http://127.0.0.1:3000

echo.
echo Server running at http://127.0.0.1:3000
echo.
echo Close this window to stop.
pause
taskkill /f /im quantpilot.exe >nul 2>&1
