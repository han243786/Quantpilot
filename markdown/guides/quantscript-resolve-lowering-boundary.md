# QuantScript Resolve vs Lowering Boundary

## Purpose

This document freezes the current Week 3 boundary between `resolve` and `lowering` in `quantscript`.

For the historical first mechanical split of the old single-file `lowering.rs`, use:

- [QuantScript First Lowering Split Patch Plan](./quantscript/guide-quantscript-first-lowering-split-patch-plan.md)

Current code status:

- the old single-file `quantscript/src/lowering.rs` has already been replaced by `quantscript/src/lowering/mod.rs`
- `context`, `shared`, `diagnostics`, and `universe` have been extracted
- `semantic` now holds alias-aware semantic bridge helpers shared by `bindings`, `fallback`, and `intents`
- `binding_sources` now owns source inference and decode helpers
- `source_recovery` now owns change-source reconstruction helpers that were previously mixed into `fallback`
- `bindings` now focuses on indicator binding assembly, while `intents` remains a dedicated lowering module
- `fallback` has now been extracted into `quantscript/src/lowering/fallback.rs`
- low-level matcher tests have been moved out of `orchestrator` and into the owning lowering modules
- `binding_sources` no longer depends on `fallback`, helper-function indicator assembly has moved back into `bindings`, spread-only operand/match types have been pulled into `intents`, `helper_env` now owns the narrow helper-function environment hydration plus shared stmt-binding walk used by both `bindings` and `source_recovery`, and `fallback` now exposes a thinner manual-formula facade instead of a broad matcher list, but `source_recovery`, `bindings`, `fallback`, and `intents` still share several internal helper surfaces, so the structural split is ahead of the final coupling cleanup
- `source_recovery` no longer reaches into `bindings::collect_bindings_from_stmts` directly; helper-body local binding recovery now goes through the narrower `helper_env` surface instead of depending on the full `bindings` collection path
- the remaining generic missing-argument helper strings are now largely confined to `shared.rs`; recent cleanup passes have already pulled the stable, user-visible failures in `universe`, `binding_sources`, and `intents` back behind structured `QPQSLOW` contracts
- the first minimal `risk.profile(...)` contract is now implemented one-to-one across formal QuantScript, graph runtime compile, and Strategy IR; it intentionally bypasses `resolve` semantics for now and lowers directly to the existing `builtin.risk.global` runtime shape
- spread now also has a formal minimal-contract document that freezes time alignment and the first intended shared-core slice; graph/runtime, Strategy IR, and formal QuantScript have now all landed the same narrow `bps` one-sided threshold slice, while broader spread arithmetic and helper-derived forms still remain outside the admitted shared-core path

The goal is simple:

- keep `resolve` as the owner of stable semantic facts
- keep `lowering` as the owner of runtime binding construction
- prevent `resolve` from drifting into a second full lowering pipeline

## Resolve Owns

`resolve` is responsible for semantic information that is stable, reusable, and does not require full runtime context.

- name resolution
- base type inference
- builtin/imported/helper classification
- member capability classification
- unified return-type rules for call-style and member-style helpers
- `ResolveResult.expr_semantics` for standardized expression semantics

Today, the stable semantics already produced by `ResolveResult.expr_semantics` are:

- `SeriesView`
- `WindowAggregateView`
- `BoundaryLookbackPair`
- `BalancedSmoothedChangePair`
- `ManualIndicatorFormula::Momentum`
- `ManualIndicatorFormula::MovingAverage`
- `ManualIndicatorFormula::ZScore`
- `ManualIndicatorFormula::MacdLine`
- `ManualIndicatorFormula::MacdHistogram`

## Resolve Does Not Own

`resolve` must not become the place where full runtime lowering is reimplemented.

It does not own:

- runtime binding construction
- data-source binding construction
- indicator binding construction
- execution intent construction
- complex cross-statement dataflow
- recursive or unbounded helper expansion
- full strategy-level lowering

Alias-aware recognition in `resolve` is intentionally limited.

- only minimal let-to-expr alias following is allowed
- alias recovery is used only to stabilize semantic facts already known to be worth standardizing
- once a pattern requires broader flow reasoning, recursive expansion, or runtime-specific interpretation, it stays in `lowering`

## Lowering Owns

`lowering` is responsible for turning resolved semantic facts into runtime-facing bindings and config.

It owns:

- source recovery for runtime bindings
- runtime indicator binding construction
- data-source inference for runtime config
- warmup and window propagation into runtime-facing outputs
- fallback matcher paths for unstandardized or more complex forms
- final lowering into Core IR or runtime config
- explicit rejection of conditional `emit Intent(...)` statements when the condition cannot be mapped to a supported runtime intent shape
- structured lowering diagnostics for known executable-contract failures such as unsupported conditional `emit Intent(...)` shapes

When a stable semantic annotation is available, `lowering` should consume it first and only do the minimum remaining extraction work.

When no stable annotation is available, `lowering` may still use local matchers and fallback logic.

Current module shape:

- `orchestrator` owns top-level lowering flow
- `universe` owns compile-time universe expansion and rebalance directive recovery
- `semantic` owns resolve-to-lowering semantic bridge helpers and alias-aware expression targeting
- `binding_sources` owns source inference, fetch/source recovery, and decode helpers
- `source_recovery` owns change-source reconstruction helpers shared by source inference and matcher layers
- `bindings` owns runtime indicator binding assembly
- `fallback` owns matcher-heavy compatibility recovery paths
- `intents` owns runtime-facing Intent construction

## RSI Boundary

RSI is the current example of a deliberate split boundary.

Already moved into `resolve`:

- `BalancedSmoothedChangePair { period, smoothing }`

Still kept in `lowering`:

- the outer RSI formula shell
- final `RsiMethod` mapping
- remaining runtime-oriented recovery around the full formula shape

This is intentional. The core stable parameter layer is standardized in `resolve`; the outer shell remains in `lowering` until it becomes equally stable and worth sharing.

## Remaining Fallback Layers

The remaining fallback matchers in `lowering` are not all the same. They should be treated in two different layers.

### Permanent runtime fallback

These should stay in `lowering` unless the product boundary itself changes.

- the outer RSI shell matchers: `match_manual_rsi_formula`, `match_rsi_rs_pair`, `match_rs_pair_from_denominator`, `match_rs_pair_expr`
- source-recovery helpers that depend on runtime interpretation: `balanced_smoothed_change_pair_source`
- change-source reconstruction helpers in `source_recovery`: `gain_loss_source_binding`, `guarded_abs_change_source`, `clamped_change_source`, `guarded_change_source`, `oriented_change_source`
- decode-side recovery helpers in `binding_sources`: `decode_smoothed_change_binding`

They stay here because they depend on runtime-facing source recovery, sign/orientation interpretation, or formula-shell confirmation rather than stable reusable parameter facts.

Known executable-contract failures should now surface as structured compile diagnostics when possible. Current examples include:

- `QPQSLOW001` for unsupported conditional `emit Intent(...)` lowering, including malformed spread-helper shapes that no longer leak generic missing-argument helper errors
- `QPQSLOW004` for unsupported runtime `Intent` actions
- `QPQSLOW007` when formal lowering cannot infer any reachable `fetch(...)` or `get_data(...)` source
- `QPQSLOW009` for unsupported rebalance `every=...` values
- `QPQSLOW010` when snapshot-dependent universe operations are used without `universe_snapshot`
- `QPQSLOW012` for unsupported universe sort orders
- `QPQSLOW013` when `rebalance(...)` is missing its allocation helper or does not receive a supported allocation helper call
- `QPQSLOW014` when a rebalance allocation helper is missing its selection input or does not receive a universe-valued selection
- `QPQSLOW015` when a rebalance allocation resolves to an empty symbol set
- `QPQSLOW016` for fixed-weight count mismatch against the selected universe
- `QPQSLOW017` for negative fixed weights
- `QPQSLOW018` for zero-total fixed weights
- `QPQSLOW019` for unsupported `rank_weight(..., method=...)` values
- `QPQSLOW020` for unsupported `score_weight(..., normalize=...)` values
- `QPQSLOW021` when `weights=...` is missing or is not a numeric list literal
- `QPQSLOW022` when indicator helpers such as `rsi`, `macd`, `momentum`, or `zscore` are missing their first argument or do not receive a fetch/get_data source there
- `QPQSLOW023` when indicator period/lookback/window arguments are missing, non-numeric, or not greater than zero
- `QPQSLOW024` when moving-average helpers are missing their source input, do not receive a fetch/get_data source, or `ema(...)` does not receive a recognized MACD line
- `QPQSLOW025` when universe helpers such as `filter/sort_by/top` are missing their universe input or do not receive a universe-valued input
- `QPQSLOW026` when `symbols(...)` is missing its list input or does not receive a list literal
- `QPQSLOW027` when `symbols([...])` contains non-string items
- `QPQSLOW028` when `top(...)` does not receive a numeric count argument
- direct single-source moving-average comparisons now validate against a shared Core IR helper first, and successful lowering emits a structured `ScalarExpr::Compare` over `SeriesExpr::WindowAgg` instead of only preserving raw condition text
- direct one-sided RSI threshold comparisons now also reuse a shared Core IR helper and emit a structured `ScalarExpr::Compare` over the lowered indicator reference plus numeric threshold; dual-sided RSI shapes still remain on the raw-text path because current runtime intent merging does not preserve two separate RSI predicates
- direct one-sided `momentum` and `zscore` threshold comparisons now also reuse the shared indicator-threshold compare helper; lowering preserves a signed comparison threshold for the one-sided path and explicitly drops the structured compare metadata again when opposite-side branches are merged into the same runtime intent

### Transitional fallback

These are still local matchers today, but they do not all have the same promotion readiness.

Promotion candidates to prioritize:

Already promoted onto a resolve-first path:

- `match_zscore_operands`
  The main path now goes through `ResolvedManualIndicatorFormula::ZScore` plus a resolve-first target/span helper. Only the older three-operand compatibility tail remains local in `lowering`.
- `match_manual_moving_average_window`
  The main path now goes through `ResolvedManualIndicatorFormula::MovingAverage` plus a resolve-first target/span helper. Only the older split `sum()/period` compatibility tail remains local in `lowering`.
- `match_sum_window_call`
  The main path now goes through `ResolvedExprSemantic::WindowAggregateView` plus a resolve-first target/span helper. Only the older capability-shaped AST fallback tail remains local in `lowering`.
- `match_latest_lookback_pair`
  The main path now goes through resolve-first semantics, preferring `ResolvedExprSemantic::BoundaryLookbackPair` and then the already-standardized `ResolvedManualIndicatorFormula::Momentum` when the pair has been promoted into a momentum formula. Only the older alias-shaped AST fallback tail remains local in `lowering`.

Keep in lowering for now:

- `match_ema_spread`
  This still depends on source orientation and runtime-facing MACD interpretation details.
- `match_macd_line_signal_pair`
  This still carries runtime-facing line/signal orientation and should not be promoted until that contract is made explicit.
- the outer RSI shell matchers
  The stable change-pair core is already partially standardized, but the final formula shell and `RsiMethod` mapping still belong to lowering today.

The promotion candidates already prefer `ResolveResult` semantics in the common case. Their remaining local logic mainly exists as a compatibility path for older, partially normalized, or alias-shaped expressions.

The rule is:

- if a matcher is only recovering a stable parameter fact, it is a promotion candidate
- if a matcher is recovering runtime source identity, sign, orientation, or final strategy meaning, it stays in `lowering`

## Admission Rules For New Resolve Semantics

New matcher results should move into `ResolveResult` only when all of the following are true.

1. The result collapses to a small, stable parameter set.
2. The result is reusable across multiple lowering call sites.
3. The result does not require full runtime context.
4. The result can be recognized with bounded alias-aware rules.
5. The result reduces repeated AST-shape matching in `lowering`.
6. The result is covered by both:
   - a resolve-level semantic annotation test
   - a lowering-level consumption or fallback regression test

If a rule fails these criteria, it stays in `lowering`.

## Practical Rule

Move stable parameter facts forward.

Do not move full runtime interpretation forward.

That is the current boundary.
