# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation single leaf closeout

> Batch: BE-001QY-03
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.root_validation` has been evaluated after BE-001QY-02 extraction.

Decision:

`continue_split: true`

The node is now physically isolated, but it is not yet a compact leaf. It is a new parent candidate under Strategy IR because it owns root DTO shape, public validation methods, multiple validation families, private helpers, and local tests.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Continue. The selected surface is isolated in `qrpc_core/src/strategy_ir/root_validation.rs`, but the file is still a 490-line parent-sized module. |
| Public method count | Continue. `StrategyIr::validation_errors` and `StrategyIr::validate` form one public API family, but `validation_errors` currently owns many independently testable validation blocks. |
| Mixed responsibility | Continue. Version/required fields, unique id checks, signal/logic checks, risk/data/execution checks, unknown-marker checks, and local fixture tests are separable concerns. |
| Parent-mediated dependency | Required. New children must communicate through `root_validation.rs`; sibling validation children must not import each other directly. |
| Future reopen rule | Allowed for concrete validation-family, helper, fixture, or root DTO API proposals only. |

## Proposed Child Queue

| Order | Child | Initial ownership |
| --- | --- | --- |
| 1 | `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` | Version rule, metadata required fields, top-level required collections, unique id checks, and `validate_unique_ids`. |
| 2 | `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` | Signal validation, indicator support validation, logic rule validation, and logic position unknownable calls through parent-owned helpers. |
| 3 | `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` | Risk unknownable checks and risk profile numeric/id validation through parent-owned helpers. |
| 4 | `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` | Data requirement checks and execution/execution profile validation through parent-owned helpers. |
| 5 | `root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` | Unknown marker helper family and `unknowns[*]` path/reason validation. |
| 6 | `root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture` | `SAMPLE_JSON` and local tests, only if the remaining test/fixture region still creates parent-sized pressure after code children close. |

## First Child Rationale

`identity_required_validation` is selected first because it is the least coupled slice:

- it reads only root fields and id strings;
- it does not require unknownable helper sharing;
- it can move without changing validation ordering by keeping the parent call site in the same position;
- it gives the next extraction a small blast radius before the more coupled signal/logic/risk/data/execution families.

## Hard Boundaries

The next child baseline must not:

- edit Rust source code;
- change `StrategyIr` fields, serde attributes, validation ordering, diagnostics, helper behavior, or test expectations;
- make validation children import each other directly;
- change closed Strategy IR child DTOs/enums/constants/public functions, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Next Step

BE-001QZ-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent_residual_judgment selects `contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
