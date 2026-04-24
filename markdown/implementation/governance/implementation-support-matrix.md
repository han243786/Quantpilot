# QuantPilot Support Matrix

## Purpose

This document is the P0 support matrix reference for the current QuantPilot beta boundary.
Use it to keep README, UI prompts, frontend capability gates, test fixtures, and acceptance checks aligned.

The matrix describes:

- what is currently supported
- what exists only under restricted boundary conditions
- what must not be marketed as supported platform capability
- which toolbar actions are capability-gated and which backend routes they touch

## Current supported boundary

### Runtime

- Supported runtime mode: `paper`
- Supported execution module: `builtin.execution.paper`
- Current market boundary:
  - exchanges: `binance`, `okx`
  - symbols: `BTCUSDT`, `ETHUSDT`, `SOLUSDT`

### Strategy IR and QuantScript boundary

- Declared indicator kinds:
  - `ma_cross`
  - `rsi`
  - `macd`
  - `momentum`
  - `spread`
  - `z_score`
  - `custom`
- Currently supported indicator kinds:
  - `ma_cross`
  - `rsi`
  - `macd`
  - `momentum`
  - `spread`
  - `z_score`
  - `custom`

Boundary notes:

- `custom` is only supported through the restricted Strategy IR expression path that lowers into Core IR.
- `custom` does not allow arbitrary host code, direct risk mutation, or execution bypass.
- `strategy_ir` is semantic preflight only. It is not the runtime source of truth.
- When present, `quantscript.formal_source` is responsible for runtime lowering.
- When artifacts disagree, runtime behavior follows `/api/runtime/compile`.
- UI and docs must present compile interpretation as three separate fields:
  `Strategy IR role`, `Runtime source`, and `Runnable truth`.
- Exact current formal QuantScript syntax and parse-vs-lowering limits are defined in `markdown/guides/quantscript/guide-formal-quantscript-syntax.md`.

### Supported frontend module keys

- `builtin.data.kline`
- `builtin.data.quote`
- `builtin.intent.double_ma`
- `builtin.intent.ma_deviation`
- `builtin.intent.rsi`
- `builtin.intent.macd`
- `builtin.intent.momentum`
- `builtin.intent.zscore`
- `builtin.intent.spread_observer`
- `builtin.agent.weighted`
- `builtin.agent.arbitrage`
- `builtin.risk.global`
- `builtin.execution.paper`
- `builtin.runtime.control`

Boundary notes:

- Spread-related and arbitrage-related module keys may appear in the beta compile path.
- They must not be described externally as proof of true arbitrage platform support.
- Frontend module exposure must remain aligned with `/api/capabilities`.
- `builtin.data.kline` and `builtin.data.quote` now expose `ping_enabled` and `request_interval_ms`
  in the frontend/runtime compile path.
- These request-control fields are not yet part of graph-generated formal QuantScript surface.

## Capability-gated UI actions

| Action | Capability gate | Backend routes | Notes |
|---|---|---|---|
| `Compile` | locked while capability sync is loading or when safe fallback is active | `/api/strategy-ir/compile`, `/api/quantscript/formal/compile`, `/api/runtime/compile` | `strategy_ir` is preflight only; runtime compile stays authoritative |
| `Export config` | locked while capability sync is loading or when safe fallback is active | `/api/runtime/compile` | export depends on a compilable runtime config |
| `Start simulation` | locked while capability sync is loading or when safe fallback is active | `/api/runtime/test-run`, `/api/runtime/runs/:run_id/events`, `/api/runtime/runs/:run_id/status` | current beta boundary is paper runtime only |
| `Run backtest` | locked while capability sync is loading or when safe fallback is active | `/api/runtime/backtest`, `/api/runtime/backtests`, `/api/runtime/backtests/:backtest_id` | current backtest is basic replay/backtest support only |
| `Run parameter sweep` | locked while capability sync is loading or when safe fallback is active | `/api/runtime/experiments/backtest-sweep`, `/api/runtime/experiments`, `/api/runtime/experiments/:experiment_id` | narrow execution-assumptions sweep on top of the existing backtest surface; not a second experiment runtime |
| `Export strategy_graph source` | not capability-gated | none | frontend graph-source draft export only; this is not the formal QuantScript language |

## Visible workspace surfaces outside `/api/capabilities` gating

These surfaces are real and visible in the current product, but they do not all derive their visibility from backend capability discovery.

| Surface | Visibility source of truth | Backend routes | Capability-driven? | Notes |
|---|---|---|---|---|
| `Strategy template library` | frontend local template registry | none | no | local starter-graph surface only; loading a template replaces the in-memory draft and does not create a backend template transport |
| `Version history` | graph persistence workflow | `/api/graphs/:graph_id/versions`, `/api/graphs/:graph_id/versions/:version_id`, `/api/graphs/:graph_id/versions/:version_id/restore`, `/api/graphs/:graph_id/versions/compare` | no | visible because persisted graph artifacts exist, not because `/api/capabilities` advertises a second runtime capability |
| `Collaboration and audit` | graph collaboration metadata and audit projection | `/api/graphs/:graph_id/audit` | no | current slice is local-actor collaboration metadata, not a remote account system |
| `Parameter sweep` | runtime backtest surface plus capability-governed trigger | `/api/runtime/experiments/backtest-sweep`, `/api/runtime/experiments`, `/api/runtime/experiments/:experiment_id` | yes | visible as a narrow workspace card, but its submit action must obey the same capability lock rules as backtest |

## Capability source behavior

### `remote`

- Normal operating state.
- Frontend should trust `/api/capabilities` as the active capability reference.

### `cache`

- Degraded but still usable.
- Frontend may keep actions available.
- UI must state that final availability still depends on live backend validation.

### `safe_fallback`

- Risk containment state.
- Frontend must hide unsupported module entry points and lock risky actions.
- UI must explain that capability verification failed and the frontend has tightened behavior to avoid exposing fake capabilities.

## Statements that are allowed

- `paper runtime beta`
- `basic backtest support`
- `restricted Custom Strategy IR expression path`

## Statements that must not appear as positive support claims

- claiming research-grade backtest support
- claiming live trading support
- claiming true arbitrage agent support
- claiming third-party plugin marketplace support

## References

- [Current Status And Release State](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/overview/overview-current-status-and-roadmap.md)
- [Compile-Chain Contract](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-compile-chain-contract.md)
- [Completed Functional Closeout Ledger](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/planning/implementation-functional-closeout-task-table.md)
- [First Release Readiness](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/planning/implementation-first-release-readiness.md)

