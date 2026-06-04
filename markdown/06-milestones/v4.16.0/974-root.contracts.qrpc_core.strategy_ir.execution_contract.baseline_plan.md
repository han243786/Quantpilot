# v4.16.0 root.contracts.qrpc_core.strategy_ir.execution_contract baseline plan

> Batch: BE-001QU-01
> Node: `root.contracts.qrpc_core.strategy_ir.execution_contract`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.execution_contract` is frozen as the Strategy IR execution DTO and execution profile reference owner after BE-001QT-01 selection.

BE-001QU-01 does not move code. It defines the exact baseline and allowed movement for BE-001QU-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/strategy_ir.rs`

Current selected boundary:

- `StrategyExecution`;
- `StrategyExecutionProfileRef`.

Current parent callers:

- `StrategyIr` embeds `StrategyExecution`;
- `StrategyIr` optionally embeds `StrategyExecutionProfileRef`;
- `StrategyIr::validation_errors` validates execution unknownable markers, optional execution profile id, finite non-negative fees, and finite non-negative slippage;
- `validate_unknownable` and `validate_unknownable_opt` read selected execution fields but remain private root validation helpers;
- tests mutate `execution.order_type`, construct `StrategyExecutionProfileRef`, and assert validation behavior through the public Strategy IR import path.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Stop after extraction. This child owns DTOs only and has no public methods. |
| Mixed responsibility | Stop after extraction. Execution venue/order/slippage settings and execution profile reference fields are one execution contract family. |
| Physical owner | Continue now. The selected surface still lives in `qrpc_core/src/strategy_ir.rs` and should be isolated. |
| Private helper pressure | Defer. Execution validation conditions and helper calls remain under `root_validation`. |
| Future reopen rule | Allowed only when a concrete execution field, execution profile field, serde/default rule, unknownable execution value type, or validation ownership proposal is made. |

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Execution DTO | `StrategyExecution` keeps `venue_type`, `order_type`, `time_in_force`, `slippage_model`, `latency_assumption_ms`, and `capital_base`. |
| Execution unknownable fields | `venue_type`, `order_type`, and `slippage_model` continue to use `KnownOrUnknown<String>`; `time_in_force` continues to use `Option<KnownOrUnknown<String>>`; `latency_assumption_ms` continues to use `Option<KnownOrUnknown<u32>>`; `capital_base` continues to use `Option<KnownOrUnknown<f64>>`. |
| Execution profile DTO | `StrategyExecutionProfileRef` keeps `profile_id`, defaulted `fee_bps`, and defaulted `slippage_bps`. |

## Allowed BE-001QU-02 Movement

BE-001QU-02 may:

- create `qrpc_core/src/strategy_ir/execution_contract.rs`;
- add a private `mod execution_contract;` declaration in `qrpc_core/src/strategy_ir.rs`;
- re-export the selected child surface from the Strategy IR parent with `pub use execution_contract::*;`;
- move only `StrategyExecution` and `StrategyExecutionProfileRef` into the child module;
- import `KnownOrUnknown` from the Strategy IR parent into the child module;
- keep all public imports from `qrpc_core::strategy_ir::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001QU-02 Movement

BE-001QU-02 must not move or rewrite:

- `StrategyIr`, `StrategyIr::validation_errors`, or `StrategyIr::validate`;
- `validate_unknownable`, `validate_unknownable_opt`, `validate_logic_rule`, `validate_unique_ids`, `indicator_kind_supported`, or any other private validation helper;
- closed `version_unknown_error`, `metadata_source`, `signal_indicator`, `logic_position`, `risk_contract`, or `data_requirement` children;
- gap/unknown DTOs or root validation children;
- validation rule conditions or error text;
- tests unless needed only to preserve module visibility;
- closed error/proto/plugin contract children, `qrpc_core/src/lib.rs`, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Parent-Child Rule

Allowed call paths:

- Strategy IR parent -> private `execution_contract` child re-export;
- root `StrategyIr` DTO and validation -> execution DTOs through the Strategy IR parent-local public surface;
- `execution_contract` child -> parent-provided `KnownOrUnknown` only;
- external callers -> execution DTOs through `qrpc_core::strategy_ir::*` or `qrpc_core::*`.

Forbidden call paths:

Any execution child import from future Strategy IR sibling modules, qrpc runtime/compiler modules, backend, executor, or release-transition paths that bypasses the Strategy IR parent.

## Proof

BE-001QU-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001QU-02 `root.contracts.qrpc_core.strategy_ir.execution_contract` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
