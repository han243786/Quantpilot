# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation parent residual judgment selects signal_logic_validation

> Batch: BE-001RB-01
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Selected child: `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.root_validation` returns to its child queue after `identity_required_validation` closeout.

Decision:

`next_child: root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation`

## Closed Root Validation Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` | Closed with `stop_split: true`; owns identity/readiness validation and unique id helper behavior. |

## Open Root Validation Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` | Selected next. Owns signal detail validation, indicator support validation, logic rule validation, and logic position unknown-marker checks through parent-mediated helpers. |
| `contracts.qrpc_core.strategy_ir.root_validation.risk_validation` | Queued. Owns risk unknownable checks and risk profile numeric/id validation. |
| `contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` | Queued. Owns data requirement and execution profile validation. |
| `contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` | Queued. Owns unknown marker helper family and `unknowns[*]` path/reason validation. |
| `contracts.qrpc_core.strategy_ir.root_validation.test_fixture` | Conditional. Only split if the remaining local test fixture still creates parent-sized pressure. |

## Selection Rationale

`signal_logic_validation` is selected next because it is the next contiguous validation block after identity/readiness validation:

- it owns the signal loop, indicator support check, and logic rule loops;
- it can preserve validation ordering by replacing the existing contiguous block with one parent-mediated helper call;
- it will need controlled access to `validate_logic_rule`, `indicator_kind_supported`, and `validate_unknownable`;
- it must not reach across to risk/data/execution/unknown-marker sibling children directly.

## Hard Boundaries

The next `signal_logic_validation` baseline must not:

- edit Rust source code;
- change signal diagnostics, logic diagnostics, indicator support behavior, unknown-marker diagnostics, validation ordering, or test expectations;
- move risk/data/execution validation, unknowns path/reason validation, public methods, root DTO fields, or tests;
- make root validation child modules import each other directly;
- change closed Strategy IR child DTOs/enums/constants/public functions, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Next Step

BE-001RC-01 `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
