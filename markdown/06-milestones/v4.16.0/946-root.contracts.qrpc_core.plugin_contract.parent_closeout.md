# v4.16.0 root.contracts.qrpc_core.plugin_contract parent closeout

> Batch: BE-001QE-01
> Node: `root.contracts.qrpc_core.plugin_contract`
> Parent: `root.contracts.qrpc_core`
> Stage: `parent_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract` is closed for the current recursive qrpc_core contracts extraction scope.

Decision:

`close_parent: true`

The parent remains equivalent because the public plugin contract surface is still exported through the same paths:

- `qrpc_core::plugin::*`;
- `qrpc_core::*` via `qrpc_core/src/lib.rs`.

## Closed Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.plugin_contract.taxonomy_extension` | Closed with `stop_split: true`; owns plugin kind taxonomy and extension-point mapping. |
| `contracts.qrpc_core.plugin_contract.capability_contract` | Closed with `stop_split: true`; owns capability identity, declaration, parser, strings, and versioning. |
| `contracts.qrpc_core.plugin_contract.execution_security_dependency` | Closed with `stop_split: true`; owns execution, compatibility, security, and dependency DTO shape. |
| `contracts.qrpc_core.plugin_contract.manifest_validation` | Closed with `stop_split: true`; owns manifest schema DTOs and `PluginManifest::validate`. |
| `contracts.qrpc_core.plugin_contract.registry` | Closed with `stop_split: true`; owns in-memory plugin registry behavior. |

## Parent Boundary

`root.contracts.qrpc_core.plugin_contract` now owns the plugin-contract parent boundary:

- `qrpc_core/src/plugin.rs` is the parent facade and test host;
- private child modules own taxonomy, capability, execution/security/dependency DTOs, manifest validation, and registry behavior;
- child communication remains mediated by the plugin contract parent surface;
- physical `plugins/*` metadata placeholders remain queued under `contracts.plugin_metadata`;
- Strategy IR, protocol primitives, runtime protocol config, artifact specs, runtime IO, RFC execution contracts, compiler/runtime/backend/executor/frontend behavior, and release transition logic remain outside this parent;
- any future plugin contract change must name the concrete child leaf before editing Rust code.

## Non-Claims

This closeout does not claim:

- plugin contract behavior changed;
- manifest validation or registry semantics changed;
- physical plugin metadata placeholders are extracted;
- Strategy IR or other qrpc_core `lib.rs` contracts are complete;
- compiler/runtime/backend/executor/frontend behavior changed;
- release transition optimization is allowed.

## Qrpc_Core Return

Return to `root.contracts.qrpc_core` residual judgment.

Recommended next child:

`root.contracts.qrpc_core.strategy_ir`

Rationale: after error contract, event envelope proto, and plugin contract are closed, the next highest-risk qrpc_core owner is the Strategy IR file, which owns validation behavior, known/unknown preservation, indicator kind surfaces, and public IR DTO shape.

## Next Step

BE-001QF-01 `root.contracts.qrpc_core` parent_residual_judgment selects `contracts.qrpc_core.strategy_ir`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
