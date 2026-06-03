# v4.16.0 root.contracts.qrpc_core.plugin_contract.capability_contract baseline plan

> Batch: BE-001PX-01
> Node: `root.contracts.qrpc_core.plugin_contract.capability_contract`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.capability_contract` is frozen as the plugin capability contract owner after BE-001PW-01 selection.

BE-001PX-01 does not move code. It defines the exact baseline and allowed movement for BE-001PX-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/plugin.rs`

Current selected boundary:

- `PLUGIN_CAPABILITY_CONTRACT_V1_VERSION`;
- `PluginCapabilityDeclaration`;
- `PluginCapabilityContract`;
- `PluginCapabilityContract::as_str`;
- `PluginCapabilityContract::parse`.

Current parent callers:

- `PluginManifest::validate` validates declared capability IDs and versions through `PluginCapabilityContract::parse`, `PluginCapabilityContract::as_str`, and `PLUGIN_CAPABILITY_CONTRACT_V1_VERSION`;
- `PluginKind::supported_capability_contracts` returns capability enum values from the already closed taxonomy child through the plugin contract parent.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Capability version | `PLUGIN_CAPABILITY_CONTRACT_V1_VERSION` remains `v1`. |
| Declaration DTO | `PluginCapabilityDeclaration` keeps `id` and `version` string fields. |
| Capability serde shape | Variants remain snake_case if serialized directly. |
| Capability strings | `as_str()` returns the same five `quantpilot.capability.*` IDs. |
| Parser | `parse()` accepts only the same five capability IDs and returns `None` for unknown IDs. |
| Manifest validation callers | Unknown capability IDs, mismatched kind/capability pairs, and non-v1 capability versions keep the same validation behavior. |

## Allowed BE-001PX-02 Movement

BE-001PX-02 may:

- create `qrpc_core/src/plugin/capability_contract.rs`;
- add a private `mod capability_contract;` declaration in `qrpc_core/src/plugin.rs`;
- re-export the selected child surface from the plugin contract parent with `pub use capability_contract::*;`;
- move only `PLUGIN_CAPABILITY_CONTRACT_V1_VERSION`, `PluginCapabilityDeclaration`, `PluginCapabilityContract`, and `PluginCapabilityContract` impl into the child module;
- update the already extracted taxonomy child to reference capability types through the plugin contract parent if needed;
- keep all public imports from `qrpc_core::plugin::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001PX-02 Movement

BE-001PX-02 must not move or rewrite:

- `PLUGIN_MANIFEST_V1_VERSION`;
- `PluginType`, `AtomRef`, `PluginManifest`, `PluginManifest::validate`, `PluginDisplay`, or manifest tests beyond imports required by the selected move;
- closed `PluginKind`, `ExtensionPoint`, or taxonomy mapping behavior;
- `PluginExecution`, `PluginExecutionEngine`, `PluginCompatibility`, `PluginSecurity`, or `PluginDependency`;
- `PluginRegistry` and registry tests;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Parent-Child Rule

Allowed call paths:

- plugin contract parent -> private `capability_contract` child re-export;
- manifest validation parent -> public capability parser and version constant;
- taxonomy child -> parent-re-exported `PluginCapabilityContract` type.

Forbidden call paths:

Any capability child import from taxonomy, manifest validation, registry, or future sibling modules that bypasses the plugin contract parent.

## Proof

BE-001PX-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001PX-02 `root.contracts.qrpc_core.plugin_contract.capability_contract` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
