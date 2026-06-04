# v4.16.0 root.contracts.qrpc_core.plugin_contract.registry single leaf closeout

> Batch: BE-001QD-03
> Node: `root.contracts.qrpc_core.plugin_contract.registry`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.registry` has been evaluated after BE-001QD-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: in-memory plugin registry storage and query behavior.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is now isolated in `qrpc_core/src/plugin/registry.rs`. |
| Public method count | Stop. The five public methods form one cohesive registry API, not five independent ownership leaves. |
| Mixed responsibility | Stop. Registration, lookup, deletion, filtering, and list projection are all part of the same in-memory registry contract. |
| Parent-mediated dependency | Covered. Registry reaches manifest and extension-point surfaces through the plugin contract parent re-export. |
| Future reopen rule | Allowed only when a concrete registry method signature, storage invariant, duplicate-id behavior, lookup/removal semantics, extension-point filtering, or list projection change is proposed. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Plugin registry behavior proposal | `contracts.qrpc_core.plugin_contract.registry` | Updated or verified in-memory plugin registry behavior |

The leaf may describe and guard:

- `PluginRegistry`;
- `PluginRegistry::register`;
- `PluginRegistry::get`;
- `PluginRegistry::remove`;
- `PluginRegistry::manifests_for_extension_point`;
- `PluginRegistry::manifests`.

## Non-Claims

This closeout does not claim:

- manifest validation logic changed;
- taxonomy mapping changed;
- capability contract parser, strings, or version changed;
- execution/security/dependency DTO shape changed;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QE-01 `root.contracts.qrpc_core.plugin_contract` parent_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
