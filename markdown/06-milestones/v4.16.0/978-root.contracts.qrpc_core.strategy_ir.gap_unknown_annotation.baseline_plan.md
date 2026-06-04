# v4.16.0 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation baseline plan

> Batch: BE-001QW-01
> Node: `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` is frozen as the Strategy IR gap annotation and strategy unknown marker DTO owner after BE-001QV-01 selection.

BE-001QW-01 does not move code. It defines the exact baseline and allowed movement for BE-001QW-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/strategy_ir.rs`

Current selected boundary:

- `GapAnnotation`;
- `GapType`;
- `GapSeverity`;
- `StrategyUnknown`.

Current parent callers:

- `StrategyIr` embeds defaulted `Vec<GapAnnotation>`;
- `StrategyIr` embeds defaulted `Vec<StrategyUnknown>`;
- `StrategyIr::validation_errors` validates `StrategyUnknown.path` and `StrategyUnknown.reason`;
- tests parse `gap_annotations` and `unknowns` through the public Strategy IR import path.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Stop after extraction. This child owns DTO/enums only and has no public methods. |
| Mixed responsibility | Stop after extraction. Gap annotations and strategy unknown markers are one missing-knowledge annotation contract family. |
| Physical owner | Continue now. The selected surface still lives in `qrpc_core/src/strategy_ir.rs` and should be isolated. |
| Private helper pressure | Defer. Unknown path/reason validation remains under `root_validation`. |
| Future reopen rule | Allowed only when a concrete gap field, gap enum variant, unknown marker field, serde rule, or validation ownership proposal is made. |

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Gap annotation DTO | `GapAnnotation` keeps `gap_type`, `summary`, `severity`, and `blocking`. |
| Gap type enum | `GapType` keeps `Expression`, `Data`, `Execution`, `Risk`, and `Other` variants with `snake_case` serde rename behavior. |
| Gap severity enum | `GapSeverity` keeps `Low`, `Medium`, `High`, and `Critical` variants with `snake_case` serde rename behavior. |
| Strategy unknown DTO | `StrategyUnknown` keeps `path` and `reason`. |

## Allowed BE-001QW-02 Movement

BE-001QW-02 may:

- create `qrpc_core/src/strategy_ir/gap_unknown_annotation.rs`;
- add a private `mod gap_unknown_annotation;` declaration in `qrpc_core/src/strategy_ir.rs`;
- re-export the selected child surface from the Strategy IR parent with `pub use gap_unknown_annotation::*;`;
- move only `GapAnnotation`, `GapType`, `GapSeverity`, and `StrategyUnknown` into the child module;
- keep all public imports from `qrpc_core::strategy_ir::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001QW-02 Movement

BE-001QW-02 must not move or rewrite:

- `StrategyIr`, `StrategyIr::validation_errors`, or `StrategyIr::validate`;
- `validate_unknownable`, `validate_unknownable_opt`, `validate_logic_rule`, `validate_unique_ids`, `indicator_kind_supported`, or any other private validation helper;
- any closed Strategy IR child;
- root validation children;
- validation rule conditions or error text;
- tests unless needed only to preserve module visibility;
- closed error/proto/plugin contract children, `qrpc_core/src/lib.rs`, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Parent-Child Rule

Allowed call paths:

- Strategy IR parent -> private `gap_unknown_annotation` child re-export;
- root `StrategyIr` DTO and validation -> gap/unknown DTOs through the Strategy IR parent-local public surface;
- external callers -> gap/unknown DTOs through `qrpc_core::strategy_ir::*` or `qrpc_core::*`.

Forbidden call paths:

Any gap/unknown child import from future Strategy IR sibling modules, qrpc runtime/compiler modules, backend, executor, or release-transition paths that bypasses the Strategy IR parent.

## Proof

BE-001QW-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001QW-02 `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
