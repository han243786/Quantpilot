# v4.16.0 root.contracts.qrpc_core.plugin_contract.manifest_validation single leaf closeout

> Batch: BE-001QB-03
> Node: `root.contracts.qrpc_core.plugin_contract.manifest_validation`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.manifest_validation` has been evaluated after BE-001QB-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: plugin manifest schema shape and manifest validation behavior.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is now isolated in `qrpc_core/src/plugin/manifest_validation.rs`. |
| Public method count | Stop. This child owns one public behavior method, `PluginManifest::validate`, and its supporting manifest DTOs. |
| Mixed responsibility | Stop. Taxonomy mapping, capability parsing, execution/security/dependency DTO shape, and registry behavior are outside this child. |
| Parent-mediated dependency | Covered. Manifest validation reaches closed DTO and taxonomy/capability surfaces through the plugin contract parent re-export. |
| Future reopen rule | Allowed only when a concrete manifest field, serde attribute, validation rule, manifest version constant, or validation error contract change is proposed. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Plugin manifest schema or validation proposal | `contracts.qrpc_core.plugin_contract.manifest_validation` | Updated or verified manifest DTO shape and `PluginManifest::validate` behavior |

The leaf may describe and guard:

- `PLUGIN_MANIFEST_V1_VERSION`;
- `PluginType`;
- `AtomRef`;
- `PluginDisplay`;
- `PluginManifest`;
- `PluginManifest::validate`.

## Non-Claims

This closeout does not claim:

- taxonomy mapping changed;
- capability contract parser, strings, or version changed;
- execution/security/dependency DTO shape changed;
- registry behavior changed;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QC-01 `root.contracts.qrpc_core.plugin_contract` parent_residual_judgment selects `contracts.qrpc_core.plugin_contract.registry`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
