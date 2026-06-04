# v4.16.0 root.contracts.qrpc_core.strategy_ir.version_unknown_error baseline plan

> Batch: BE-001QI-01
> Node: `root.contracts.qrpc_core.strategy_ir.version_unknown_error`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.version_unknown_error` is frozen as the Strategy IR identity, unknown-preservation, and validation diagnostic owner after BE-001QH-01 selection.

BE-001QI-01 does not move code. It defines the exact baseline and allowed movement for BE-001QI-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/strategy_ir.rs`

Current selected boundary:

- `STRATEGY_IR_V0_VERSION`;
- `KnownOrUnknown<T>`;
- `KnownOrUnknown::is_unknown`;
- `StrategyIrValidationError`;
- `StrategyIrValidationError` `Display` implementation;
- `StrategyIrValidationError` `std::error::Error` implementation.

Current parent callers:

- `StrategyIr::validation_errors` compares `ir_version` against `STRATEGY_IR_V0_VERSION`;
- `StrategyIr::validate` returns `StrategyIrValidationError`;
- multiple Strategy IR DTOs use `KnownOrUnknown<T>`;
- tests call `KnownOrUnknown::is_unknown` and expect validation errors through `StrategyIr::validate`.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Stop after extraction. The child owns one compact helper method, `KnownOrUnknown::is_unknown`. |
| Mixed responsibility | Stop after extraction. Version identity, unknown preservation, and validation diagnostics are a small shared identity surface. |
| Physical owner | Continue now. The selected surface still lives in `qrpc_core/src/strategy_ir.rs` and should be isolated. |
| Private helper pressure | Defer. Splitting the error type away from version/unknown identity would add ceremony without clearer ownership. |
| Future reopen rule | Allowed only when a concrete Strategy IR version string, unknown marker shape, `is_unknown` semantics, validation error carrier, or display/error behavior change is proposed. |

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Version constant | `STRATEGY_IR_V0_VERSION` remains `strategy_ir/v0`. |
| Unknown wrapper serde | `KnownOrUnknown<T>` keeps untagged serde shape and `Known(T)` / `Unknown(String)` variants. |
| Unknown helper | `KnownOrUnknown::is_unknown` continues to match only `Unknown(_)`. |
| Validation error carrier | `StrategyIrValidationError` keeps `errors: Vec<String>` and derives `Debug`, `Clone`, and `PartialEq`. |
| Display behavior | Display keeps joining errors with `; `. |
| Error trait | `StrategyIrValidationError` continues implementing `std::error::Error`. |

## Allowed BE-001QI-02 Movement

BE-001QI-02 may:

- create `qrpc_core/src/strategy_ir/version_unknown_error.rs`;
- add a private `mod version_unknown_error;` declaration in `qrpc_core/src/strategy_ir.rs`;
- re-export the selected child surface from the Strategy IR parent with `pub use version_unknown_error::*;`;
- move only `STRATEGY_IR_V0_VERSION`, `KnownOrUnknown<T>`, `KnownOrUnknown::is_unknown`, `StrategyIrValidationError`, and the error Display/Error impls into the child module;
- move the `std::fmt` import into the child if it is no longer needed by the parent;
- keep all public imports from `qrpc_core::strategy_ir::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001QI-02 Movement

BE-001QI-02 must not move or rewrite:

- `StrategyIr`, `StrategyIr::validation_errors`, or `StrategyIr::validate`;
- Strategy IR DTO families, validation rules, indicator lists, gap annotations, private validation helpers, or tests;
- closed error/proto/plugin contract children;
- `qrpc_core/src/lib.rs`, `qrpc_core_ir`, `qrpc_compiler`, `qrpc_runtime`, `quantscript`, backend, executor, frontend, physical `plugins/*`, or release transition.

## Parent-Child Rule

Allowed call paths:

- Strategy IR parent -> private `version_unknown_error` child re-export;
- Strategy IR parent validation -> `STRATEGY_IR_V0_VERSION` and `StrategyIrValidationError` through the parent-local public surface;
- DTO children and root validation -> `KnownOrUnknown<T>` through the Strategy IR parent.

Forbidden call paths:

Any child import from future Strategy IR sibling modules, qrpc runtime/compiler modules, backend, executor, or release-transition paths that bypasses the Strategy IR parent.

## Proof

BE-001QI-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001QI-02 `root.contracts.qrpc_core.strategy_ir.version_unknown_error` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
