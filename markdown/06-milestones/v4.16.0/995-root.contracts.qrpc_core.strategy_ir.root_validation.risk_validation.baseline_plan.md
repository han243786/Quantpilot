# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation baseline plan

> Batch: BE-001RE-01
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

This baseline freezes the next Strategy IR root validation child before extraction.

The next implementation step may move only risk unknownable checks and risk profile id/numeric validation into a private child module under `root_validation.rs`.

This step keeps the already closed identity and signal/logic validation children authoritative. The new Rust-local acceleration plan remains an execution layer on top of the existing recursive governance flow: earlier closeouts stay frozen, while this baseline narrows the next safe movement.

## Frozen Surface

The selected child owns the current contiguous validation block in `qrpc_core/src/strategy_ir/root_validation.rs` immediately after `signal_logic_validation::validate_signal_and_logic(self, &mut errors);`:

- `risk_rules.max_position_ratio` unknownable check;
- `risk_rules.stop_loss_ratio` unknownable check;
- optional `risk_rules.take_profit_ratio` unknownable check;
- optional `risk_rules.max_drawdown_ratio` unknownable check;
- optional `risk_rules.max_trades_per_day` unknownable check;
- `risk_profile.profile_id` must be `global` diagnostic;
- `risk_profile.max_position` finite and greater-than-zero diagnostic;
- `risk_profile.max_total_leverage` greater-than-or-equal-to-one diagnostic;
- `risk_profile.max_exchange_leverage` greater-than-or-equal-to-one diagnostic.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Validation ordering | This child must run immediately after `signal_logic_validation` and before data requirement validation. |
| Risk unknown markers | Risk unknownable diagnostics continue to use the parent-owned `validate_unknownable` and `validate_unknownable_opt` helper family. |
| Risk paths | Risk path labels remain exactly `risk_rules.max_position_ratio`, `risk_rules.stop_loss_ratio`, `risk_rules.take_profit_ratio`, `risk_rules.max_drawdown_ratio`, and `risk_rules.max_trades_per_day`. |
| Risk profile id | The current runtime still accepts only `risk_profile.profile_id == "global"`. |
| Risk profile numeric checks | `max_position` must stay finite and greater than zero; leverage fields must stay greater than or equal to one. |
| Root public API | `StrategyIr::validation_errors` and `StrategyIr::validate` public behavior remains unchanged. |
| Serde shape | Root `StrategyIr` fields and closed Strategy IR child DTO serde behavior remain unchanged. |

## Planned Extraction Shape

BE-001RE-02 may:

- create `qrpc_core/src/strategy_ir/root_validation/risk_validation.rs`;
- add `mod risk_validation;` to `qrpc_core/src/strategy_ir/root_validation.rs`;
- replace the selected inline block with one parent-mediated helper call, tentatively `risk_validation::validate_risk(self, &mut errors);`;
- keep `validate_unknownable` and `validate_unknownable_opt` parent-owned because data requirement validation and execution validation still depend on the same helper family;
- keep child visibility at `pub(super)`.

## Rust Local Refactor Fields

| Field | Baseline |
| --- | --- |
| Crate | `qrpc-core` |
| Parent facade | `qrpc_core/src/strategy_ir/root_validation.rs` remains the coordinator for child module declarations and validation call order. |
| Child visibility | Child entry function should be `pub(super)`; no external public API is created. |
| Public exports | No `pub use` change is expected for this child; `qrpc_core::strategy_ir::*` and `qrpc_core::*` remain preserved through existing parent exports. |
| Parent-owned helpers | `validate_unknownable` and `validate_unknownable_opt` remain parent-owned for sibling residuals. |
| Sibling dependency rule | `risk_validation` must not import `identity_required_validation`, `signal_logic_validation`, `data_execution_validation`, `unknown_marker_validation`, or test fixture children. |
| Cargo gates | `cargo fmt --check`, `cargo check -p qrpc-core`, and `cargo test -p qrpc-core` are the Rust minimum gates for extraction. |

## Same-Parent Parallel Wave Judgment

This child remains a single-child extraction for BE-001RE-02.

`same_parent_parallel_children` is not activated here because `risk_validation`, queued `data_execution_validation`, and queued `unknown_marker_validation` still touch parent-owned unknown marker helper behavior or validation ordering. The safer high-speed path is one child extraction with a parent facade lock.

## Hard Boundaries

BE-001RE-02 must not:

- change validation ordering, diagnostics, path labels, root DTO fields, public validation methods, serde shape, or local test expectations;
- move data requirement validation, execution validation, unknowns path/reason validation, local tests, or sample fixture;
- move or rename `validate_unknownable` or `validate_unknownable_opt`;
- make root validation child modules import each other directly;
- widen visibility beyond `pub(super)` unless a compile-proven parent boundary requires it and the widening is documented;
- change closed Strategy IR child DTOs/enums/constants/public functions, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Split Rule Pre-Check

| Rule | Baseline result |
| --- | --- |
| Physical owner | Worth extracting. The selected slice is a coherent risk validation family and currently occupies a contiguous validation region. |
| Public surface | Stop after extraction unless this child gains multiple public helper families; it should expose only parent-local validation behavior. |
| State ownership | Stop after extraction. The child is stateless and appends diagnostics through the parent-provided `errors` vector. |
| Side effects | Stop after extraction. No IO, persistence, locks, runtime state, or external side effects are involved. |
| Parent-mediated dependency | Required. The child receives `StrategyIr` through the root validation parent and may use only parent-owned helper surfaces. |
| Shared helper pressure | `validate_unknownable` and `validate_unknownable_opt` remain parent-owned until data/execution/unknown marker children close and helper ownership can be judged separately. |
| Future reopen rule | Allowed only for concrete risk unknownable or risk profile validation proposals. |

## Next Step

BE-001RE-02 `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p qrpc-core`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-pre-commit-hook.ps1`
