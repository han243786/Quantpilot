# v4.16.0 root.contracts.qrpc_core.plugin_contract parent residual judgment selects manifest_validation

> Batch: BE-001QA-01
> Node: `root.contracts.qrpc_core.plugin_contract`
> Selected child: `root.contracts.qrpc_core.plugin_contract.manifest_validation`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract` returns to its remaining child queue after `execution_security_dependency` closeout.

Decision:

`next_child: root.contracts.qrpc_core.plugin_contract.manifest_validation`

## Closed Plugin Contract Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.plugin_contract.taxonomy_extension` | Closed with `stop_split: true`; owns plugin kind taxonomy and extension-point mapping. |
| `contracts.qrpc_core.plugin_contract.capability_contract` | Closed with `stop_split: true`; owns capability identity, declaration, parser, strings, and versioning. |
| `contracts.qrpc_core.plugin_contract.execution_security_dependency` | Closed with `stop_split: true`; owns execution, compatibility, security, and dependency DTO shape. |

## Open Plugin Contract Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.plugin_contract.manifest_validation` | Selected next. Owns manifest version, manifest/type/display/atom DTOs, and `PluginManifest::validate`. |
| `contracts.qrpc_core.plugin_contract.registry` | Queued. Owns in-memory registry behavior. |

## Selection Rationale

`contracts.qrpc_core.plugin_contract.manifest_validation` is selected because it is now the last behavior-heavy contract surface before the registry:

- physical region: `qrpc_core/src/plugin.rs`;
- public version constant: `PLUGIN_MANIFEST_V1_VERSION`;
- public DTOs: `PluginType`, `AtomRef`, `PluginDisplay`, `PluginManifest`;
- public method: `PluginManifest::validate`;
- depends on closed taxonomy, capability, and execution/security/dependency children through the plugin contract parent;
- no ownership of `PluginRegistry`, physical `plugins/*`, Strategy IR, or runtime behavior.

## Hard Boundaries

The next `root.contracts.qrpc_core.plugin_contract.manifest_validation` baseline must not:

- edit Rust source code;
- change manifest fields, serde attributes, version strings, validation rule conditions, validation error text, or tests;
- change closed taxonomy/capability/execution-security-dependency modules, `PluginRegistry`, or registry behavior;
- move physical plugin registry placeholders from `plugins/*`;
- change Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Next Step

BE-001QB-01 `root.contracts.qrpc_core.plugin_contract.manifest_validation` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
