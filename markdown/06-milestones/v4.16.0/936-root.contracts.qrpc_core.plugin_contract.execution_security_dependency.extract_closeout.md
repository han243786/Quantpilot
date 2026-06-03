# v4.16.0 root.contracts.qrpc_core.plugin_contract.execution_security_dependency extract closeout

> Batch: BE-001PZ-02
> Node: `root.contracts.qrpc_core.plugin_contract.execution_security_dependency`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `extract_closeout`
> Movement: Rust code moved under the plugin contract parent.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.execution_security_dependency` has been physically extracted from the plugin contract parent into a private child module.

Moved code:

- `PluginExecution`;
- `PluginExecutionEngine`;
- `PluginCompatibility`;
- `PluginSecurity`;
- `PluginDependency`.

New child owner:

- `qrpc_core/src/plugin/execution_security_dependency.rs`

Parent facade:

- `qrpc_core/src/plugin.rs`

## Equivalence Claim

The public contract remains equivalent:

- `qrpc_core::plugin::PluginExecution` and related DTOs remain exported through the plugin contract parent;
- `qrpc_core::PluginExecution` and related DTOs remain exported through `qrpc_core/src/lib.rs` via the existing `pub use plugin::*`;
- execution engine serde names, security serde defaults, DTO field names, and dependency fields are unchanged;
- `PluginManifest::validate` continues to own and execute all validation logic for the selected DTO fields;
- tests continue constructing the same DTOs through the same public import path.

## Parent-Child Rule

Allowed dependency preserved:

- manifest parent -> selected DTO child through parent re-export.

No direct sibling import was introduced. The DTO child does not import taxonomy, capability, manifest validation, registry, or runtime siblings.

## Files Changed

| File | Change |
| --- | --- |
| `qrpc_core/src/plugin.rs` | Added private child module declaration and public re-export; removed DTO code now owned by the child. |
| `qrpc_core/src/plugin/execution_security_dependency.rs` | Added extracted execution, compatibility, security, and dependency DTO owner. |

## Non-Claims

This extraction does not claim:

- manifest validation logic changed;
- taxonomy mapping changed;
- capability contract parser or strings changed;
- registry behavior changed;
- tests were rewritten;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001PZ-03 `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
