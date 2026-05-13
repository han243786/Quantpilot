@echo off
setlocal enabledelayedexpansion

set "ROOT=%~dp0.."
pushd "%ROOT%" || exit /b 1

echo ============================================
echo   QuantPilot Closeout Gates
echo   Started at %time%
echo ============================================
set START_TIME=%time%

echo [1/9] UTF-8 encoding check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-utf8.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [2/9] User-facing text check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-user-facing-text.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [3/9] Capability governance check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-capability-governance.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [4/9] cargo check --workspace...
cargo check --workspace
if errorlevel 1 goto :fail
echo        PASS

echo [5/9] cargo test --workspace...
cargo test --workspace
if errorlevel 1 goto :fail
echo        PASS

echo [6/9] cargo clippy --workspace...
cargo clippy --workspace --all-targets -- -D warnings
if errorlevel 1 goto :fail
echo        PASS

echo [7/9] npx vite build...
pushd "frontend" || goto :fail
call npx vite build
if errorlevel 1 goto :fail_frontend
echo        PASS

echo [8/9] npx vitest run...
call npx vitest run
if errorlevel 1 goto :fail_frontend
echo        PASS

echo [9/9] npm audit...
call npm audit --audit-level=moderate
if errorlevel 1 goto :fail_frontend
echo        PASS

popd
popd

echo ============================================
echo   ALL GATES PASSED
echo   Started: %START_TIME%
echo   Ended:   %time%
echo ============================================
exit /b 0

:fail_frontend
popd

:fail
popd
echo ============================================
echo   GATE FAILED at %time%
echo ============================================
exit /b 1
