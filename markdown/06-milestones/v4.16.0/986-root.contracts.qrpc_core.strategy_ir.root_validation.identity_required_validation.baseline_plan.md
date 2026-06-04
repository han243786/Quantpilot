# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation baseline plan

> Batch: BE-001RA-01
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

This baseline freezes the first Strategy IR root validation child before extraction.

The next implementation step may move only the version, required-field, required-collection, duplicate-id, and `validate_unique_ids` logic into a child module under `root_validation.rs`.

## Frozen Surface

The selected child owns:

- `ir_version` equality check against `STRATEGY_IR_V0_VERSION`;
- `metadata.strategy_id`, `metadata.name`, and `metadata.summary` required checks;
- top-level `signals`, `data_requirements`, and `logic.entry_rules` non-empty checks;
- unique id checks for signals, data requirements, and logic rules;
- private helper `validate_unique_ids`.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Version diagnostic | `ir_version 必须是 {STRATEGY_IR_V0_VERSION}，但实际为 {actual}` remains unchanged. |
| Metadata diagnostics | Existing Chinese required-field messages remain unchanged. |
| Collection diagnostics | Existing signals, data_requirements, and logic.entry_rules non-empty messages remain unchanged. |
| Duplicate id labels | Labels remain exactly `signals`, `data_requirements`, and `logic rules`. |
| Duplicate id diagnostic | `{label} 包含重复的 id: {value}` remains unchanged. |
| Validation ordering | This child must be called at the same point where the original inline checks ran, before signal/detail validation loops. |

## Planned Extraction Shape

BE-001RA-02 may:

- create `qrpc_core/src/strategy_ir/root_validation/identity_required_validation.rs`;
- add `mod identity_required_validation;` to `qrpc_core/src/strategy_ir/root_validation.rs`;
- move `validate_unique_ids` into the child;
- add one parent-mediated helper call, tentatively `identity_required_validation::validate_identity_and_required_fields(self, &mut errors);`;
- remove `use std::collections::BTreeSet;` from `root_validation.rs` if `validate_unique_ids` moves fully to the child.

## Hard Boundaries

BE-001RA-02 must not:

- change validation ordering, diagnostics, labels, or helper behavior;
- move signal detail validation, logic rule validation, risk validation, data validation, execution validation, unknown marker validation, public methods, root DTO fields, or tests;
- make child validation modules import each other directly;
- change closed Strategy IR child DTOs/enums/constants/public functions, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Split Rule Pre-Check

| Rule | Baseline result |
| --- | --- |
| Physical owner | Worth extracting. The selected slice is an independent initial validation block plus one helper. |
| Public method count | Stop after extraction unless this child grows new public API; it should expose only parent-local helper behavior. |
| Mixed responsibility | Stop after extraction. Version, required fields, and unique ids are one identity/readiness validation family. |
| Parent-mediated dependency | Required. The child receives `StrategyIr` through the root validation parent and does not call sibling children. |
| Future reopen rule | Allowed only for concrete identity, required-field, required-collection, or duplicate-id validation proposals. |

## Next Step

BE-001RA-02 `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
