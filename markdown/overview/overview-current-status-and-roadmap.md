# Current Status And Release State

## Current product truth

QuantPilot already has a runnable end-to-end beta chain:

- frontend graph editor, validation, compile, runtime event display, and
  backtest detail page
- backend Axum API for graph save/load, compile, paper run, backtest, and
  capability discovery
- runtime chain for data -> intent -> agent -> risk -> execution -> fill
- QuantScript AST, helper function analysis, formula lowering, and
  runtime-backed indicator intents

Current K-line-driven intent support is real:

- double moving average
- moving-average deviation
- RSI
- MACD
- Momentum
- ZScore

Current basic backtest support is also real:

- historical replay
- persisted backtest records
- equity curve
- basic summary metrics

## What must not be overstated

The following items are still not real platform capability and must not be
described as supported:

- true arbitrage agent support in the current spot beta
- research-grade backtest with complete market microstructure semantics
- any paper strategy can be expressed directly in QuantScript

The frontend now treats these gaps honestly:

- unsupported modules are not shown in the standard module sidebar
- legacy graphs still load, but unsupported modules are surfaced as explicit
  validation errors
- backend exposes `/api/capabilities` as the current source of truth for
  supported modules, runtime modes, indicators, exchanges, and symbols
- `/api/capabilities` now exposes both compatibility fields and structured
  support entries:
  - `strategy_ir.indicator_support`
  - `runtime.mode_support`
  - `runtime.execution_module_support`
  - `market_data.exchange_support`
  - `market_data.symbol_support`
  - `frontend.declared_module_keys`
  - `frontend.module_support`

Current declared-but-not-supported truth must be read literally:

- `Custom` is only supported through the restricted Strategy IR expression path
  that lowers into Core IR
- `Custom` does not allow arbitrary host code, direct risk mutation, or direct
  execution bypass
- plugin manifest and registry support now exist in `qrpc_core`, but the
  current plugin-marketplace slice is still local-metadata only; it is not yet
  a remote install or third-party distribution surface
- old consumers may still read legacy summary fields, but new consumers should
  prefer the structured support entries

## Active contract boundaries

The compile chain has a fixed priority and should be described consistently in
UI, docs, and tests:

- `strategy_ir` is a semantic preflight artifact only
- `strategy_ir` can fail the compile early, but it does not replace the runtime
  compile source of truth
- `quantscript.formal_source` is responsible for runtime lowering when present
- if formal QuantScript lowering is unavailable, the system falls back to the
  graph-generated `runtime_config`
- when these artifacts disagree, runtime behavior follows the runtime compile
  source of truth, not the `strategy_ir` preflight artifact

The current contract details now live in dedicated docs instead of being
repeated here:

- compile interpretation:
  [Compile-Chain Contract](../implementation/governance/implementation-compile-chain-contract.md)
- runtime and backtest explanation:
  [Runtime / Backtest Explanation Contract](../implementation/runtime/implementation-runtime-backtest-explanation-contract.md)
- persistence and replay:
  [Persistence / Replay Contract](../implementation/runtime/implementation-persistence-replay-contract.md)
- QuantScript retained surface:
  [QuantScript Retained-Surface Contract](../implementation/governance/implementation-quantscript-retained-surface-contract.md)

## Current architecture boundary

Today the system should be understood as:

- one paper/runtime beta centered on `BTCUSDT`, `ETHUSDT`, and `SOLUSDT`
- supported exchanges limited to `binance` and `okx`
- supported runtime mode limited to `paper`
- supported execution module limited to `builtin.execution.paper`
- supported frontend modules limited to modules the backend can compile
  one-to-one

QuantScript is stronger than a config shell, but it is still not a full
research language. The parser still accepts some broader syntax, but future
development must contract toward a narrow trunk:

- data fetch/alignment
- whitelisted indicators
- constrained universe/filter/score/top-k pipelines
- minimal control flow
- standardized `emit Intent(...)`

Risk/execution details, general state, and general-purpose language features
are not the intended growth path.

Use these docs as the active references:

- [QuantScript Trunk Baseline](../guides/quantscript/guide-quantscript-trunk-baseline.md)
- [Formal QuantScript Syntax Guide](../guides/quantscript/guide-formal-quantscript-syntax.md)
- [V1 Freeze / De-scope Checklist](../guides/quantscript/guide-v1-freeze-descope-checklist.md)

## Current closeout / release state

The current optimization priority is release-state confirmation, not capability
expansion.
Use the dedicated docs below as the active release surface:

- [First Release Readiness](../implementation/planning/implementation-first-release-readiness.md)
- [Support Matrix](../implementation/governance/implementation-support-matrix.md)
- [Test Layer Expectations](../implementation/runtime/implementation-test-layer-expectations.md)
- [Archived Functional Closeout Ledger](../archive/planning-retired/implementation-functional-closeout-task-table.md)

Current repository-level status:

- `cargo test --workspace` passes
- `cargo clippy --workspace --all-targets -- -D warnings` passes
- frontend unit tests pass
- the canonical Windows frontend gate form is `cmd /c npm run ...`
- as of the latest `2026-04-26` P0/P1 closeout, `cmd /c npm run test:e2e`
  passes from `frontend` without a manually pre-started backend because the
  suite stays on the isolated API-mock contract
- UTF-8 and user-facing text gates pass
- capability-governance snapshot is current
- P1 history filtering and save flows now keep freshly saved run/backtest
  records visible through stale filters, then reload persisted detail state
- the accepted closeout wrapper passes; remaining cleanup is P2 repository
  hygiene and public-release blocker handling
- opt-in visual review route/API fixture drift was repaired on `2026-04-26`
  and re-checked on `2026-04-28`; the spec now captures strategy hub, strategy
  workspace, backtest detail, and backtest compare screenshots with
  reduced-motion capture when `VISUAL_REVIEW=1` is set
- the `postcss <8.5.10` moderate audit finding was fixed through
  `postcss@8.5.12`; the remaining npm audit risk is the Vite/esbuild chain,
  still accepted only for private-baseline use and still blocking public
  release claims

## V1 freeze direction

- treat the current formal QuantScript trunk, landed shared-core slices,
  outward-moved `risk.profile(...)` / `execution.profile(...)`, the first
  narrow spread slice, and the first executable backtest/report slice as the
  retained `V1` surface
- treat wider spread contracts, `MACD` shared-core expansion, generic
  risk/execution DSL growth, per-trade compare, fill-timeline compare, and
  broader research-report expansion as deferred work
- before declaring `V1` closed, prefer deleting duplicate truth sources,
  compressing completed queue items, and keeping docs/prompts/UI wording
  aligned with the retained surface instead of widening feature scope

## Acceptance rule

No feature should be exposed in UI, docs, or prompts unless all of the
following are true:

- backend compile path exists
- runtime semantics exist
- validation can reject unsupported use honestly
- event output can explain behavior
- tests cover the path
- frontend and docs text is saved as UTF-8 and verified to display correctly in
  the rendered product
