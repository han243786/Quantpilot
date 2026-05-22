@echo off
setlocal enabledelayedexpansion

set "ROOT=%~dp0.."
pushd "%ROOT%" || exit /b 1

echo ============================================
echo   QuantPilot Closeout Gates
echo   Started at %time%
echo ============================================
set START_TIME=%time%

echo [1/17] UTF-8 encoding check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-utf8.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [2/17] User-facing text check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-user-facing-text.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [3/17] Capability governance check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-capability-governance.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [4/17] i18n check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-i18n.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [5/17] Version consistency check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-version-consistency.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [6/17] cargo check --workspace...
cargo check --workspace
if errorlevel 1 goto :fail
echo        PASS

echo [7/17] cargo test --workspace...
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\test.ps1" test --workspace
if errorlevel 1 goto :fail
echo        PASS

echo [8/17] cargo clippy --workspace...
cargo clippy --workspace --all-targets
if errorlevel 1 goto :fail
echo        PASS

echo [9/17] Executor warning budget...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-executor-warning-budget.ps1" -MaxWarnings 49
if errorlevel 1 goto :fail
echo        PASS

echo [10/17] npx vite build...
pushd "frontend" || goto :fail
call npx vite build
if errorlevel 1 goto :fail_frontend
echo        PASS

echo [11/17] npx vitest run...
call npx vitest run
if errorlevel 1 goto :fail_frontend
echo        PASS

echo [12/17] npm run test:e2e...
call npm run test:e2e
if errorlevel 1 goto :fail_frontend
echo        PASS

echo [13/17] npm audit...
call npm audit --audit-level=moderate
if errorlevel 1 goto :fail_frontend
echo        PASS

popd

echo [14/17] executor frontend build...
pushd "frontend-executor" || goto :fail
call npm run build
if errorlevel 1 goto :fail_frontend
echo        PASS

popd

echo [15/17] cargo check --bin executor...
cargo check --bin executor
if errorlevel 1 goto :fail
echo        PASS

echo [16/17] cargo test --bin executor...
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\test.ps1" test --bin executor
if errorlevel 1 goto :fail
echo        PASS

echo [17/17] QS scenario smoke...
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\scenario-smoke.ps1"
if errorlevel 1 goto :fail
echo        PASS

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
