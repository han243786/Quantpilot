# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation baseline plan

> Batch: BE-001RC-01
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

This baseline freezes the next Strategy IR root validation child before extraction.

The next implementation step may move only signal detail validation, indicator support validation, logic rule validation, and logic position unknown-marker checks into a private child module under `root_validation.rs`.

This step also confirms the local Rust refactor acceleration asset can attach to the existing recursive governance flow without invalidating earlier closeouts: previous Strategy IR child closeouts remain authoritative, while this baseline adds Rust-specific facade, visibility, and Cargo gate fields for the next extraction.

## Frozen Surface

The selected child owns the current contiguous validation block in `qrpc_core/src/strategy_ir/root_validation.rs` immediately after `identity_required_validation::validate_identity_and_required_fields(self, &mut errors);`:

- signal detail loop over `self.signals`;
- `signals[{index}].signal_id 是必需的`;
- `signals[{index}].name 是必需的`;
- `signals[{index}].indicator.inputs 必须包含至少一个输入`;
- `signals[{index}].indicator.inputs 对于 spread 必须包含至少两个输入`;
- unsupported indicator diagnostic `signals[{index}].indicator.kind {:?} 不被当前运行时支持`;
- logic entry rule loop over `self.logic.entry_rules`;
- logic exit rule loop over `self.logic.exit_rules`;
- `logic.entry_rules[{index}].rule_id 是必需的`;
- `logic.entry_rules[{index}].condition 是必需的`;
- `logic.exit_rules[{index}].rule_id 是必需的`;
- `logic.exit_rules[{index}].condition 是必需的`;
- `logic.position_sizing.value` unknown marker check;
- optional `logic.rebalance_rule.frequency` unknown marker check.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Validation ordering | This child must run immediately after `identity_required_validation` and before risk validation. |
| Signal diagnostics | All signal required-field, input-count, spread-input, and unsupported indicator messages remain unchanged. |
| Indicator support | `indicator_kind_supported` behavior continues to use `supported_indicator_kinds().contains(kind)`. |
| Logic diagnostics | Logic rule path labels remain exactly `logic.entry_rules[{index}]` and `logic.exit_rules[{index}]`. |
| Unknown marker diagnostics | Logic unknownable diagnostics continue to use `{path} 未知标记必须为 "unknown"` through the parent-owned helper. |
| Root public API | `StrategyIr::validation_errors` and `StrategyIr::validate` public behavior remains unchanged. |
| Serde shape | Root `StrategyIr` fields and closed Strategy IR child DTO serde behavior remain unchanged. |

## Planned Extraction Shape

BE-001RC-02 may:

- create `qrpc_core/src/strategy_ir/root_validation/signal_logic_validation.rs`;
- add `mod signal_logic_validation;` to `qrpc_core/src/strategy_ir/root_validation.rs`;
- replace the selected inline block with one parent-mediated helper call, tentatively `signal_logic_validation::validate_signal_and_logic(self, &mut errors);`;
- move `validate_logic_rule` into the child if it is used only by this child after extraction;
- keep `indicator_kind_supported` parent-owned unless the extraction proves it is used only by this child and moving it does not create sibling pressure;
- keep `validate_unknownable` parent-owned because risk, data, and execution validation residuals also depend on the same helper family.

## Rust Local Refactor Fields

| Field | Baseline |
| --- | --- |
| Crate | `qrpc-core` |
| Parent facade | `qrpc_core/src/strategy_ir/root_validation.rs` remains the coordinator for child module declarations and validation call order. |
| Child visibility | Child entry function should be `pub(super)`; no external public API is created. |
| Public exports | No `pub use` change is expected for this child; `qrpc_core::strategy_ir::*` and `qrpc_core::*` remain preserved through existing parent exports. |
| Parent-owned helpers | `validate_unknownable` remains parent-owned; `indicator_kind_supported` is parent-owned unless BE-001RC-02 proves local-only ownership. |
| Sibling dependency rule | `signal_logic_validation` must not import `identity_required_validation`, `risk_validation`, `data_execution_validation`, `unknown_marker_validation`, or test fixture children. |
| Cargo gates | `cargo fmt --check`, `cargo check -p qrpc-core`, and `cargo test -p qrpc-core` are the Rust minimum gates for extraction. |

## Same-Parent Parallel Wave Judgment

This child remains a single-child extraction for BE-001RC-02.

`same_parent_parallel_children` is not activated here because `signal_logic_validation` and the queued `risk_validation` / `data_execution_validation` children all touch the same parent-owned unknownable helper family and parent validation call order. The Rust child file can be prepared cleanly, but parent facade ordering should be advanced one child at a time until helper ownership is stable.

## Hard Boundaries

BE-001RC-02 must not:

- change validation ordering, diagnostics, path labels, indicator support behavior, unknown marker behavior, root DTO fields, public validation methods, serde shape, or local test expectations;
- move risk validation, data requirement validation, execution validation, unknowns path/reason validation, local tests, or sample fixture;
- make root validation child modules import each other directly;
- widen visibility beyond `pub(super)` unless a compile-proven parent boundary requires it and the widening is documented;
- change closed Strategy IR child DTOs/enums/constants/public functions, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Split Rule Pre-Check

| Rule | Baseline result |
| --- | --- |
| Physical owner | Worth extracting. The selected slice is a coherent signal/logic validation family and currently occupies a contiguous validation region. |
| Public surface | Stop after extraction unless this child gains multiple public helper families; it should expose only parent-local validation behavior. |
| State ownership | Stop after extraction. The child is stateless and appends diagnostics through the parent-provided `errors` vector. |
| Side effects | Stop after extraction. No IO, persistence, locks, runtime state, or external side effects are involved. |
| Parent-mediated dependency | Required. The child receives `StrategyIr` through the root validation parent and may use only parent-owned helper surfaces. |
| Shared helper pressure | `validate_unknownable` remains parent-owned until risk/data/execution children close and helper ownership can be judged separately. |
| Future reopen rule | Allowed only for concrete signal detail, indicator support, logic rule, or logic unknown-marker validation proposals. |

## Next Step

BE-001RC-02 `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p qrpc-core`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`

