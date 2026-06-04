# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation extract closeout

> Batch: BE-001RC-02
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Stage: `extract_closeout`
> Movement: Rust code extraction.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` has been extracted from `root_validation.rs` into a private child module.

The root validation parent preserves the original validation order:

1. identity/readiness validation;
2. signal and logic validation;
3. risk validation;
4. data requirement validation;
5. execution validation;
6. unknown path/reason validation.

## Files Changed

- `qrpc_core/src/strategy_ir/root_validation.rs`
- `qrpc_core/src/strategy_ir/root_validation/signal_logic_validation.rs`

## Extraction Result

`qrpc_core/src/strategy_ir/root_validation.rs` now declares:

- `mod identity_required_validation;`
- `mod signal_logic_validation;`

`StrategyIr::validation_errors` now delegates the selected block through:

- `signal_logic_validation::validate_signal_and_logic(self, &mut errors);`

The new child module owns:

- signal detail validation;
- indicator support validation through the parent-owned `indicator_kind_supported`;
- logic entry/exit rule validation;
- private `validate_logic_rule`;
- logic position sizing and rebalance unknown marker checks through the parent-owned `validate_unknownable`.

## Rust Local Fields

| Field | Result |
| --- | --- |
| Crate | `qrpc-core` |
| Parent facade | Preserved. `root_validation.rs` remains the coordinator for module declarations and validation call order. |
| Child visibility | Preserved as `pub(super)` for the child entry function; no external public API was added. |
| Public exports | No `pub use` changes. Existing `qrpc_core::strategy_ir::*` and `qrpc_core::*` surfaces remain unchanged. |
| Parent-owned helpers | `indicator_kind_supported` and `validate_unknownable` remain parent-owned. |
| Moved helper | `validate_logic_rule` moved into `signal_logic_validation` because it is local to this child. |
| Sibling dependency check | Passed. `signal_logic_validation` does not import sibling validation children. |

## Equivalence Preservation

| Surface | Preserved evidence |
| --- | --- |
| Validation ordering | The parent call remains immediately after `identity_required_validation` and before risk validation. |
| Signal diagnostics | Message text and path labels moved unchanged into the child module. |
| Logic diagnostics | `logic.entry_rules[{index}]` and `logic.exit_rules[{index}]` path labels moved unchanged. |
| Unknown marker diagnostics | The child calls the same parent-owned `validate_unknownable` helper. |
| Root public API | `StrategyIr::validation_errors` and `StrategyIr::validate` signatures are unchanged. |
| Serde shape | No DTO fields, derives, or serde attributes changed. |

## Hard Boundaries Held

BE-001RC-02 did not:

- change validation ordering, diagnostics, path labels, indicator support behavior, unknown marker behavior, root DTO fields, public validation methods, serde shape, or local test expectations;
- move risk validation, data requirement validation, execution validation, unknowns path/reason validation, local tests, or sample fixture;
- make root validation child modules import each other directly;
- widen visibility beyond `pub(super)`;
- change closed Strategy IR child DTOs/enums/constants/public functions, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Gates

- `cargo fmt --check`
- `cargo check -p qrpc-core`

BE-001RC-03 must run the full closeout gate set, including `cargo test -p qrpc-core` and governance checks.

## Next Step

BE-001RC-03 `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` single_leaf_closeout.

