# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation baseline plan

> Batch: BE-001RG-01
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

This baseline freezes the next Strategy IR root validation child before extraction.

The next implementation step may move only data requirement checks plus execution and execution profile validation into a private child module under `root_validation.rs`.

The already closed identity, signal/logic, and risk validation children remain authoritative. This baseline narrows the next safe movement without changing Rust source code.

## Frozen Surface

The selected child owns the current contiguous validation block in `qrpc_core/src/strategy_ir/root_validation.rs` immediately after `risk_validation::validate_risk(self, &mut errors);` and before the `unknowns` loop:

- loop over `self.data_requirements`;
- `data_requirements[{index}].data_id` required-field validation;
- `data_requirements[{index}].fields` required-field validation;
- `data_requirements[{index}].venue` unknownable check;
- `data_requirements[{index}].symbol` unknownable check;
- `data_requirements[{index}].granularity` unknownable check;
- `data_requirements[{index}].lookback` unknownable check;
- `execution.venue_type` unknownable check;
- `execution.order_type` unknownable check;
- `execution.slippage_model` unknownable check;
- optional `execution.time_in_force` unknownable check;
- optional `execution.latency_assumption_ms` unknownable check;
- optional `execution.capital_base` unknownable check;
- `execution_profile.profile_id` must be `paper` diagnostic;
- `execution_profile.fee_bps` finite and non-negative diagnostic;
- `execution_profile.slippage_bps` finite and non-negative diagnostic.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Validation ordering | This child must run immediately after `risk_validation` and before `unknowns[*]` path/reason validation. |
| Data paths | Data path labels remain exactly `data_requirements[{index}].data_id`, `fields`, `venue`, `symbol`, `granularity`, and `lookback`. |
| Execution paths | Execution path labels remain exactly `execution.venue_type`, `execution.order_type`, `execution.slippage_model`, `execution.time_in_force`, `execution.latency_assumption_ms`, and `execution.capital_base`. |
| Execution profile id | The current runtime still accepts only `execution_profile.profile_id == "paper"`. |
| Execution profile numeric checks | `fee_bps` and `slippage_bps` must stay finite and non-negative when present. |
| Unknown marker diagnostics | Data/execution unknownable diagnostics continue to use the parent-owned `validate_unknownable` and `validate_unknownable_opt` helper family. |
| Root public API | `StrategyIr::validation_errors` and `StrategyIr::validate` public behavior remains unchanged. |
| Serde shape | Root `StrategyIr` fields and closed Strategy IR child DTO serde behavior remain unchanged. |

## Planned Extraction Shape

BE-001RG-02 may:

- create `qrpc_core/src/strategy_ir/root_validation/data_execution_validation.rs`;
- add `mod data_execution_validation;` to `qrpc_core/src/strategy_ir/root_validation.rs`;
- replace the selected inline block with one parent-mediated helper call, tentatively `data_execution_validation::validate_data_and_execution(self, &mut errors);`;
- keep `validate_unknownable` and `validate_unknownable_opt` parent-owned until the remaining unknown marker validation child closes and helper ownership can be judged from evidence;
- keep child visibility at `pub(super)`.

## Rust Local Refactor Fields

| Field | Baseline |
| --- | --- |
| Crate | `qrpc-core` |
| Parent facade | `qrpc_core/src/strategy_ir/root_validation.rs` remains the coordinator for child module declarations and validation call order. |
| Child visibility | Child entry function should be `pub(super)`; no external public API is created. |
| Public exports | No `pub use` change is expected for this child; `qrpc_core::strategy_ir::*` and `qrpc_core::*` remain preserved through existing parent exports. |
| Parent-owned helpers | `validate_unknownable` and `validate_unknownable_opt` remain parent-owned for queued sibling residuals. |
| Sibling dependency rule | `data_execution_validation` must not import `identity_required_validation`, `signal_logic_validation`, `risk_validation`, `unknown_marker_validation`, or test fixture children. |
| Cargo gates | `cargo fmt --check`, `cargo check -p qrpc-core`, and `cargo test -p qrpc-core` are the Rust minimum gates for extraction. |

## Same-Parent Parallel Wave Judgment

This child remains a single-child extraction for BE-001RG-02.

`same_parent_parallel_children` is not activated here because `data_execution_validation` and queued `unknown_marker_validation` still touch parent-owned unknown marker helper behavior or validation ordering. The safer high-speed path is one child extraction with a parent facade lock.

## Hard Boundaries

BE-001RG-02 must not:

- change validation ordering, diagnostics, path labels, root DTO fields, public validation methods, serde shape, or local test expectations;
- move unknowns path/reason validation, local tests, or sample fixture;
- move or rename `validate_unknownable` or `validate_unknownable_opt`;
- make root validation child modules import each other directly;
- widen visibility beyond `pub(super)` unless a compile-proven parent boundary requires it and the widening is documented;
- change closed Strategy IR child DTOs/enums/constants/public functions, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Split Rule Pre-Check

| Rule | Baseline result |
| --- | --- |
| Physical owner | Worth extracting. The selected slice is a coherent data/execution validation family and currently occupies a contiguous validation region. |
| Public surface | Stop after extraction unless this child gains multiple public helper families; it should expose only parent-local validation behavior. |
| State ownership | Stop after extraction. The child is stateless and appends diagnostics through the parent-provided `errors` vector. |
| Side effects | Stop after extraction. No IO, persistence, locks, runtime state, or external side effects are involved. |
| Parent-mediated dependency | Required. The child receives `StrategyIr` through the root validation parent and may use only parent-owned helper surfaces. |
| Shared helper pressure | `validate_unknownable` and `validate_unknownable_opt` remain parent-owned until unknown marker validation closes and helper ownership can be judged separately. |
| Future reopen rule | Allowed only for concrete data requirement, execution unknownable, or execution profile validation proposals. |

## Next Step

BE-001RG-02 `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p qrpc-core`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-pre-commit-hook.ps1`
