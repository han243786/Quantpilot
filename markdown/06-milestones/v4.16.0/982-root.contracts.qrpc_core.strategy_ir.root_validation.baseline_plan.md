# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation baseline plan

> Batch: BE-001QY-01
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

This baseline freezes the remaining Strategy IR root validation surface before extraction.

The next implementation step may move only the root Strategy IR DTO, validation methods, private validation helpers, and local Strategy IR tests from `qrpc_core/src/strategy_ir.rs` into a private child module under the Strategy IR parent.

## Frozen Surface

The selected child owns:

- `StrategyIr`;
- `StrategyIr::validation_errors`;
- `StrategyIr::validate`;
- `validate_unique_ids`;
- `validate_logic_rule`;
- `indicator_kind_supported`;
- `validate_unknownable`;
- `validate_unknownable_opt`;
- the local `strategy_ir::tests` module and `SAMPLE_JSON` fixture.

## Current Behavior Dependencies

The extracted root validation module must preserve these dependencies through the Strategy IR parent:

- `STRATEGY_IR_V0_VERSION` and `StrategyIrValidationError` from `version_unknown_error`;
- all closed Strategy IR DTO/enums through parent-local imports or re-exports;
- `supported_indicator_kinds()` from `signal_indicator`;
- `KnownOrUnknown<T>` unknown-marker semantics from `version_unknown_error`.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Root DTO shape | `StrategyIr` fields, defaults, serde attributes, and deny-unknown-fields behavior remain unchanged. |
| Public validation API | `validation_errors()` returns the same ordered `Vec<String>`; `validate()` returns `Ok(())` or `StrategyIrValidationError { errors }` with the same errors. |
| Version rule | `ir_version` must equal `STRATEGY_IR_V0_VERSION` with the same diagnostic string. |
| Required fields | Metadata, signal, data requirement, and logic required-field diagnostics remain unchanged. |
| Duplicate ids | Signal id, data id, and logic rule id duplicate detection keeps the same labels and error string shape. |
| Signal validation | Empty ids/names, missing inputs, spread input count, and supported indicator checks remain unchanged. |
| Logic validation | Entry/exit rule required fields, position sizing unknown marker, and optional rebalance unknown marker checks remain unchanged. |
| Risk validation | Unknownable risk markers, risk profile id, max position, and leverage floor checks remain unchanged. |
| Data validation | Data id, field list, venue, symbol, granularity, and lookback checks remain unchanged. |
| Execution validation | Unknownable execution markers, execution profile id, finite fee, and finite slippage checks remain unchanged. |
| Unknown validation | `unknowns[*].path` and `unknowns[*].reason` required checks remain unchanged. |
| Tests | Existing Strategy IR tests keep the same assertions and sample JSON semantics. |

## Planned Extraction Shape

BE-001QY-02 may:

- create `qrpc_core/src/strategy_ir/root_validation.rs`;
- move the frozen surface into that child;
- add `mod root_validation;` to `qrpc_core/src/strategy_ir.rs`;
- add `pub use root_validation::*;` to preserve `qrpc_core::strategy_ir::*` and `qrpc_core::*`;
- remove `use std::collections::BTreeSet;` from the parent if the helper owner moves fully to the child.

## Hard Boundaries

BE-001QY-02 must not:

- change validation rule conditions, validation ordering, diagnostics, helper behavior, serde shape, or tests;
- change closed child DTOs/enums/constants/public functions beyond imports required by the selected move;
- make any closed Strategy IR child import another closed sibling directly;
- change `qrpc_core/src/lib.rs`, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Split Rule Pre-Check

| Rule | Baseline result |
| --- | --- |
| Physical owner | Worth extracting. The selected residual is the only remaining root code in `qrpc_core/src/strategy_ir.rs`. |
| Public method count | Watch. The child owns two public methods, but both are one validation API family. |
| Mixed responsibility | Watch. Validation spans all Strategy IR DTO families, but it is one root invariant pass. Further split is deferred to closeout after extraction evidence. |
| Parent-mediated dependency | Required. Closed child DTOs and helper surfaces must be reached through the Strategy IR parent boundary. |
| Future reopen rule | Allowed only when a concrete validation family, validation helper, test fixture, or Strategy IR root DTO proposal is made. |

## Next Step

BE-001QY-02 `root.contracts.qrpc_core.strategy_ir.root_validation` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
