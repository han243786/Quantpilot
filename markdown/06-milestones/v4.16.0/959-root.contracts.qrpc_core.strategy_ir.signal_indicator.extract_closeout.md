# v4.16.0 root.contracts.qrpc_core.strategy_ir.signal_indicator extract closeout

> Batch: BE-001QM-02
> Node: `root.contracts.qrpc_core.strategy_ir.signal_indicator`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `extract_closeout`
> Movement: Rust code moved under the Strategy IR parent.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.signal_indicator` has been physically extracted from the Strategy IR parent into a private child module.

Moved code:

- `SignalDefinition`;
- `IndicatorDefinition`;
- `IndicatorKind`;
- `DECLARED_INDICATOR_KINDS`;
- `SUPPORTED_INDICATOR_KINDS`;
- `declared_indicator_kinds`;
- `supported_indicator_kinds`.

New child owner:

- `qrpc_core/src/strategy_ir/signal_indicator.rs`

Parent facade:

- `qrpc_core/src/strategy_ir.rs`

## Equivalence Claim

The public contract remains equivalent:

- `qrpc_core::strategy_ir::SignalDefinition`, `IndicatorDefinition`, `IndicatorKind`, `declared_indicator_kinds`, and `supported_indicator_kinds` remain exported through the Strategy IR parent;
- the same surfaces remain exported through `qrpc_core/src/lib.rs` via the existing `pub use strategy_ir::*`;
- signal/indicator fields, serde defaults, `deny_unknown_fields`, indicator variants, snake_case enum rename behavior, and declared/supported indicator ordering are unchanged;
- `indicator_kind_supported` remains in the Strategy IR parent as a root validation helper and continues calling `supported_indicator_kinds` through the parent-local public surface.

## Parent-Child Rule

Allowed dependency preserved:

- Strategy IR parent -> private `signal_indicator` child re-export;
- root Strategy IR DTO and validation -> signal/indicator DTOs and registry functions through the Strategy IR parent-local public surface.

No direct sibling path import was introduced. The signal/indicator child does not import future Strategy IR siblings, qrpc runtime/compiler modules, backend, executor, or release-transition paths.

## Files Changed

| File | Change |
| --- | --- |
| `qrpc_core/src/strategy_ir.rs` | Added private signal/indicator child module declaration and public re-export; removed signal/indicator code now owned by the child. |
| `qrpc_core/src/strategy_ir/signal_indicator.rs` | Added extracted Strategy IR signal/indicator DTOs, indicator enum, and public indicator registry functions. |

## Non-Claims

This extraction does not claim:

- indicator support behavior changed;
- root validation rules changed;
- logic/risk/data/execution/gap DTOs changed;
- tests were rewritten;
- closed error/proto/plugin contract children, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QM-03 `root.contracts.qrpc_core.strategy_ir.signal_indicator` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
