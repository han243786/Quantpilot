# V1 Freeze / De-scope Checklist

This document defines the retained `V1` surface and explicit de-scope boundary.
If a task is not explicitly retained below, treat it as deferred rather than as
unfinished work in the current phase.

## V1 Goal

Close the current phase with one honest, narrow, test-guarded product surface:

- formal QuantScript trunk remains explicit and bounded
- stable shared-core slices stay aligned across entry points
- structured diagnostics replace the obvious plain-string failure paths
- `risk.profile(...)` and `execution.profile(...)` keep complexity outside the
  main strategy syntax
- the first executable `P4` slice is visible and comparable through artifacts,
  detail views, list summaries, and compare/report workflow

## Retained In V1

### 1. QuantScript trunk and shared-core

- formal QuantScript stable trunk only
- landed shared-core slices:
  - direct MA compare
  - one-sided `RSI`
  - one-sided `momentum`
  - one-sided `zscore`
  - first narrow spread slice
- cross-entry equivalence guardrails for the landed slices
- explicit early-fail behavior for parser-accepted but unsupported constructs

### 2. Structured diagnostics

- `QPQSLOW` structured failures for the stable helper/input contracts that are
  already landed
- golden-like API response-shape coverage for representative compile-side and
  lowering-side failures
- spread formal rejection golden-like coverage for the admitted narrow slice

### 3. Outward-moved profile complexity

- `risk.profile("global")`
- `execution.profile("paper")`
- keep both contracts narrow; do not widen them into generic DSLs during `V1`

### 4. First executable P4 slice

- backtest assumptions:
  - `fee_bps`
  - `slippage_bps`
  - `latency_ms`
- override precedence:
  - `request override > profile default > backend fallback`
- artifact/detail/list visibility for the assumptions module
- compare/report workflow for:
  - `execution_assumptions`
  - `metrics`
  - `trade_ledger`
  - `equity_curve`
- current richer metrics slice:
  - `net_profit`
  - `turnover_ratio`
  - `average_trade_notional`
  - `fee_drag_ratio`

## Deferred Beyond V1

These items are explicitly deferred. Do not let them re-enter the active
closeout notes
unless a new contract review reopens them.

### 1. Shared-core and language expansion

- `MACD` shared-core expansion
- wider spread contracts: `ratio`, `absolute`, dual-sided spread, line-vs-line spread
- arbitrary user-defined indicator DSL
- arbitrary comparator DSL
- generalized language features outside the current trunk

### 2. Risk and execution expansion

- generic risk DSL
- generic execution DSL
- `broker.profile(...)` expansion beyond a separately approved minimal contract
- strategy-local latency scripting
- venue-specific fee-table systems

### 3. Backtest and report expansion

- per-trade compare
- fill-timeline compare
- full timeline compare UI
- probabilistic slippage
- market-impact simulation
- order book simulation
- wider trade-outcome analytics that require a new contract review:
  - `winning_trade_count`
  - `losing_trade_count`
  - `average_trade_return`
- broader compare/report DSL behavior

## Delete Or Compress Before Phase Close

### 1. Delete duplicate truth sources

- remove old compare/report shapes that duplicate artifact truth
- remove helper logic that re-derives assumptions, metrics, trade-ledger, or
  equity-curve summaries from already unified modules
- remove tests that preserve outdated compare/report expectations once the new
  unified module is authoritative
- after `V1`, migrate from the retained dual `report_narrative` /
  `compare_report` surface toward `compare_report` as the single external
  report truth; the ordered cleanup lives in [Compare Report V1 Post-Migration Checklist](./guide-compare-report-v1-post-migration-checklist.md)

### 2. Compress active planning docs

- move completed tasks out of the active closeout docs
- compress phase-complete slices into short factual status bullets
- stop listing already landed work as if it were still a next-step task

### 3. Keep one authoritative wording path

- compare/report wording should come from the unified compare/report module
- docs should distinguish current implementation fact from deferred work
- prompts and product wording should not mention deferred capabilities as if
  they were near-term work

## V1 Exit Checklist

Treat `V1` as phase-closed only when all of the following are true:

1. Active docs no longer present deferred work as current implementation.
2. The active closeout docs are compressed so they contain only unfinished work.
3. Artifact, detail, list, and compare/report views all point at the same
   assumptions / metrics / trade-ledger / equity-curve truth modules.
4. Compare/report contracts are covered by stable API or artifact guardrails.
5. No deferred capability is silently exposed in docs, prompts, or UI wording.
6. UTF-8 checks pass on the active docs that define the retained `V1` surface.

## Operating Rule After Freeze

After `V1` freeze:

- optimize and clean within the retained surface
- do not widen the retained contracts without a new written contract review
- treat deferred items as the start of a later phase, not as leftover TODOs
