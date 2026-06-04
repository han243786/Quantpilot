# v4.16.0 root.contracts.qrpc_core.strategy_ir.metadata_source baseline plan

> Batch: BE-001QK-01
> Node: `root.contracts.qrpc_core.strategy_ir.metadata_source`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.metadata_source` is frozen as the Strategy IR metadata/source DTO owner after BE-001QJ-01 selection.

BE-001QK-01 does not move code. It defines the exact baseline and allowed movement for BE-001QK-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/strategy_ir.rs`

Current selected boundary:

- `StrategyMetadata`;
- `StrategySource`;
- `StrategySourceType`.

Current parent callers:

- `StrategyIr` embeds `StrategyMetadata`;
- `StrategyMetadata` embeds `StrategySource`;
- `StrategyIr::validation_errors` reads `metadata.strategy_id`, `metadata.name`, and `metadata.summary`;
- tests build metadata/source through the public Strategy IR import path.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Stop after extraction. This child owns DTOs only and no behavior methods. |
| Mixed responsibility | Stop after extraction. Metadata identity, authors/tags, and source attribution form one DTO family. |
| Physical owner | Continue now. The selected surface still lives in `qrpc_core/src/strategy_ir.rs` and should be isolated. |
| Private helper pressure | Defer. No helper split is needed because the child has no validation helpers or internal behavior. |
| Future reopen rule | Allowed only when a concrete metadata field, source field, enum variant, serde default, or source type rename rule change is proposed. |

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Metadata DTO | `StrategyMetadata` keeps `strategy_id`, `name`, `summary`, `source`, defaulted `authors`, and defaulted `tags`. |
| Source DTO | `StrategySource` keeps `source_type`, `paper_title`, and optional `paper_reference`. |
| Source type serde | `StrategySourceType` keeps `snake_case` rename behavior and variants `ManualPaperAnalysis`, `LlmPaperAnalysis`, `HumanAuthored`, and `Imported`. |

## Allowed BE-001QK-02 Movement

BE-001QK-02 may:

- create `qrpc_core/src/strategy_ir/metadata_source.rs`;
- add a private `mod metadata_source;` declaration in `qrpc_core/src/strategy_ir.rs`;
- re-export the selected child surface from the Strategy IR parent with `pub use metadata_source::*;`;
- move only `StrategyMetadata`, `StrategySource`, and `StrategySourceType` into the child module;
- keep all public imports from `qrpc_core::strategy_ir::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001QK-02 Movement

BE-001QK-02 must not move or rewrite:

- `StrategyIr`, `StrategyIr::validation_errors`, or `StrategyIr::validate`;
- closed `version_unknown_error` child;
- signal/indicator, logic/position, risk, data, execution, gap/unknown, or root validation children;
- validation rule conditions or error text;
- tests unless needed only to preserve module visibility;
- closed error/proto/plugin contract children, `qrpc_core/src/lib.rs`, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Parent-Child Rule

Allowed call paths:

- Strategy IR parent -> private `metadata_source` child re-export;
- root `StrategyIr` DTO and validation -> metadata/source DTOs through the Strategy IR parent-local public surface;
- external callers -> metadata/source DTOs through `qrpc_core::strategy_ir::*` or `qrpc_core::*`.

Forbidden call paths:

Any metadata/source child import from future Strategy IR sibling modules, qrpc runtime/compiler modules, backend, executor, or release-transition paths that bypasses the Strategy IR parent.

## Proof

BE-001QK-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001QK-02 `root.contracts.qrpc_core.strategy_ir.metadata_source` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
