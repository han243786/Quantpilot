# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation parent residual judgment selects data_execution_validation

> Batch: BE-001RF-01
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Selected child: `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.root_validation` returns to its child queue after `risk_validation` closeout.

Decision:

`next_child: root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation`

## Closed Root Validation Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` | Closed with `stop_split: true`; owns identity/readiness validation and unique id helper behavior. |
| `contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` | Closed with `stop_split: true`; owns signal/detail, indicator support, logic rule, and logic unknown-marker validation. |
| `contracts.qrpc_core.strategy_ir.root_validation.risk_validation` | Closed with `stop_split: true`; owns risk unknownable checks and risk profile id/numeric validation. |

## Open Root Validation Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` | Selected next. Owns data requirement checks plus execution and execution profile validation through parent-mediated helpers. |
| `contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` | Queued. Owns unknown marker helper family and `unknowns[*]` path/reason validation. |
| `contracts.qrpc_core.strategy_ir.root_validation.test_fixture` | Conditional. Only split if the remaining local test fixture still creates parent-sized pressure. |

## Selection Rationale

`data_execution_validation` is selected next because it is the next contiguous validation region after risk validation:

- it owns `data_requirements[{index}].data_id` required-field validation;
- it owns `data_requirements[{index}].fields` required-field validation;
- it owns `data_requirements[{index}].venue`, `symbol`, `granularity`, and `lookback` unknownable checks;
- it owns `execution.venue_type`, `execution.order_type`, and `execution.slippage_model` unknownable checks;
- it owns optional `execution.time_in_force`, `execution.latency_assumption_ms`, and `execution.capital_base` unknownable checks;
- it owns `execution_profile.profile_id` validation against `paper`;
- it owns execution profile numeric validation for `fee_bps` and `slippage_bps`;
- it can preserve validation ordering by replacing the existing contiguous data/execution block with one parent-mediated helper call;
- it should keep `validate_unknownable` and `validate_unknownable_opt` parent-owned because queued unknown marker validation still needs helper ownership to be judged from evidence.

## Same-Parent Parallel Judgment

`same_parent_parallel_children` remains inactive for this selection.

`data_execution_validation` and `unknown_marker_validation` both touch parent-owned unknown marker helper behavior or validation order. The safer high-speed path is still single-child extraction with parent helper mediation until the remaining validation families close and helper ownership can be judged from evidence.

## Hard Boundaries

The next `data_execution_validation` baseline must not:

- edit Rust source code;
- change data diagnostics, execution diagnostics, execution profile numeric behavior, unknown-marker diagnostics, validation ordering, or test expectations;
- move unknowns path/reason validation, local tests, public methods, root DTO fields, or closed sibling validation children;
- make root validation child modules import each other directly;
- change closed Strategy IR child DTOs/enums/constants/public functions, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Next Step

BE-001RG-01 `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p qrpc-core`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
