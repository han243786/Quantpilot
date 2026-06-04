# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation parent residual judgment selects risk_validation

> Batch: BE-001RD-01
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Selected child: `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.root_validation` returns to its child queue after `signal_logic_validation` closeout.

Decision:

`next_child: root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation`

## Closed Root Validation Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` | Closed with `stop_split: true`; owns identity/readiness validation and unique id helper behavior. |
| `contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` | Closed with `stop_split: true`; owns signal/detail, indicator support, logic rule, and logic unknown-marker validation. |

## Open Root Validation Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.root_validation.risk_validation` | Selected next. Owns risk unknownable checks and risk profile id/numeric validation through parent-mediated helpers. |
| `contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` | Queued. Owns data requirement and execution profile validation. |
| `contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` | Queued. Owns unknown marker helper family and `unknowns[*]` path/reason validation. |
| `contracts.qrpc_core.strategy_ir.root_validation.test_fixture` | Conditional. Only split if the remaining local test fixture still creates parent-sized pressure. |

## Selection Rationale

`risk_validation` is selected next because it is the next contiguous validation block after signal/logic validation:

- it owns `risk_rules.max_position_ratio` and `risk_rules.stop_loss_ratio` unknownable checks;
- it owns optional `risk_rules.take_profit_ratio`, `risk_rules.max_drawdown_ratio`, and `risk_rules.max_trades_per_day` unknownable checks;
- it owns `risk_profile.profile_id` validation against `global`;
- it owns risk profile numeric validation for `max_position`, `max_total_leverage`, and `max_exchange_leverage`;
- it can preserve validation ordering by replacing the existing contiguous risk block with one parent-mediated helper call;
- it should keep `validate_unknownable` and `validate_unknownable_opt` parent-owned because data/execution validation still needs the same helper family.

## Same-Parent Parallel Judgment

`same_parent_parallel_children` remains inactive for this selection.

`risk_validation`, `data_execution_validation`, and `unknown_marker_validation` all touch parent-owned unknown-marker helper behavior or validation order. The safer high-speed path is still single-child extraction with parent helper mediation until the remaining validation families close and helper ownership can be judged from evidence.

## Hard Boundaries

The next `risk_validation` baseline must not:

- edit Rust source code;
- change risk diagnostics, risk profile numeric behavior, unknown-marker diagnostics, validation ordering, or test expectations;
- move data requirement validation, execution validation, unknowns path/reason validation, public methods, root DTO fields, or tests;
- make root validation child modules import each other directly;
- change closed Strategy IR child DTOs/enums/constants/public functions, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Next Step

BE-001RE-01 `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p qrpc-core`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`

