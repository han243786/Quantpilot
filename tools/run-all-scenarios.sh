#!/bin/bash
# run-all-scenarios.sh — Run all backend scenarios + frontend E2E tests locally
# Usage: bash tools/run-all-scenarios.sh

set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BACKEND_PID=""
FRONTEND_PID=""

cleanup() {
  echo ""
  echo "[cleanup] Stopping services..."
  [ -n "$BACKEND_PID" ] && taskkill //F //PID "$BACKEND_PID" 2>/dev/null || true
  [ -n "$FRONTEND_PID" ] && taskkill //F //PID "$FRONTEND_PID" 2>/dev/null || true
}
trap cleanup EXIT

echo "════════════════════════════════════════════════"
echo " QuantPilot Full Scenario Test Suite"
echo "════════════════════════════════════════════════"

# ── Backend ──
echo ""
echo "[1/5] Building backend..."
cd "$ROOT"
cargo build --release 2>&1 | tail -3

echo "[2/5] Starting backend..."
QUANTPILOT_PORT=3000 QUANTPILOT_DEV=true QUANTPILOT_API_KEY= ./target/release/quantpilot.exe &
BACKEND_PID=$!
sleep 3

# Verify backend
for i in $(seq 1 10); do
  if curl -s http://127.0.0.1:3000/api/health > /dev/null 2>&1; then
    echo "  Backend ready (PID $BACKEND_PID)"
    break
  fi
  sleep 2
done

# ── Frontend ──
echo "[3/5] Starting frontend..."
cd "$ROOT/frontend"
npx vite --host 127.0.0.1 --port 5173 &
FRONTEND_PID=$!
sleep 5

# ── Run Backend Scenarios ──
echo ""
echo "[4/5] Running backend scenarios..."
SCENARIO_DIR="$ROOT/tests/scenarios"
FAILED=0
PASSED=0

for qs in "$SCENARIO_DIR"/scenario_*.qs; do
  if [ -f "$qs" ]; then
    echo "  Running: $(basename "$qs")"
    if node "$ROOT/tools/run-scenario.js" "$qs"; then
      PASSED=$((PASSED + 1))
    else
      FAILED=$((FAILED + 1))
    fi
  fi
done

echo "  Backend: $PASSED passed, $FAILED failed"

# ── Run Frontend E2E ──
echo ""
echo "[5/5] Running frontend E2E..."
cd "$ROOT/frontend"
npx playwright test tests/e2e/scenario-test-v2.spec.js --config=playwright.real.config.js

# ── Generate Report ──
echo ""
echo "Generating report..."
node "$ROOT/tools/generate-test-report.js"

echo ""
echo "════════════════════════════════════════════════"
echo " Done! Report: markdown/测试/测试报告-latest.md"
echo "════════════════════════════════════════════════"
