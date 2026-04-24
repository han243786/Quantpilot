# QuantPilot

QuantPilot is a single-machine quantitative trading sandbox focused on honest capability boundaries, reproducible runtime behavior, and release-time contract discipline.

The current release target is `v0.1.0`.

## Beta Scope

QuantPilot beta scope today:

- paper runtime only
- sandbox execution only
- exchanges within the current beta boundary: `binance`, `okx`
- symbol boundary in the current beta path: `BTCUSDT`, `ETHUSDT`, `SOLUSDT`
- graph editor, validation, compile, paper run, backtest, run history, and backtest detail/compare views
- QuantScript runtime lowering when `quantscript.formal_source` is present

## Beta Intent Kinds

Intent kinds available in the beta path:

- double moving average
- moving-average deviation
- RSI
- MACD
- Momentum
- ZScore

## Non-Claims

The following items must not be described as supported product capability:

- live trading
- research-grade backtest semantics
- true arbitrage platform support
- third-party plugin marketplace support
- arbitrary host-code execution through QuantScript

## Compile source of truth

Compile artifact priority is fixed:

- `strategy_ir` is semantic preflight only
- `quantscript.formal_source` owns runtime lowering when present
- otherwise runtime compile uses the graph-generated `runtime_config`
- runnable output always follows runtime compile, not the `strategy_ir` preflight result

Naming boundary is also fixed:

- `quantscript.formal_source` is the formal QuantScript product path
- `strategy_graph` / graph-source artifacts are graph serialization and import/export helpers, not the formal QuantScript language
- legacy section-based QuantScript config parsing remains compatibility-only inside the crate and is not the primary product entrypoint

Current syntax reference:

- [QuantScript Trunk Baseline](./markdown/guides/quantscript/guide-quantscript-trunk-baseline.md)
- [Formal QuantScript Syntax Guide](./markdown/guides/quantscript/guide-formal-quantscript-syntax.md)

## Quick start

### Backend

Run from the repository root:

```powershell
.\start-backend.bat
```

The backend listens on `http://127.0.0.1:3000`.

### Frontend

Run from the repository root:

```powershell
.\frontend\start-frontend.bat
```

The frontend dev server listens on `http://127.0.0.1:5173`.

`frontend\start-frontend.bat` installs frontend dependencies automatically when `node_modules` is missing.

## Environment variables

Frontend environment variables:

- `VITE_BACKEND_ORIGIN`
  Used by the Vite dev proxy. Defaults to `http://127.0.0.1:3000`.
- `VITE_API_BASE_URL`
  Optional direct API base. When set, the browser uses this value instead of deriving `/api` from the current origin.

See:

- `./.env.example`
- `./frontend/.env.example`

## Quality gates

Release candidates should pass all of the following:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1
cargo test --workspace
cd frontend; cmd /c npm run test
cd frontend; cmd /c npm run build
cd frontend; cmd /c npm run test:e2e
```

Canonical one-shot Windows gate wrapper:

```powershell
.\tools\run-closeout-gates.bat
```

Optional local smoke gate when editing the gate scripts themselves:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-gates-smoke.ps1
```

E2E gate contract:

- `cmd /c npm run test:e2e` must run from `frontend` without manually pre-starting the backend
- Playwright E2E uses the isolated API-mock contract and must not leak requests to `127.0.0.1:3000`

## Repository hygiene

Local build and runtime artifacts are intentionally ignored:

- Rust `target/` output
- frontend `node_modules/`, `dist/`, Playwright output, and test results
- runtime artifacts under `storage/runs/`, `storage/backtests/`, and local test artifact directories

Artifact cleanup is dry-run by default:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\cleanup-artifacts.ps1
```

Optional flags:

- `-OlderThanDays <N>`
- `-IncludeLogs`
- `-Mode execute`

## Documentation entry points

- [Docs Root](./markdown/README.md)
- [Docs Index](./markdown/overview/overview-docs-index.md)
- [Support Matrix](./markdown/implementation/governance/implementation-support-matrix.md)
- [Compile-Chain Contract](./markdown/implementation/governance/implementation-compile-chain-contract.md)
- [QuantScript Retained-Surface Contract](./markdown/implementation/governance/implementation-quantscript-retained-surface-contract.md)
- [Runtime / Backtest Explanation Contract](./markdown/implementation/runtime/implementation-runtime-backtest-explanation-contract.md)
- [Persistence / Replay Contract](./markdown/implementation/runtime/implementation-persistence-replay-contract.md)
- [Capability Governance](./markdown/implementation/governance/implementation-capability-governance.md)
- [Capability Governance Registry Snapshot](./markdown/implementation/governance/implementation-capability-governance-registry.generated.md)
- [Artifact Governance](./markdown/implementation/governance/implementation-artifact-governance.md)
- [Current Status And Release State](./markdown/overview/overview-current-status-and-roadmap.md)
- [Trading Sandbox Implementation](./markdown/implementation/runtime/implementation-trading-sandbox.md)
- [Testing Module Implementation](./markdown/implementation/runtime/implementation-testing-module.md)
- [Active QRPC RFC Index (`RFC-001` to `RFC-020`)](./markdown/protocol/README.md)

## Release checklist items still requiring owner choice

The repository can be optimized technically without guessing legal policy.
The owner decision for public visibility is now explicit:

- keep the repository private under the current all-rights-reserved placeholder
  state before any public release, even though functional development progress,
  repository stability, and release readiness are owner-scored at least `9/10`
- replace the placeholder text in [LICENSE](./LICENSE) with final approved
  outbound license text only when public-release eligibility is reconsidered
- create private baseline commits only after the accepted baseline gate passes

Shortest current release state list:

- `LICENSE` is still placeholder-only
- `tools\run-closeout-gates.bat` is the accepted private-baseline gate set
- public release remains blocked until a separate public-release approval and
  outbound license decision are made

See [First Release Readiness](./markdown/implementation/planning/implementation-first-release-readiness.md)
for the baseline go/no-go list and owner action list.

