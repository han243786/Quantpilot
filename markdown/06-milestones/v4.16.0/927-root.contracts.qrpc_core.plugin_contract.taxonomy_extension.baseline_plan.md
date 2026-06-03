# v4.16.0 root.contracts.qrpc_core.plugin_contract.taxonomy_extension baseline plan

> Batch: BE-001PV-01
> Node: `root.contracts.qrpc_core.plugin_contract.taxonomy_extension`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.taxonomy_extension` is frozen as the plugin taxonomy and extension-point mapping owner after BE-001PU-01 selection.

BE-001PV-01 does not move code. It defines the exact baseline and allowed movement for BE-001PV-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/plugin.rs`

Current selected boundary:

- `PluginKind`;
- `PluginKind::as_str`;
- `PluginKind::supported_extension_points`;
- `PluginKind::supported_capability_contracts`;
- `ExtensionPoint`;
- `ExtensionPoint::as_str`.

Parent-mediated dependency:

- `PluginKind::supported_capability_contracts` returns `PluginCapabilityContract` values that remain owned by the queued `capability_contract` child. BE-001PV-02 may reference this type through the plugin contract parent, but must not move capability parsing or capability constants.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| PluginKind serde shape | Variants remain `data`, `intent`, `agent`, `risk`, and `execution` through `rename_all = "snake_case"`. |
| PluginKind strings | `as_str()` returns `data`, `intent`, `agent`, `risk`, and `execution`. |
| ExtensionPoint serde shape | Variants remain `data_module_provider`, `intent_module_provider`, `agent_module_provider`, `risk_checker_provider`, and `execution_module_provider`. |
| ExtensionPoint strings | `as_str()` returns the same five extension point strings. |
| Extension mapping | Each plugin kind maps to exactly one corresponding extension point. |
| Capability mapping | Each plugin kind maps to exactly one corresponding capability contract through parent-mediated `PluginCapabilityContract`. |
| Manifest validation callers | `PluginManifest::validate` continues to call the same public mapping methods. |

## Allowed BE-001PV-02 Movement

BE-001PV-02 may:

- create `qrpc_core/src/plugin/taxonomy_extension.rs`;
- add a private `mod taxonomy_extension;` declaration in `qrpc_core/src/plugin.rs`;
- re-export the selected child surface from the plugin contract parent with `pub use taxonomy_extension::*;`;
- move only `PluginKind`, its impl, `ExtensionPoint`, and its impl into the child module;
- reference `super::PluginCapabilityContract` from the child to preserve the existing `supported_capability_contracts` return type;
- keep all public imports from `qrpc_core::plugin::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001PV-02 Movement

BE-001PV-02 must not move or rewrite:

- `PLUGIN_MANIFEST_V1_VERSION` or `PLUGIN_CAPABILITY_CONTRACT_V1_VERSION`;
- `PluginManifest`, `PluginManifest::validate`, `PluginType`, `AtomRef`, `PluginDisplay`, or manifest tests beyond imports required by the selected move;
- `PluginCapabilityContract`, `PluginCapabilityDeclaration`, parser logic, or capability string constants;
- `PluginExecution`, `PluginExecutionEngine`, `PluginCompatibility`, `PluginSecurity`, or `PluginDependency`;
- `PluginRegistry` and registry tests;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Parent-Child Rule

Allowed call paths:

- plugin contract parent -> private `taxonomy_extension` child re-export;
- manifest validation parent -> public taxonomy methods;
- taxonomy child -> parent-owned `PluginCapabilityContract` type through `super::PluginCapabilityContract`.

Forbidden call paths:

Any taxonomy child import from a future capability sibling module or any closed/open sibling that bypasses the plugin contract parent.

## Proof

BE-001PV-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001PV-02 `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
