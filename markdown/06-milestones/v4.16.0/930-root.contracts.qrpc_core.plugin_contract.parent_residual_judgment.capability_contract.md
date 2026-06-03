# v4.16.0 root.contracts.qrpc_core.plugin_contract parent residual judgment selects capability_contract

> Batch: BE-001PW-01
> Node: `root.contracts.qrpc_core.plugin_contract`
> Selected child: `root.contracts.qrpc_core.plugin_contract.capability_contract`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract` returns to its remaining child queue after `taxonomy_extension` closeout.

Decision:

`next_child: root.contracts.qrpc_core.plugin_contract.capability_contract`

## Closed Plugin Contract Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.plugin_contract.taxonomy_extension` | Closed with `stop_split: true`; owns `PluginKind`, `ExtensionPoint`, and their mapping impls. |

## Open Plugin Contract Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.plugin_contract.capability_contract` | Selected next. Owns capability declaration DTO, capability contract enum, parser, string mapping, and capability version constant. |
| `contracts.qrpc_core.plugin_contract.execution_security_dependency` | Queued. Owns execution, compatibility, security, and dependency DTOs. |
| `contracts.qrpc_core.plugin_contract.manifest_validation` | Queued. Owns manifest structure and validation logic. |
| `contracts.qrpc_core.plugin_contract.registry` | Queued. Owns in-memory registry behavior. |

## Selection Rationale

`contracts.qrpc_core.plugin_contract.capability_contract` is selected because taxonomy now depends on a parent-owned capability enum and the next clean split is to give that enum its own child:

- physical region: `qrpc_core/src/plugin.rs`;
- public version constant: `PLUGIN_CAPABILITY_CONTRACT_V1_VERSION`;
- public DTO: `PluginCapabilityDeclaration`;
- public enum: `PluginCapabilityContract`;
- public methods: `PluginCapabilityContract::{as_str,parse}`;
- no ownership of manifest validation body, registry storage, execution/security/dependency DTOs, physical `plugins/*`, Strategy IR, or runtime behavior.

## Hard Boundaries

The next `root.contracts.qrpc_core.plugin_contract.capability_contract` baseline must not:

- edit Rust source code;
- change capability contract variants, serde rename rules, capability string literals, parser behavior, capability version value, or tests;
- change `PluginKind`, `ExtensionPoint`, `PluginManifest::validate`, `PluginRegistry`, execution/security/dependency DTOs, manifest fields, or manifest version constant;
- move physical plugin registry placeholders from `plugins/*`;
- change Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Next Step

BE-001PX-01 `root.contracts.qrpc_core.plugin_contract.capability_contract` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
