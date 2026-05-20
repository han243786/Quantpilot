# QuantScript Real Strategy Authoring Trial

This document records one retained-`V1` trial of writing real strategies with the
current QuantScript executable trunk.

## Scope

The goal is not to prove that QuantScript is a complete strategy platform. The
goal is narrower:

- write several realistic retained-`V1` strategies as source files
- run them through analysis
- lower them to runtime config
- compile them into runtime protocol
- confirm that formal compile also emits `quantscript_authoring_view`

## Trial Samples

The sample sources live in:

- `quantscript/authoring_samples/ma_trend_profiles.qs`
- `quantscript/authoring_samples/rsi_reversion_profiles.qs`
- `quantscript/authoring_samples/spread_bps_observer_profiles.qs`
- `quantscript/authoring_samples/equal_weight_rotation_profiles.qs`
- `quantscript/authoring_samples/universe_metadata_rotation_rank_weight.qs`
- `quantscript/authoring_samples/universe_metadata_signal_gated_rotation.qs`

The retained boundary fixture now lives separately:

- `quantscript/boundary_samples/factor_rank_rotation_boundary.qs`

The retained authoring samples cover six admitted shapes:

1. single-asset MA trend with `risk.profile("global")` and `execution.profile("paper")`
2. single-asset RSI reversion with profiles
3. narrow spread observer with explicit `align_asof(...)` and `output="bps"`
4. equal-weight rotation with retained rebalance helper path
5. metadata-ranked cross-sectional rotation with `universe/filter/sort_by/top/rebalance`
6. metadata-ranked selection plus per-symbol signal gating plus retained rebalance
The separate boundary fixture is a deliberately more realistic factor-ranked
rotation attempt used to find the first retained wall, not an admitted authoring
sample.

## Result

Under the current retained `V1` surface, all retained authoring samples:

- parse as formal QuantScript
- pass `analyze_formal_quant_script(...)` without analysis errors
- lower through `parse_formal_quant_script_config(...)`
- compile through `compile_runtime_protocol_config(...)`
- compile successfully through `/api/quantscript/formal/compile`
- emit `artifacts.strategy.metadata.quantscript_authoring_view`

That successful set now includes a more realistic degraded cross-sectional
authoring shape:

- build the candidate set from compile-time universe metadata
- compute per-symbol indicators inside the `for` loop
- gate each symbol independently with retained signals
- finish with retained rebalance

This is enough to conclude that QuantScript can already be used for a first round
of real strategy authoring, as long as authoring stays inside the retained
product surface.

## Trial-Discovered Boundary

The equal-weight rotation sample also exposed a real current limitation in
`quantscript_authoring_view` Phase 1:

- the strategy itself compiles and lowers successfully
- but the authoring artifact does not yet split the rebalance workflow into an
  ideal `data -> intent -> agent` trio
- the current line-based classifier produces `agent` and `mixed` sections around
  `symbols(...)`, the `for` loop, and `rebalance(...)`

This is not a strategy-authoring blocker. It is an authoring-artifact fidelity
gap that should inform later refinement of section classification.

## First Retained Boundary In A More Real Strategy Family

When the trial moves from single-asset and simple rebalance examples into
cross-sectional rotation, the first meaningful retained boundary is:

- QuantScript `V1` can rank and rebalance on compile-time universe metadata such
  as `market_cap`, `volume_24h`, and `listing_age_days`
- QuantScript `V1` cannot yet rank `universe(...)` selections by in-script,
  dynamically computed factor values

This is why:

- `universe_metadata_rotation_rank_weight.qs` compiles successfully when it uses
  `sort_by(liquid, key="market_cap", order="desc")`
- `universe_metadata_signal_gated_rotation.qs` also compiles successfully when it
  keeps ranking on `market_cap` but moves dynamic logic into per-symbol `ma(...)`
  and `rsi(...)` gating before `emit Intent(...)`
- `boundary_samples/factor_rank_rotation_boundary.qs` fails with `QPQSLOW011` when it attempts
  `sort_by(liquid, key="factor_score", order="desc")`

So the first real retained wall is not "you cannot do portfolio rotation at
all". The first wall is narrower and more useful:

- you can do metadata-ranked cross-sectional rotation
- you can do metadata-ranked selection plus per-symbol signal gating plus
  retained rebalance
- you cannot yet do dynamic factor-ranked universe construction in formal
  QuantScript `V1`

The documented post-`V1` replacement direction for that boundary is:

- [QuantScript Instrument Pool Minimal Contract](./QUANTSCRIPT_INSTRUMENT_POOL_MINIMAL_CONTRACT.md)

That contract does not change current support.
It records the intended replacement for the old helper-only pool model.

## What This Trial Proves

- QuantScript is not limited to toy parse-only snippets.
- The retained trunk is already strong enough for:
- direct single-asset signal strategies
- profile-attached strategies
- narrow retained spread logic
- retained rebalance workflows
- metadata-ranked cross-sectional rotation on compile-time universe metadata
- metadata-ranked selection plus per-symbol signal gating plus retained rebalance
- `quantscript_authoring_view` is now grounded in real authored strategies, not
  only synthetic compile fixtures.

## What This Trial Does Not Prove

This trial does **not** prove support for deferred or parser-only areas:

- wider spread shapes
- MACD shared-core expansion
- broader execution/risk DSL
- per-trade compare
- fill timeline compare
- richer trade-outcome analytics
- general-purpose portfolio language design

It also does not prove that compile failures emit partial authoring view.
Current Phase 1 artifact emission still happens on successful formal compile.

## Practical Authoring Rule

If you want to write real strategies against current QuantScript `V1`, stay inside
this pattern:

- top-level `fn strategy()`
- optional `# risk`, `# execution`, `# data`, `# intent`, `# agent` authoring headers
- optional `risk.profile("global", ...)`
- optional `execution.profile("paper", ...)`
- retained fetch/indicator/intention forms
- retained rebalance helper forms only

For cross-sectional work, a practical retained rule is:

- use compile-time universe metadata to build the selection set
- use per-symbol indicator logic to gate `emit Intent(...)`
- do not expect `sort_by(...)` or `top(...)` to rank symbols by in-script factor
  variables yet

Treat anything outside that retained trunk as deferred, even if the parser can
still read it.

---
> 文档版本: v3.7.0 | 最后更新: 2026-05-21 | QuantPilot v3.7.0
