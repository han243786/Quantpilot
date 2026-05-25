@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"
chcp 65001 >nul
set QUANTPILOT_DEV=true
echo ============================================
echo   QuantPilot Desktop v4.7.0
echo   QUANTPILOT_DEV = true
echo ============================================
taskkill /f /im quantpilot.exe 2>nul
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :5173 ^| findstr LISTENING 2^>nul') do taskkill /f /pid %%a 2>nul
taskkill /f /im quantpilot-tauri.exe 2>nul
echo.
echo ============================================
echo   Step 1/3: Building backend...
echo ============================================
cargo build --bin quantpilot
if !ERRORLEVEL! NEQ 0 (
    echo [ERROR] Backend build failed!
    pause
    exit /b 1
)
echo.
echo ============================================
echo   Step 2/3: Starting backend...
echo ============================================
start "QuantPilot-Backend" /min "target\debug\quantpilot.exe"
echo   Waiting for backend to start on port 3000...
set TRIES=0
:wait
ping 127.0.0.1 -n 2 >nul
set /a TRIES+=1
netstat -ano | findstr :3000 | findstr LISTENING >nul 2>&1
if !ERRORLEVEL! EQU 0 goto ready
if !TRIES! LSS 30 goto wait
echo   [WARN] Backend did not start in 30s. Proceeding anyway...
goto launch
:ready
echo   Backend is ready on port 3000!
:launch
echo.
echo ============================================
echo   Step 3/3: Starting Tauri Desktop...
echo ============================================
cd /d "%~dp0src-tauri"
cargo tauri dev
pause
