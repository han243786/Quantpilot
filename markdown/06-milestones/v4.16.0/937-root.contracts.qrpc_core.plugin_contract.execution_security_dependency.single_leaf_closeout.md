# v4.16.0 root.contracts.qrpc_core.plugin_contract.execution_security_dependency single leaf closeout

> Batch: BE-001PZ-03
> Node: `root.contracts.qrpc_core.plugin_contract.execution_security_dependency`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.execution_security_dependency` has been evaluated after BE-001PZ-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: plugin execution, compatibility, security, and dependency DTO shape.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is now isolated in `qrpc_core/src/plugin/execution_security_dependency.rs`. |
| Public method count | Stop. This child owns DTOs and one enum only; it has no behavior methods. |
| Mixed responsibility | Stop. Manifest validation, taxonomy mapping, capability parsing, and registry behavior are outside this child. |
| Parent-mediated dependency | Covered. Manifest validation reaches the DTOs through the plugin contract parent re-export. |
| Future reopen rule | Allowed only when a concrete execution engine, compatibility field, security field/default, dependency field, or DTO serde shape change is proposed. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Plugin execution/security/dependency DTO maintenance proposal | `contracts.qrpc_core.plugin_contract.execution_security_dependency` | Updated or verified execution, compatibility, security, and dependency DTO shape |

The leaf may describe and guard:

- `PluginExecution`;
- `PluginExecutionEngine`;
- `PluginCompatibility`;
- `PluginSecurity`;
- `PluginDependency`.

## Non-Claims

This closeout does not claim:

- manifest validation logic changed;
- taxonomy mapping changed;
- capability contract parser or strings changed;
- registry behavior changed;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QA-01 `root.contracts.qrpc_core.plugin_contract` parent_residual_judgment selects `contracts.qrpc_core.plugin_contract.manifest_validation`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
