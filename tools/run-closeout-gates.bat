@echo off
setlocal

set "ROOT=%~dp0.."
pushd "%ROOT%" || exit /b 1

powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-utf8.ps1"
if errorlevel 1 goto :fail

powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-user-facing-text.ps1"
if errorlevel 1 goto :fail

powershell -NoProfile -ExecutionPolicy Bypass -File "tools\check-capability-governance.ps1"
if errorlevel 1 goto :fail

cargo test --workspace
if errorlevel 1 goto :fail

pushd "frontend" || goto :fail

cmd /c npm run test
if errorlevel 1 goto :fail_frontend

cmd /c npm run build
if errorlevel 1 goto :fail_frontend

cmd /c npm run test:e2e
if errorlevel 1 goto :fail_frontend

popd
popd
exit /b 0

:fail_frontend
popd

:fail
popd
exit /b 1
