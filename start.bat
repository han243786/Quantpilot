@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"
chcp 65001
set QUANTPILOT_DEV=true
echo ============================================
echo   QuantPilot Desktop v2.0.0
echo   QUANTPILOT_DEV = true
echo ============================================
taskkill /f /im quantpilot.exe
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :5173 ^| findstr LISTENING 2^>nul') do taskkill /f /pid %%a 2>nul
taskkill /f /im quantpilot-tauri.exe
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
set TRIES=0
:wait
timeout /t 1 /nobreak
set /a TRIES+=1
powershell -Command "try { $null = (New-Object Net.Sockets.TcpClient).Connect('127.0.0.1', 3000); exit 0 } catch { exit 1 }"
if !ERRORLEVEL! EQU 0 goto ready
if !TRIES! LSS 30 goto wait
echo   [WARN] Backend did not start. Proceeding anyway...
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
