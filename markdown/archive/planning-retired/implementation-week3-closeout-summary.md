# Week 3 Closeout Summary

This appendix records the current Week 3 closeout state for Typed HIR, `resolve`, and `lowering`.

It is meant to be read together with:

- [implementation-weekly-execution-plan.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/planning-retired/implementation-weekly-execution-plan.md)
- [implementation-final-execution-blueprint.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/planning-retired/implementation-final-execution-blueprint.md)

## Promoted Semantics

The following stable semantics have already been promoted into `ResolveResult`:

- `SeriesView`
- `WindowAggregateView`
- `BoundaryLookbackPair`
- `BalancedSmoothedChangePair`
- `ManualIndicatorFormula::Momentum`
- `ManualIndicatorFormula::MovingAverage`
- `ManualIndicatorFormula::ZScore`
- `ManualIndicatorFormula::MacdLine`
- `ManualIndicatorFormula::MacdSignal`
- `ManualIndicatorFormula::MacdHistogram`

## Permanent Fallback

The following logic should remain in `lowering` as permanent runtime-facing fallback unless the product boundary itself changes:

- `match_manual_rsi_formula`
- `match_rsi_rs_pair`
- `match_rs_pair_from_denominator`
- `match_rs_pair_expr`
- `balanced_smoothed_change_pair_source`
- `decode_smoothed_change_binding`
- `gain_loss_source_binding`
- `guarded_abs_change_source`
- `clamped_change_source`
- `guarded_change_source`
- `oriented_change_source`

These stay in `lowering` because they recover runtime source identity, change orientation, sign constraints, or final strategy meaning instead of only recovering stable parameter facts.

## Transitional Fallback

The following matcher layer is still local fallback today, but remains a valid future promotion target if source recovery can also be standardized cleanly:

- `match_sum_window_call`
- `match_latest_lookback_pair`
- `match_ema_spread`
- `match_macd_line_signal_pair`
- `match_zscore_operands`
- `match_manual_moving_average_window`

These are already partially de-risked because the common path prefers `ResolveResult` semantics first.

## Rule

Use this rule when deciding whether a matcher should move again:

- If it only recovers a stable parameter fact, it remains a promotion candidate.
- If it recovers runtime source identity, sign, orientation, or final strategy meaning, it stays in `lowering`.

