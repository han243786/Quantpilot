# Backtest Execution-Assumptions Minimal Contract

## Purpose

This document defines the first honest contract for research-grade backtest
execution assumptions.

The goal is not to introduce a broad execution DSL.
The goal is to make the current backtest result explicitly state the three
assumptions that most directly change simulated fills:

- `fee_bps`
- `slippage_bps`
- `latency_ms`

Everything else should remain out of scope for the first step.

## Why This Comes First

Backtest output becomes misleading faster from hidden execution assumptions than
from missing richer summary metrics.

The first `P4` slice should therefore:

1. make execution assumptions explicit
2. make their override order explicit
3. make their compile/runtime path consistent across entry points

Only after that should the roadmap expand toward richer metrics, trade ledger,
artifact management, or compare workflows.

## Minimal Field Set

The first contract admits only these fields:

- `fee_bps`
  - float
  - must be `>= 0`
  - means per-fill fee assumption in basis points
- `slippage_bps`
  - float
  - must be `>= 0`
  - means simulated price slippage in basis points
- `latency_ms`
  - integer
  - must be `>= 0`
  - means assumed decision-to-fill latency for backtest simulation

Do not add:

- spread model DSL
- queue position model
- market impact model
- partial-fill policy DSL
- venue routing policy
- per-order overrides inside trunk QuantScript

## Ownership Split

The first contract should be split across two layers.

### Layer 1. Shared profile defaults

`execution.profile(...)` should own the reusable default values that belong to
the strategy shape itself.

For the first `P4` slice, that means:

- `slippage_bps`
- `fee_bps`

`latency_ms` is intentionally excluded from the first `execution.profile(...)`
extension, because it is more naturally tied to a specific simulation run than
to a strategy identity.

### Layer 2. Backtest request override

The backtest request should own run-specific execution assumptions.

For the first `P4` slice, that means:

- optional `fee_bps`
- optional `slippage_bps`
- optional `latency_ms`

These fields should override any profile defaults for that specific backtest
request only.

## Override Rule

The first backtest execution-assumptions contract should resolve values in this
order:

1. backtest request explicit override
2. execution profile default
3. backend fallback default

This order must be documented and tested.

## Backend Fallback Defaults

Until a richer execution-assumptions layer exists, the first defaults should
remain simple and explicit:

- `fee_bps = 10.0`
- `slippage_bps = 5.0`
- `latency_ms = 0`

If these defaults change later, the docs, compile path, runtime path, and tests
must all be updated together.

## Cross-Entry Alignment Rule

The first slice is only worth implementing if the same semantic shape can be
expressed across:

- graph/runtime compile
- Strategy IR
- formal QuantScript

That does not mean each entry point must carry the same syntax.
It means they must lower to the same backtest execution-assumptions shape.

## Recommended Entry-Point Shape

### Graph/runtime

Graph/runtime should continue using the existing execution node plus explicit
backtest-request overrides.

The first landed graph/runtime work should not invent a second execution node
type.

### Strategy IR

Strategy IR should extend the existing narrow `execution_profile` path rather
than introduce a separate backtest-only execution DSL.

The first addition should be:

- `fee_bps` on the same narrow execution profile shape

while `latency_ms` remains request-scoped.

### Formal QuantScript

Formal QuantScript should keep using the narrow top-level:

```qs
execution.profile("paper", slippage_bps=5.0, fee_bps=0.0)
```

It should not admit `latency_ms` inside the trunk language in the first slice.
`latency_ms` belongs to the backtest request layer.

## Minimal Executable Task Order

The first `P4` slice should be implemented in this order:

1. Extend the `execution.profile(...)` contract doc to add `fee_bps` and keep
   `slippage_bps`.
2. Add backtest request fields for `fee_bps`, `slippage_bps`, and `latency_ms`.
3. Define one shared resolved execution-assumptions shape in the backend.
4. Make graph/runtime compile use that shape.
5. Make Strategy IR use the same shape.
6. Make formal QuantScript use the same shape.
7. Add explicit precedence tests for request override vs profile default.
8. Add backtest artifact/report visibility so users can see which assumptions
   were actually applied.

## Minimal Test Matrix

The first implementation should not ship without:

1. compile tests that prove all three entry points lower to the same execution
   assumption shape
2. override-order tests that lock request override > profile default > backend
   fallback
3. validation tests that reject negative `fee_bps`, negative `slippage_bps`,
   and negative `latency_ms`
4. backtest artifact tests that show the applied assumptions in a stable output
   location

## Current Status

This contract is now partially landed.

Already landed:

- narrow `execution.profile("paper", fee_bps=..., slippage_bps=...)` across
  formal QuantScript, graph runtime compile, and Strategy IR
- request-scoped backtest overrides for `fee_bps`, `slippage_bps`, and
  `latency_ms`
- backend resolution order locked as `request override > profile default >
  backend fallback`
- backtest artifact manifest now records the resolved execution assumptions
- `latency_ms` now acts as a backtest execution-clock lag
- that lag now shifts execution and portfolio timestamps in backtest outputs and
  projected artifacts, including the event log, trade ledger, and equity curve
- the same resolved assumptions are now also projected into
  `metrics.execution_assumptions`, so users do not need to open the manifest to
  see the applied `fee_bps`, `slippage_bps`, and `latency_ms`
- backtest run/detail responses now also expose the same assumptions module at
  top level as `execution_assumptions`, so clients do not need to dig into
  artifact internals for the minimal assumptions view
- backtest list responses now also expose a compressed
  `filters.execution_assumptions_tag` view with a value label and a source label
  for lightweight scanning
- `metrics.execution_assumptions`, run/detail top-level
  `execution_assumptions`, and list-level `filters.execution_assumptions_tag`
  are now organized around one shared assumptions module shape instead of three
  unrelated summaries
- that summary now also carries per-field source tags
  (`request_override`, `profile_default`, `backend_fallback`), and those source
  tags are required to stay consistent with the embedded backtest manifest
- artifact/unit tests and API golden-like tests now lock the
  `metrics.execution_assumptions` field set, value source, and manifest
  consistency for this minimal assumptions slice

Not yet landed:

- richer artifact/report visibility beyond the current timestamp projections

Partially landed:

- a minimal compare workflow now exists for exactly two backtest ids
- that compare output now exposes four stable top-level blocks:
  - `execution_assumptions`
  - `metrics`
  - `trade_ledger`
  - `report_narrative`
- the block reports one of three statuses:
  - `same`
  - `different`
  - `missing`
- the `execution_assumptions` block exposes field-level diff statuses for:
  - `fee_bps`
  - `slippage_bps`
  - `latency_ms`
  - `sources`
- the `metrics` block exposes field-level diff statuses for:
  - `step_count`
  - `trade_count`
  - `total_return_ratio`
  - `max_drawdown_ratio`
  - `final_equity`
  - `net_profit`
  - `turnover_ratio`
  - `average_trade_notional`
  - `fee_drag_ratio`
- the `metrics` block now also exposes a grouped `drilldown` layer:
  - `performance`
  - `activity`
  - `costs`
- each drill-down group still only reports field-level `same` / `different` /
  `missing`, but each field now also carries the left and right resolved values
  so compare clients can explain differences without re-deriving them
- this slice still does not add timeline or per-trade metrics compare
- the `trade_ledger` block exposes field-level diff statuses for:
  - `trade_count`
  - `buy_fill_count`
  - `sell_fill_count`
  - `total_fees_paid`
  - `buy_fees_paid`
  - `sell_fees_paid`
  - `total_filled_notional`
  - `buy_filled_notional`
  - `sell_filled_notional`
  - `average_fill_price`
  - `average_buy_fill_price`
  - `average_sell_fill_price`
  - `average_fee_per_fill`
  - `average_buy_fee`
  - `average_sell_fee`
- the `report_narrative` block is now a stable report module with:
  - a headline
  - short bullets
  - top-level highlights
  - friendly source explanations
  - explicit sections for `Execution assumptions`, `Metrics summary`, and
    `Trade ledger summary`
- the compare response now also exposes a top-level `compare_report` view that
  organizes the same compare truth into:
  - one shared headline
  - an overview layer (`bullets` plus `highlights`)
  - module views for `execution_assumptions`, `metrics`, `trade_ledger`, and
    `equity_curve`
- under the retained `V1` surface, `report_narrative` and `compare_report`
  still coexist as the outward compare/report contract; the post-`V1`
  migration path toward `compare_report` as the single external report truth is
  tracked in [Compare Report V1 Post-Migration Checklist](./guide-compare-report-v1-post-migration-checklist.md)
- each report section now also carries a one-line `summary`, so compare clients
  can render a compact narrative layer before they drill into section lines
- compare/report now also carries a narrow time-series `equity_curve` module:
  - summary-field compare over `point_count`, `started_at_ms`, `ended_at_ms`,
    `first_equity`, `final_equity`, `min_equity`, and `max_equity`
  - sample drill-down for `start`, `middle`, and `end`
  - left/right sample values for `ts_ms`, `equity`, `cash_balance`, and
    `net_notional`
- the report narrative now also includes an explicit `Equity curve` section so
  this time-series drill-down is visible in the same report layer as
  assumptions, metrics, and trade ledger
- this remains a narrow time-series compare contract, not a full timeline
  compare UI or a free-form series analysis DSL
- the `Trade ledger summary` section now uses one shared ledger-summary module
  instead of a separate compare-only shape, so compare/report output stays
  aligned with artifact truth for fill counts, fee splits, notional, and
  average fill prices
- that `report_narrative` block now also carries a small source explanation
  section that translates `request_override`, `profile_default`, and
  `backend_fallback` into user-facing labels for fee, slippage, and latency
- the compare block carries the left and right assumptions modules so clients can
  compare resolved values and source tags without re-deriving them
- compare still does not yet cover richer metrics, ledger drill-down, or broader
  report sections beyond this first narrative layer
- the current richer metrics slice is now intentionally narrow and cost-aware:
  - `net_profit`
  - `turnover_ratio`
  - `average_trade_notional`
  - `fee_drag_ratio`
- those richer metrics are derived from the projected equity curve plus the
  landed trade-ledger summary, rather than from a separate metrics-only
  calculation path

## Non-Goals For This Slice

Do not expand this slice into:

- execution microstructure modeling
- probabilistic slippage
- spread-sensitive fill model DSL
- order book simulation
- venue-specific fee tables
- strategy-local latency scripting

Those belong to later review, not to the first `P4` executable contract.
