@echo off
setlocal enabledelayedexpansion

set "ROOT=%~dp0.."
pushd "%ROOT%" || exit /b 1

echo ============================================
echo   QuantPilot Closeout Gates
echo   Started at %time%
echo ============================================
set START_TIME=%time%

echo [1/22] UTF-8 encoding check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-utf8.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [2/22] User-facing text check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-user-facing-text.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [3/22] Capability governance check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-capability-governance.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [4/22] i18n check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-i18n.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [5/22] Version consistency check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-version-consistency.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [6/22] Feature evolution contract check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-feature-evolution.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [7/22] Developer learning closeout check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-learning-closeout.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [8/22] Pre-commit hook sync check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-pre-commit-hook.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [9/22] Cleanup boundary check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-cleanup-boundary.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [10/22] cargo fmt --check...
cargo fmt --check
if errorlevel 1 goto :fail
echo        PASS

echo [11/22] cargo check --workspace...
cargo check --workspace
if errorlevel 1 goto :fail
echo        PASS

echo [12/22] cargo test --workspace...
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\test.ps1" test --workspace
if errorlevel 1 goto :fail
echo        PASS

echo [13/22] cargo clippy --workspace...
cargo clippy --workspace --all-targets
if errorlevel 1 goto :fail
echo        PASS

echo [14/22] Executor warning budget...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-executor-warning-budget.ps1" -MaxWarnings 47
if errorlevel 1 goto :fail
echo        PASS

echo [15/22] npx vite build...
pushd "frontend" || goto :fail
call npx vite build
if errorlevel 1 goto :fail_frontend
echo        PASS

echo [16/22] npx vitest run...
call npx vitest run
if errorlevel 1 goto :fail_frontend
echo        PASS

echo [17/22] npm run test:e2e...
call npm run test:e2e
if errorlevel 1 goto :fail_frontend
echo        PASS

echo [18/22] npm audit...
call npm audit --audit-level=moderate
if errorlevel 1 goto :fail_frontend
echo        PASS

popd

echo [19/22] executor frontend build...
pushd "frontend-executor" || goto :fail
call npm run build
if errorlevel 1 goto :fail_frontend
echo        PASS

popd

echo [20/22] cargo check --bin executor...
cargo check --bin executor
if errorlevel 1 goto :fail
echo        PASS

echo [21/22] cargo test --bin executor...
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\test.ps1" test --bin executor
if errorlevel 1 goto :fail
echo        PASS

echo [22/22] QS scenario smoke...
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
