# v4.16.0 root.contracts.qrpc_core.strategy_ir.signal_indicator single leaf closeout

> Batch: BE-001QM-03
> Node: `root.contracts.qrpc_core.strategy_ir.signal_indicator`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.signal_indicator` has been evaluated after BE-001QM-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: Strategy IR signal/indicator DTO shape and public indicator registry surfaces.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is now isolated in `qrpc_core/src/strategy_ir/signal_indicator.rs`. |
| Public method count | Stop. The two public registry functions are tied to one indicator enum/list owner. |
| Mixed responsibility | Stop. Signal DTO, indicator DTO, indicator taxonomy, and declared/supported registries are one contract family. |
| Parent-mediated dependency | Covered. Root Strategy IR DTO and validation reach signal/indicator surfaces through the Strategy IR parent re-export. |
| Future reopen rule | Allowed only when a concrete signal field, indicator field, enum variant, serde rename rule, declared/supported list, or public indicator registry return contract change is proposed. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Strategy IR signal/indicator DTO or registry proposal | `contracts.qrpc_core.strategy_ir.signal_indicator` | Updated or verified signal/indicator DTO shape and public indicator registry behavior |

The leaf may describe and guard:

- `SignalDefinition`;
- `IndicatorDefinition`;
- `IndicatorKind`;
- `declared_indicator_kinds`;
- `supported_indicator_kinds`;
- `DECLARED_INDICATOR_KINDS`;
- `SUPPORTED_INDICATOR_KINDS`.

## Non-Claims

This closeout does not claim:

- root validation helper behavior changed;
- validation rule conditions or error text changed;
- logic/risk/data/execution/gap DTOs changed;
- protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QN-01 `root.contracts.qrpc_core.strategy_ir` parent_residual_judgment selects `contracts.qrpc_core.strategy_ir.logic_position`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
