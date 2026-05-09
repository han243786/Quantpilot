@echo off
setlocal
chcp 65001 >nul 2>&1
cd /d "%~dp0"

echo ============================================
echo   QuantPilot Desktop
echo ============================================
echo   Starting Tauri dev mode...
echo   Window will open when ready.
echo ============================================

cd /d "%~dp0src-tauri"
cargo tauri dev

pause
