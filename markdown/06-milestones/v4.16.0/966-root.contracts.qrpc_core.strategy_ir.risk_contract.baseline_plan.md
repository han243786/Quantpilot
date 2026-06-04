# v4.16.0 root.contracts.qrpc_core.strategy_ir.risk_contract baseline plan

> Batch: BE-001QQ-01
> Node: `root.contracts.qrpc_core.strategy_ir.risk_contract`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.risk_contract` is frozen as the Strategy IR risk rule and risk profile DTO owner after BE-001QP-01 selection.

BE-001QQ-01 does not move code. It defines the exact baseline and allowed movement for BE-001QQ-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/strategy_ir.rs`

Current selected boundary:

- `StrategyRiskRules`;
- `StrategyRiskProfileRef`.

Current parent callers:

- `StrategyIr` embeds `StrategyRiskRules`;
- `StrategyIr` optionally embeds `StrategyRiskProfileRef`;
- `StrategyIr::validation_errors` validates risk unknownable markers, optional risk profile id, finite positive max position, and leverage floors;
- `validate_unknownable` and `validate_unknownable_opt` read selected risk fields but remain private root validation helpers;
- tests parse `risk_rules`, mutate profile references indirectly through the public Strategy IR import path, and assert unknown marker behavior through `risk_rules.take_profit_ratio`.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Stop after extraction. This child owns DTOs only and has no public methods. |
| Mixed responsibility | Stop after extraction. Risk limits and risk profile reference fields are one risk contract family. |
| Physical owner | Continue now. The selected surface still lives in `qrpc_core/src/strategy_ir.rs` and should be isolated. |
| Private helper pressure | Defer. Risk validation conditions and helper calls remain under `root_validation`. |
| Future reopen rule | Allowed only when a concrete risk rule field, risk profile field, serde/default rule, unknownable risk value type, or validation ownership proposal is made. |

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Risk rules DTO | `StrategyRiskRules` keeps `max_position_ratio`, `stop_loss_ratio`, `take_profit_ratio`, `max_drawdown_ratio`, `max_trades_per_day`, and defaulted `notes`. |
| Risk unknownable fields | `max_position_ratio` and `stop_loss_ratio` continue to use `KnownOrUnknown<f64>`; `take_profit_ratio` and `max_drawdown_ratio` continue to use `Option<KnownOrUnknown<f64>>`; `max_trades_per_day` continues to use `Option<KnownOrUnknown<u32>>`. |
| Risk profile DTO | `StrategyRiskProfileRef` keeps `profile_id`, defaulted `max_position`, defaulted `max_total_leverage`, defaulted `max_exchange_leverage`, and defaulted `min_action_interval_ms`. |

## Allowed BE-001QQ-02 Movement

BE-001QQ-02 may:

- create `qrpc_core/src/strategy_ir/risk_contract.rs`;
- add a private `mod risk_contract;` declaration in `qrpc_core/src/strategy_ir.rs`;
- re-export the selected child surface from the Strategy IR parent with `pub use risk_contract::*;`;
- move only `StrategyRiskRules` and `StrategyRiskProfileRef` into the child module;
- import `KnownOrUnknown` from the Strategy IR parent into the child module;
- keep all public imports from `qrpc_core::strategy_ir::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001QQ-02 Movement

BE-001QQ-02 must not move or rewrite:

- `StrategyIr`, `StrategyIr::validation_errors`, or `StrategyIr::validate`;
- `validate_unknownable`, `validate_unknownable_opt`, `validate_logic_rule`, `validate_unique_ids`, `indicator_kind_supported`, or any other private validation helper;
- closed `version_unknown_error`, `metadata_source`, `signal_indicator`, or `logic_position` children;
- data/execution/gap/unknown DTOs or root validation children;
- validation rule conditions or error text;
- tests unless needed only to preserve module visibility;
- closed error/proto/plugin contract children, `qrpc_core/src/lib.rs`, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Parent-Child Rule

Allowed call paths:

- Strategy IR parent -> private `risk_contract` child re-export;
- root `StrategyIr` DTO and validation -> risk DTOs through the Strategy IR parent-local public surface;
- `risk_contract` child -> parent-provided `KnownOrUnknown` only;
- external callers -> risk DTOs through `qrpc_core::strategy_ir::*` or `qrpc_core::*`.

Forbidden call paths:

Any risk child import from future Strategy IR sibling modules, qrpc runtime/compiler modules, backend, executor, or release-transition paths that bypasses the Strategy IR parent.

## Proof

BE-001QQ-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001QQ-02 `root.contracts.qrpc_core.strategy_ir.risk_contract` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
