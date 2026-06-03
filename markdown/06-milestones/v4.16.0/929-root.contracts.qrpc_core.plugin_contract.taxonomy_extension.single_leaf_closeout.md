# v4.16.0 root.contracts.qrpc_core.plugin_contract.taxonomy_extension single leaf closeout

> Batch: BE-001PV-03
> Node: `root.contracts.qrpc_core.plugin_contract.taxonomy_extension`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.taxonomy_extension` has been evaluated after BE-001PV-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: plugin kind taxonomy and extension-point mapping.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is now isolated in `qrpc_core/src/plugin/taxonomy_extension.rs`. |
| Public method count | Stop. The public methods are only mapping helpers for the same enum family. |
| Mixed responsibility | Stop. Manifest validation, capability parsing, DTO schema, and registry behavior are outside this child. |
| Parent-mediated dependency | Covered. Capability mapping reaches `PluginCapabilityContract` through the plugin contract parent. |
| Future reopen rule | Allowed only when a concrete PluginKind, ExtensionPoint, mapping string, serde shape, or taxonomy/capability mapping change is proposed. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Plugin taxonomy or extension-point maintenance proposal | `contracts.qrpc_core.plugin_contract.taxonomy_extension` | Updated or verified `PluginKind`, `ExtensionPoint`, and mapping impls |

The leaf may describe and guard:

- `PluginKind` variants and serde names;
- `PluginKind::as_str`;
- `PluginKind::supported_extension_points`;
- `PluginKind::supported_capability_contracts`;
- `ExtensionPoint` variants and serde names;
- `ExtensionPoint::as_str`.

## Non-Claims

This closeout does not claim:

- manifest validation logic changed;
- capability contract parser or strings changed;
- execution/security/dependency DTOs changed;
- registry behavior changed;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001PW-01 `root.contracts.qrpc_core.plugin_contract` parent_residual_judgment selects `contracts.qrpc_core.plugin_contract.capability_contract`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
