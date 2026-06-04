# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation parent residual judgment selects identity_required_validation

> Batch: BE-001QZ-01
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Selected child: `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.root_validation` begins its child split after BE-001QY-03 determined that it is still parent-sized.

Decision:

`next_child: root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation`

## Open Root Validation Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` | Selected next. Owns version rule, metadata required fields, top-level required collections, unique id checks, and `validate_unique_ids`. |
| `contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` | Queued. Owns signal validation, indicator support validation, and logic rule validation. |
| `contracts.qrpc_core.strategy_ir.root_validation.risk_validation` | Queued. Owns risk unknownable checks and risk profile numeric/id validation. |
| `contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` | Queued. Owns data requirement and execution profile validation. |
| `contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` | Queued. Owns unknown marker helper family and `unknowns[*]` path/reason validation. |
| `contracts.qrpc_core.strategy_ir.root_validation.test_fixture` | Conditional. Only split if the remaining local test fixture still creates parent-sized pressure. |

## Selection Rationale

`identity_required_validation` is selected first because it is independent and low blast-radius:

- it reads only root `StrategyIr` identity, metadata, collection emptiness, and id strings;
- it can preserve validation ordering by replacing the first validation block with one parent-mediated helper call;
- it does not need the unknownable helper family;
- it can move before signal/logic/risk/data/execution validation without changing public behavior.

## Hard Boundaries

The next `identity_required_validation` baseline must not:

- edit Rust source code;
- change validation ordering, diagnostics, duplicate-id labels, or required-field semantics;
- change `StrategyIr` fields, serde attributes, public validation methods, local tests, or closed Strategy IR child DTOs;
- make root validation child modules import each other directly;
- change protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Next Step

BE-001RA-01 `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
