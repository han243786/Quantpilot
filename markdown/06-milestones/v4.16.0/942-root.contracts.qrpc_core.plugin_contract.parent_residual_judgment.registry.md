# v4.16.0 root.contracts.qrpc_core.plugin_contract parent residual judgment selects registry

> Batch: BE-001QC-01
> Node: `root.contracts.qrpc_core.plugin_contract`
> Selected child: `root.contracts.qrpc_core.plugin_contract.registry`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract` returns to its remaining child queue after `manifest_validation` closeout.

Decision:

`next_child: root.contracts.qrpc_core.plugin_contract.registry`

## Closed Plugin Contract Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.plugin_contract.taxonomy_extension` | Closed with `stop_split: true`; owns plugin kind taxonomy and extension-point mapping. |
| `contracts.qrpc_core.plugin_contract.capability_contract` | Closed with `stop_split: true`; owns capability identity, declaration, parser, strings, and versioning. |
| `contracts.qrpc_core.plugin_contract.execution_security_dependency` | Closed with `stop_split: true`; owns execution, compatibility, security, and dependency DTO shape. |
| `contracts.qrpc_core.plugin_contract.manifest_validation` | Closed with `stop_split: true`; owns manifest schema DTOs and `PluginManifest::validate`. |

## Open Plugin Contract Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.plugin_contract.registry` | Selected next. Owns in-memory plugin registry behavior. |

## Selection Rationale

`contracts.qrpc_core.plugin_contract.registry` is selected because it is the only remaining plugin contract child:

- physical region: `qrpc_core/src/plugin.rs`;
- public owner: `PluginRegistry`;
- public methods: `PluginRegistry::{register,get,remove,manifests_for_extension_point,manifests}`;
- behavior surface: registration validation call, duplicate id rejection, in-memory storage, lookup, deletion, extension point filtering, and manifest list projection;
- depends on closed manifest and extension-point surfaces through the plugin contract parent;
- no ownership of manifest DTO fields, validation rule internals, physical `plugins/*`, Strategy IR, runtime behavior, or release transition.

## Hard Boundaries

The next `root.contracts.qrpc_core.plugin_contract.registry` baseline must not:

- edit Rust source code;
- change registry method signatures, ordering, duplicate-id behavior, lookup semantics, removal semantics, or extension-point filtering;
- change manifest fields, serde attributes, validation rule conditions, validation error text, taxonomy mapping, capability parsing, or execution/security/dependency DTO shape;
- move physical plugin registry placeholders from `plugins/*`;
- change Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Next Step

BE-001QD-01 `root.contracts.qrpc_core.plugin_contract.registry` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
