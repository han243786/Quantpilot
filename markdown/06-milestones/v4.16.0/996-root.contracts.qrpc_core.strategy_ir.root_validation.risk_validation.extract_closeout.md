# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation extract closeout

> Batch: BE-001RE-02
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Stage: `extract_closeout`
> Movement: Rust code extraction.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` has been extracted from `root_validation.rs` into a private child module.

The root validation parent preserves the original validation order:

1. identity/readiness validation;
2. signal and logic validation;
3. risk validation;
4. data requirement validation;
5. execution validation;
6. unknown path/reason validation.

## Files Changed

- `qrpc_core/src/strategy_ir/root_validation.rs`
- `qrpc_core/src/strategy_ir/root_validation/risk_validation.rs`

## Extraction Result

`qrpc_core/src/strategy_ir/root_validation.rs` now declares:

- `mod identity_required_validation;`
- `mod risk_validation;`
- `mod signal_logic_validation;`

`StrategyIr::validation_errors` now delegates the selected block through:

- `risk_validation::validate_risk(self, &mut errors);`

The new child module owns:

- `risk_rules.max_position_ratio` unknownable validation;
- `risk_rules.stop_loss_ratio` unknownable validation;
- optional `risk_rules.take_profit_ratio` unknownable validation;
- optional `risk_rules.max_drawdown_ratio` unknownable validation;
- optional `risk_rules.max_trades_per_day` unknownable validation;
- `risk_profile.profile_id` runtime profile id validation;
- `risk_profile.max_position` finite and greater-than-zero validation;
- `risk_profile.max_total_leverage` lower-bound validation;
- `risk_profile.max_exchange_leverage` lower-bound validation.

## Rust Local Fields

| Field | Result |
| --- | --- |
| Crate | `qrpc-core` |
| Parent facade | Preserved. `root_validation.rs` remains the coordinator for module declarations and validation call order. |
| Child visibility | Preserved as `pub(super)` for the child entry function; no external public API was added. |
| Public exports | No `pub use` changes. Existing `qrpc_core::strategy_ir::*` and `qrpc_core::*` surfaces remain unchanged. |
| Parent-owned helpers | `validate_unknownable` and `validate_unknownable_opt` remain parent-owned. |
| Sibling dependency check | Passed. `risk_validation` does not import sibling validation children. |

## Equivalence Preservation

| Surface | Preserved evidence |
| --- | --- |
| Validation ordering | The parent call remains immediately after `signal_logic_validation` and before data requirement validation. |
| Risk diagnostics | Message text and path labels moved unchanged into the child module. |
| Unknown marker diagnostics | The child calls the same parent-owned `validate_unknownable` and `validate_unknownable_opt` helpers. |
| Root public API | `StrategyIr::validation_errors` and `StrategyIr::validate` signatures are unchanged. |
| Serde shape | No DTO fields, derives, or serde attributes changed. |

## Hard Boundaries Held

BE-001RE-02 did not:

- change validation ordering, diagnostics, path labels, root DTO fields, public validation methods, serde shape, or local test expectations;
- move data requirement validation, execution validation, unknowns path/reason validation, local tests, or sample fixture;
- move or rename `validate_unknownable` or `validate_unknownable_opt`;
- make root validation child modules import each other directly;
- widen visibility beyond `pub(super)`;
- change closed Strategy IR child DTOs/enums/constants/public functions, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Gates

- `cargo fmt --check`
- `cargo check -p qrpc-core`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-pre-commit-hook.ps1`

## Next Step

BE-001RE-03 `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` single_leaf_closeout.
