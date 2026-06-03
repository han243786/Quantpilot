# v4.16.0 root.contracts.qrpc_core.plugin_contract parent residual judgment selects execution_security_dependency

> Batch: BE-001PY-01
> Node: `root.contracts.qrpc_core.plugin_contract`
> Selected child: `root.contracts.qrpc_core.plugin_contract.execution_security_dependency`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract` returns to its remaining child queue after `capability_contract` closeout.

Decision:

`next_child: root.contracts.qrpc_core.plugin_contract.execution_security_dependency`

## Closed Plugin Contract Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.plugin_contract.taxonomy_extension` | Closed with `stop_split: true`; owns plugin kind taxonomy and extension-point mapping. |
| `contracts.qrpc_core.plugin_contract.capability_contract` | Closed with `stop_split: true`; owns capability identity, declaration, parser, strings, and versioning. |

## Open Plugin Contract Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.plugin_contract.execution_security_dependency` | Selected next. Owns execution, compatibility, security, and dependency DTOs. |
| `contracts.qrpc_core.plugin_contract.manifest_validation` | Queued. Owns manifest structure and validation logic. |
| `contracts.qrpc_core.plugin_contract.registry` | Queued. Owns in-memory registry behavior. |

## Selection Rationale

`contracts.qrpc_core.plugin_contract.execution_security_dependency` is selected because it is the next compact DTO-only surface after taxonomy and capability extraction:

- physical region: `qrpc_core/src/plugin.rs`;
- public DTOs: `PluginExecution`, `PluginCompatibility`, `PluginSecurity`, and `PluginDependency`;
- public enum: `PluginExecutionEngine`;
- serde default fields: `PluginSecurity::{allow_network,enforce_max_compute_ms,enforce_max_memory_mb}`;
- no ownership of manifest validation body, capability parser, taxonomy mapping, registry storage, physical `plugins/*`, Strategy IR, or runtime behavior.

## Hard Boundaries

The next `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` baseline must not:

- edit Rust source code;
- change DTO fields, enum variants, serde rename/default rules, validation rules, or tests;
- change closed taxonomy/capability modules, `PluginManifest::validate`, `PluginRegistry`, manifest fields outside the selected DTO references, or version constants;
- move physical plugin registry placeholders from `plugins/*`;
- change Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Next Step

BE-001PZ-01 `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
