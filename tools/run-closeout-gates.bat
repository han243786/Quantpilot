@echo off
setlocal enabledelayedexpansion

set "ROOT=%~dp0.."
pushd "%ROOT%" || exit /b 1

echo ============================================
echo   QuantPilot Closeout Gates
echo   Started at %time%
echo ============================================
set START_TIME=%time%

echo [1/24] UTF-8 encoding check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-utf8.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [2/24] User-facing text check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-user-facing-text.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [3/24] Capability governance check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-capability-governance.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [4/24] i18n check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-i18n.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [5/24] Version consistency check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-version-consistency.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [6/24] Feature evolution contract check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-feature-evolution.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [7/24] Developer learning closeout check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-learning-closeout.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [8/24] Pre-commit hook sync check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-pre-commit-hook.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [9/24] Cleanup boundary check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-cleanup-boundary.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [10/24] cargo fmt --check...
cargo fmt --check
if errorlevel 1 goto :fail
echo        PASS

echo [11/24] cargo check --workspace...
cargo check --workspace
if errorlevel 1 goto :fail
echo        PASS

echo [12/24] cargo test --workspace...
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\test.ps1" test --workspace
if errorlevel 1 goto :fail
echo        PASS

echo [13/24] Workspace clippy warning budget...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-clippy-warning-budget.ps1" -MaxWarnings 58
if errorlevel 1 goto :fail
echo        PASS

echo [14/24] Executor warning budget...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-executor-warning-budget.ps1" -MaxWarnings 0
if errorlevel 1 goto :fail
echo        PASS

echo [15/24] npx vite build...
pushd "frontend" || goto :fail
call npx vite build
if errorlevel 1 goto :fail_frontend
echo        PASS

echo [16/24] npx vitest run...
call npx vitest run
if errorlevel 1 goto :fail_frontend
echo        PASS

echo [17/24] npm run test:e2e...
call npm run test:e2e
if errorlevel 1 goto :fail_frontend
echo        PASS

echo [18/24] npm audit...
call npm audit --audit-level=moderate
if errorlevel 1 goto :fail_frontend
echo        PASS

popd

echo [19/24] executor frontend build...
pushd "frontend-executor" || goto :fail
call npm run build
if errorlevel 1 goto :fail_frontend
echo        PASS

popd

echo [20/24] cargo check --bin executor...
cargo check --bin executor
if errorlevel 1 goto :fail
echo        PASS

echo [21/24] cargo test --bin executor...
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\test.ps1" test --bin executor
if errorlevel 1 goto :fail
echo        PASS

echo [22/24] QS scenario smoke...
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\scenario-smoke.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [23/24] Clean worktree check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-clean-worktree.ps1"
if errorlevel 1 goto :fail
echo        PASS

echo [24/24] Full feature tree check...
powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-full-feature-tree.ps1"
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
