# v4.16.0 root.contracts.qrpc_core.strategy_ir.signal_indicator baseline plan

> Batch: BE-001QM-01
> Node: `root.contracts.qrpc_core.strategy_ir.signal_indicator`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.signal_indicator` is frozen as the Strategy IR signal/indicator DTO and public indicator registry owner after BE-001QL-01 selection.

BE-001QM-01 does not move code. It defines the exact baseline and allowed movement for BE-001QM-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/strategy_ir.rs`

Current selected boundary:

- `SignalDefinition`;
- `IndicatorDefinition`;
- `IndicatorKind`;
- `DECLARED_INDICATOR_KINDS`;
- `SUPPORTED_INDICATOR_KINDS`;
- `declared_indicator_kinds`;
- `supported_indicator_kinds`.

Current parent callers:

- `StrategyIr` embeds `Vec<SignalDefinition>`;
- `StrategyIr::validation_errors` reads signal ids, names, indicator inputs, `IndicatorKind::Spread`, and `indicator_kind_supported`;
- `indicator_kind_supported` calls `supported_indicator_kinds`;
- tests mutate `IndicatorKind::Custom` and validate support behavior through the public Strategy IR import path.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Stop after extraction. This child owns two public registry functions with one shared indicator enum/list owner. |
| Mixed responsibility | Stop after extraction. Signal DTO, indicator DTO, indicator taxonomy, and declared/supported registry lists are one contract family. |
| Physical owner | Continue now. The selected surface still lives in `qrpc_core/src/strategy_ir.rs` and should be isolated. |
| Private helper pressure | Defer. `indicator_kind_supported` remains a root validation helper until `root_validation` is processed. |
| Future reopen rule | Allowed only when a concrete signal field, indicator field, enum variant, serde rename rule, declared/supported list, or public indicator registry return contract change is proposed. |

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Signal DTO | `SignalDefinition` keeps `signal_id`, `name`, `indicator`, and defaulted `transforms`. |
| Indicator DTO | `IndicatorDefinition` keeps `kind`, `inputs`, and defaulted `params`. |
| Indicator enum | `IndicatorKind` keeps all 18 variants and `snake_case` serde rename behavior. |
| Declared registry | `DECLARED_INDICATOR_KINDS` and `declared_indicator_kinds()` keep the same 18 entries and ordering. |
| Supported registry | `SUPPORTED_INDICATOR_KINDS` and `supported_indicator_kinds()` keep the same 18 entries and ordering. |

## Allowed BE-001QM-02 Movement

BE-001QM-02 may:

- create `qrpc_core/src/strategy_ir/signal_indicator.rs`;
- add a private `mod signal_indicator;` declaration in `qrpc_core/src/strategy_ir.rs`;
- re-export the selected child surface from the Strategy IR parent with `pub use signal_indicator::*;`;
- move only `SignalDefinition`, `IndicatorDefinition`, `IndicatorKind`, `DECLARED_INDICATOR_KINDS`, `SUPPORTED_INDICATOR_KINDS`, `declared_indicator_kinds`, and `supported_indicator_kinds` into the child module;
- move the `BTreeMap` and `serde_json::Value` imports into the child only if the parent no longer needs them for other DTOs;
- keep all public imports from `qrpc_core::strategy_ir::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001QM-02 Movement

BE-001QM-02 must not move or rewrite:

- `StrategyIr`, `StrategyIr::validation_errors`, or `StrategyIr::validate`;
- `indicator_kind_supported` or any other private validation helper;
- closed `version_unknown_error` or `metadata_source` children;
- logic/position, risk, data, execution, gap/unknown, or root validation children;
- validation rule conditions or error text;
- tests unless needed only to preserve module visibility;
- closed error/proto/plugin contract children, `qrpc_core/src/lib.rs`, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Parent-Child Rule

Allowed call paths:

- Strategy IR parent -> private `signal_indicator` child re-export;
- root `StrategyIr` DTO and validation -> signal/indicator DTOs and registry functions through the Strategy IR parent-local public surface;
- external callers -> signal/indicator DTOs and indicator registry functions through `qrpc_core::strategy_ir::*` or `qrpc_core::*`.

Forbidden call paths:

Any signal/indicator child import from future Strategy IR sibling modules, qrpc runtime/compiler modules, backend, executor, or release-transition paths that bypasses the Strategy IR parent.

## Proof

BE-001QM-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001QM-02 `root.contracts.qrpc_core.strategy_ir.signal_indicator` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
