# v4.16.0 root.contracts.qrpc_core.strategy_ir.data_requirement baseline plan

> Batch: BE-001QS-01
> Node: `root.contracts.qrpc_core.strategy_ir.data_requirement`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.data_requirement` is frozen as the Strategy IR data requirement DTO and data type taxonomy owner after BE-001QR-01 selection.

BE-001QS-01 does not move code. It defines the exact baseline and allowed movement for BE-001QS-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/strategy_ir.rs`

Current selected boundary:

- `DataRequirement`;
- `DataRequirementType`.

Current parent callers:

- `StrategyIr` embeds `Vec<DataRequirement>`;
- `StrategyIr::validation_errors` checks required data requirements, unique `data_id`, non-empty `data_id`, non-empty `fields`, and unknownable `venue`, `symbol`, `granularity`, and `lookback`;
- `validate_unknownable` reads selected data fields but remains a private root validation helper;
- tests parse `data_requirements` through the public Strategy IR import path.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Stop after extraction. This child owns DTO/enums only and has no public methods. |
| Mixed responsibility | Stop after extraction. Data requirement fields and data type taxonomy are one data requirement contract family. |
| Physical owner | Continue now. The selected surface still lives in `qrpc_core/src/strategy_ir.rs` and should be isolated. |
| Private helper pressure | Defer. Data requirement validation conditions and helper calls remain under `root_validation`. |
| Future reopen rule | Allowed only when a concrete data requirement field, data requirement type enum variant, serde rule, unknownable data value type, or validation ownership proposal is made. |

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Data requirement DTO | `DataRequirement` keeps `data_id`, `venue`, `symbol`, `data_type`, `granularity`, `lookback`, and `fields`. |
| Data unknownable fields | `venue`, `symbol`, and `granularity` continue to use `KnownOrUnknown<String>`; `lookback` continues to use `KnownOrUnknown<u32>`. |
| Data type enum | `DataRequirementType` keeps `Kline`, `Quote`, `Tick`, `OrderBook`, `Fundamental`, and `Event` variants with `snake_case` serde rename behavior. |

## Allowed BE-001QS-02 Movement

BE-001QS-02 may:

- create `qrpc_core/src/strategy_ir/data_requirement.rs`;
- add a private `mod data_requirement;` declaration in `qrpc_core/src/strategy_ir.rs`;
- re-export the selected child surface from the Strategy IR parent with `pub use data_requirement::*;`;
- move only `DataRequirement` and `DataRequirementType` into the child module;
- import `KnownOrUnknown` from the Strategy IR parent into the child module;
- keep all public imports from `qrpc_core::strategy_ir::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001QS-02 Movement

BE-001QS-02 must not move or rewrite:

- `StrategyIr`, `StrategyIr::validation_errors`, or `StrategyIr::validate`;
- `validate_unknownable`, `validate_unknownable_opt`, `validate_logic_rule`, `validate_unique_ids`, `indicator_kind_supported`, or any other private validation helper;
- closed `version_unknown_error`, `metadata_source`, `signal_indicator`, `logic_position`, or `risk_contract` children;
- execution/gap/unknown DTOs or root validation children;
- validation rule conditions or error text;
- tests unless needed only to preserve module visibility;
- closed error/proto/plugin contract children, `qrpc_core/src/lib.rs`, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Parent-Child Rule

Allowed call paths:

- Strategy IR parent -> private `data_requirement` child re-export;
- root `StrategyIr` DTO and validation -> data requirement DTOs through the Strategy IR parent-local public surface;
- `data_requirement` child -> parent-provided `KnownOrUnknown` only;
- external callers -> data requirement DTOs through `qrpc_core::strategy_ir::*` or `qrpc_core::*`.

Forbidden call paths:

Any data requirement child import from future Strategy IR sibling modules, qrpc runtime/compiler modules, backend, executor, or release-transition paths that bypasses the Strategy IR parent.

## Proof

BE-001QS-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001QS-02 `root.contracts.qrpc_core.strategy_ir.data_requirement` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
