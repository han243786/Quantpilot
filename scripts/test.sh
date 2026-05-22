#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if command -v taskkill >/dev/null 2>&1; then
  taskkill /f /im quantpilot.exe >/dev/null 2>&1 || true
  taskkill /f /im quantpilot-tauri.exe >/dev/null 2>&1 || true
  taskkill /f /im executor.exe >/dev/null 2>&1 || true
else
  pkill -f "target/.*/quantpilot" >/dev/null 2>&1 || true
  pkill -f "target/.*/executor" >/dev/null 2>&1 || true
fi

sleep 1

if [ "$#" -eq 0 ]; then
  set -- test --workspace
fi

exec cargo "$@"
