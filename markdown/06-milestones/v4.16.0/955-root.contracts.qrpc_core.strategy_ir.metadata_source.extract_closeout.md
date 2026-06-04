# v4.16.0 root.contracts.qrpc_core.strategy_ir.metadata_source extract closeout

> Batch: BE-001QK-02
> Node: `root.contracts.qrpc_core.strategy_ir.metadata_source`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `extract_closeout`
> Movement: Rust code moved under the Strategy IR parent.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.metadata_source` has been physically extracted from the Strategy IR parent into a private child module.

Moved code:

- `StrategyMetadata`;
- `StrategySource`;
- `StrategySourceType`.

New child owner:

- `qrpc_core/src/strategy_ir/metadata_source.rs`

Parent facade:

- `qrpc_core/src/strategy_ir.rs`

## Equivalence Claim

The public contract remains equivalent:

- `qrpc_core::strategy_ir::StrategyMetadata`, `StrategySource`, and `StrategySourceType` remain exported through the Strategy IR parent;
- `qrpc_core::StrategyMetadata`, `StrategySource`, and `StrategySourceType` remain exported through `qrpc_core/src/lib.rs` via the existing `pub use strategy_ir::*`;
- metadata/source field names, serde defaults, `deny_unknown_fields`, source type variants, and snake_case enum rename behavior are unchanged;
- `StrategyIr` continues embedding `StrategyMetadata`;
- validation reads metadata fields through the same public parent-local surface.

## Parent-Child Rule

Allowed dependency preserved:

- Strategy IR parent -> private `metadata_source` child re-export;
- root Strategy IR DTO and validation -> metadata/source DTOs through the Strategy IR parent-local public surface.

No direct sibling path import was introduced. The metadata/source child does not import future Strategy IR siblings, qrpc runtime/compiler modules, backend, executor, or release-transition paths.

## Files Changed

| File | Change |
| --- | --- |
| `qrpc_core/src/strategy_ir.rs` | Added private metadata/source child module declaration and public re-export; removed metadata/source DTO code now owned by the child. |
| `qrpc_core/src/strategy_ir/metadata_source.rs` | Added extracted Strategy IR metadata/source DTO owner. |

## Non-Claims

This extraction does not claim:

- metadata/source serde shape changed;
- root validation rules changed;
- signal/indicator, logic/risk/data/execution/gap DTOs changed;
- tests were rewritten;
- closed error/proto/plugin contract children, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QK-03 `root.contracts.qrpc_core.strategy_ir.metadata_source` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
