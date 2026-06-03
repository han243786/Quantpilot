# v4.16.0 root.contracts.qrpc_core.plugin_contract.capability_contract single leaf closeout

> Batch: BE-001PX-03
> Node: `root.contracts.qrpc_core.plugin_contract.capability_contract`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.capability_contract` has been evaluated after BE-001PX-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: plugin capability contract identity, version, declaration DTO, parser, and string mapping.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is now isolated in `qrpc_core/src/plugin/capability_contract.rs`. |
| Public method count | Stop. The public methods are only string/parser helpers for the same enum family. |
| Mixed responsibility | Stop. Manifest validation, taxonomy mapping, DTO schema outside capability declaration, and registry behavior are outside this child. |
| Parent-mediated dependency | Covered. Manifest validation and taxonomy reach capability items through the plugin contract parent. |
| Future reopen rule | Allowed only when a concrete capability ID, parser rule, capability version, declaration DTO, serde shape, or capability string change is proposed. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Plugin capability contract maintenance proposal | `contracts.qrpc_core.plugin_contract.capability_contract` | Updated or verified capability version, declaration DTO, enum, parser, and string impl |

The leaf may describe and guard:

- `PLUGIN_CAPABILITY_CONTRACT_V1_VERSION`;
- `PluginCapabilityDeclaration`;
- `PluginCapabilityContract` variants and serde names;
- `PluginCapabilityContract::as_str`;
- `PluginCapabilityContract::parse`.

## Non-Claims

This closeout does not claim:

- manifest validation logic changed;
- taxonomy mapping changed;
- execution/security/dependency DTOs changed;
- registry behavior changed;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001PY-01 `root.contracts.qrpc_core.plugin_contract` parent_residual_judgment selects `contracts.qrpc_core.plugin_contract.execution_security_dependency`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
