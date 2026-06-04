# v4.16.0 root.contracts.qrpc_core.plugin_contract.registry extract closeout

> Batch: BE-001QD-02
> Node: `root.contracts.qrpc_core.plugin_contract.registry`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `extract_closeout`
> Movement: Rust code moved under the plugin contract parent.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.registry` has been physically extracted from the plugin contract parent into a private child module.

Moved code:

- `PluginRegistry`;
- `PluginRegistry::register`;
- `PluginRegistry::get`;
- `PluginRegistry::remove`;
- `PluginRegistry::manifests_for_extension_point`;
- `PluginRegistry::manifests`.

New child owner:

- `qrpc_core/src/plugin/registry.rs`

Parent facade:

- `qrpc_core/src/plugin.rs`

## Equivalence Claim

The public contract remains equivalent:

- `qrpc_core::plugin::PluginRegistry` remains exported through the plugin contract parent;
- `qrpc_core::PluginRegistry` remains exported through `qrpc_core/src/lib.rs` via the existing `pub use plugin::*`;
- registration still validates first, rejects duplicate ids with the same error string, then inserts by manifest id;
- lookup, removal, extension-point filtering, and manifest list projection semantics are unchanged;
- tests continue exercising registry behavior through the same public import path.

## Parent-Child Rule

Allowed dependency preserved:

- registry child -> `PluginManifest` and `ExtensionPoint` through the plugin contract parent;
- plugin contract parent -> registry child re-export.

No direct sibling path import was introduced. The registry child does not import manifest validation by file path, runtime, plugin metadata, or future sibling modules.

## Files Changed

| File | Change |
| --- | --- |
| `qrpc_core/src/plugin.rs` | Added private registry child module declaration and public re-export; removed registry code now owned by the child. |
| `qrpc_core/src/plugin/registry.rs` | Added extracted in-memory registry owner and public registry methods. |

## Non-Claims

This extraction does not claim:

- registry behavior changed;
- manifest validation logic changed;
- taxonomy mapping changed;
- capability contract parser or strings changed;
- execution/security/dependency DTOs changed;
- tests were rewritten;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QD-03 `root.contracts.qrpc_core.plugin_contract.registry` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
