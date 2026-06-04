# v4.16.0 root.contracts.qrpc_core.strategy_ir.version_unknown_error extract closeout

> Batch: BE-001QI-02
> Node: `root.contracts.qrpc_core.strategy_ir.version_unknown_error`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `extract_closeout`
> Movement: Rust code moved under the Strategy IR parent.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.version_unknown_error` has been physically extracted from the Strategy IR parent into a private child module.

Moved code:

- `STRATEGY_IR_V0_VERSION`;
- `KnownOrUnknown<T>`;
- `KnownOrUnknown::is_unknown`;
- `StrategyIrValidationError`;
- `StrategyIrValidationError` Display implementation;
- `StrategyIrValidationError` `std::error::Error` implementation.

New child owner:

- `qrpc_core/src/strategy_ir/version_unknown_error.rs`

Parent facade:

- `qrpc_core/src/strategy_ir.rs`

## Equivalence Claim

The public contract remains equivalent:

- `qrpc_core::strategy_ir::STRATEGY_IR_V0_VERSION`, `KnownOrUnknown`, and `StrategyIrValidationError` remain exported through the Strategy IR parent;
- `qrpc_core::STRATEGY_IR_V0_VERSION`, `KnownOrUnknown`, and `StrategyIrValidationError` remain exported through `qrpc_core/src/lib.rs` via the existing `pub use strategy_ir::*`;
- `KnownOrUnknown<T>` keeps the same untagged serde shape and `is_unknown` semantics;
- `StrategyIrValidationError` keeps the same error vector, display joining behavior, and error trait implementation;
- `StrategyIr::validation_errors` and `StrategyIr::validate` continue to use the same public surfaces through the Strategy IR parent.

## Parent-Child Rule

Allowed dependency preserved:

- Strategy IR parent -> private `version_unknown_error` child re-export;
- root validation -> version constant and validation error through the Strategy IR parent-local public surface;
- DTO families -> `KnownOrUnknown<T>` through the Strategy IR parent.

No direct sibling path import was introduced. The new child does not import future Strategy IR siblings, qrpc runtime/compiler modules, backend, executor, or release-transition paths.

## Files Changed

| File | Change |
| --- | --- |
| `qrpc_core/src/strategy_ir.rs` | Added private child module declaration and public re-export; removed version/unknown/error code now owned by the child. |
| `qrpc_core/src/strategy_ir/version_unknown_error.rs` | Added extracted Strategy IR version, unknown wrapper, and validation error diagnostic owner. |

## Non-Claims

This extraction does not claim:

- Strategy IR validation rules changed;
- DTO families changed;
- indicator registries changed;
- gap annotations changed;
- tests were rewritten;
- closed error/proto/plugin contract children, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QI-03 `root.contracts.qrpc_core.strategy_ir.version_unknown_error` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
