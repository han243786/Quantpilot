# v4.16.0 root.contracts.qrpc_core.plugin_contract parent residual judgment selects taxonomy_extension

> Batch: BE-001PU-01
> Node: `root.contracts.qrpc_core.plugin_contract`
> Selected child: `root.contracts.qrpc_core.plugin_contract.taxonomy_extension`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract` returns to the child queue created by BE-001PT-01.

Decision:

`next_child: root.contracts.qrpc_core.plugin_contract.taxonomy_extension`

## Closed Plugin Contract Children

No `root.contracts.qrpc_core.plugin_contract` children are closed yet.

## Open Plugin Contract Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.plugin_contract.taxonomy_extension` | Selected next. Owns plugin kind taxonomy and extension point mapping. |
| `contracts.qrpc_core.plugin_contract.capability_contract` | Queued. Owns capability contract IDs, parser, and declaration DTO. |
| `contracts.qrpc_core.plugin_contract.execution_security_dependency` | Queued. Owns execution, compatibility, security, and dependency DTOs. |
| `contracts.qrpc_core.plugin_contract.manifest_validation` | Queued. Owns manifest structure and validation logic. |
| `contracts.qrpc_core.plugin_contract.registry` | Queued. Owns in-memory registry behavior. |

## Selection Rationale

`contracts.qrpc_core.plugin_contract.taxonomy_extension` is selected first because it is the smallest independent plugin-contract child:

- physical region: `qrpc_core/src/plugin.rs`;
- public enum surface: `PluginKind`, `ExtensionPoint`;
- public methods: `PluginKind::{as_str,supported_extension_points,supported_capability_contracts}`, `ExtensionPoint::as_str`;
- dependency shape: capability mapping is referenced but should remain parent-mediated through public contract re-exports;
- no ownership of manifest fields, validation body, execution/security DTOs, registry storage, physical `plugins/*`, Strategy IR, or runtime behavior.

## Hard Boundaries

The next `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` baseline must not:

- edit Rust source code;
- change enum variants, serde rename rules, string literals, supported extension-point mappings, supported capability mappings, or tests;
- change `PluginManifest::validate`, `PluginCapabilityContract::{as_str,parse}`, `PluginRegistry`, execution/security/dependency DTOs, or manifest version constants;
- move physical plugin registry placeholders from `plugins/*`;
- change Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Next Step

BE-001PV-01 `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
