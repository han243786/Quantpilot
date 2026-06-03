# v4.16.0 root.contracts.qrpc_core.plugin_contract.execution_security_dependency baseline plan

> Batch: BE-001PZ-01
> Node: `root.contracts.qrpc_core.plugin_contract.execution_security_dependency`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.execution_security_dependency` is frozen as the plugin execution, compatibility, security, and dependency DTO owner after BE-001PY-01 selection.

BE-001PZ-01 does not move code. It defines the exact baseline and allowed movement for BE-001PZ-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/plugin.rs`

Current selected boundary:

- `PluginExecution`;
- `PluginExecutionEngine`;
- `PluginCompatibility`;
- `PluginSecurity`;
- `PluginDependency`.

Current parent callers:

- `PluginManifest` stores the selected DTOs as manifest fields;
- `PluginManifest::validate` validates `PluginCompatibility`, `PluginSecurity`, `PluginExecution::entrypoint`, and `PluginDependency::version_req`;
- tests construct these DTOs through the plugin contract parent.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Execution DTO | `PluginExecution` keeps `engine` and `entrypoint`. |
| Execution engine serde shape | `PluginExecutionEngine` variants remain `builtin`, `quant_script`, and `native` through `rename_all = "snake_case"`. |
| Compatibility DTO | `PluginCompatibility` keeps `core_ir_version` and `capability_api_version`. |
| Security DTO | `PluginSecurity` keeps `max_compute_ms`, `max_memory_mb`, `allow_network`, `enforce_max_compute_ms`, and `enforce_max_memory_mb`. |
| Security defaults | `allow_network`, `enforce_max_compute_ms`, and `enforce_max_memory_mb` keep existing serde defaults. |
| Dependency DTO | `PluginDependency` keeps `plugin_id` and `version_req`. |
| Manifest validation callers | Existing non-empty checks, zero checks, entrypoint path checks, and version_req format checks remain owned by manifest validation, not this DTO child. |

## Allowed BE-001PZ-02 Movement

BE-001PZ-02 may:

- create `qrpc_core/src/plugin/execution_security_dependency.rs`;
- add a private `mod execution_security_dependency;` declaration in `qrpc_core/src/plugin.rs`;
- re-export the selected child surface from the plugin contract parent with `pub use execution_security_dependency::*;`;
- move only `PluginExecution`, `PluginExecutionEngine`, `PluginCompatibility`, `PluginSecurity`, and `PluginDependency` into the child module;
- keep all public imports from `qrpc_core::plugin::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001PZ-02 Movement

BE-001PZ-02 must not move or rewrite:

- `PLUGIN_MANIFEST_V1_VERSION`, `PluginType`, `AtomRef`, `PluginManifest`, `PluginManifest::validate`, `PluginDisplay`, or manifest tests beyond imports required by the selected move;
- closed taxonomy and capability child modules;
- `PluginRegistry` and registry tests;
- validation conditions for compatibility fields, security limits, entrypoint path, or dependency version requirements;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Parent-Child Rule

Allowed call paths:

- plugin contract parent -> private `execution_security_dependency` child re-export;
- manifest parent -> selected DTOs through parent-local public surface.

Forbidden call paths:

Any execution/security/dependency child import from taxonomy, capability, manifest validation, registry, or future sibling modules that bypasses the plugin contract parent.

## Proof

BE-001PZ-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001PZ-02 `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
