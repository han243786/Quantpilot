# v4.16.0 root.contracts.qrpc_core.plugin_contract.taxonomy_extension extract closeout

> Batch: BE-001PV-02
> Node: `root.contracts.qrpc_core.plugin_contract.taxonomy_extension`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `extract_closeout`
> Movement: Rust code moved under the plugin contract parent.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.taxonomy_extension` has been physically extracted from the plugin contract parent into a private child module.

Moved code:

- `PluginKind`;
- `PluginKind::{as_str,supported_extension_points,supported_capability_contracts}`;
- `ExtensionPoint`;
- `ExtensionPoint::as_str`.

New child owner:

- `qrpc_core/src/plugin/taxonomy_extension.rs`

Parent facade:

- `qrpc_core/src/plugin.rs`

## Equivalence Claim

The public contract remains equivalent:

- `qrpc_core::plugin::PluginKind` remains exported through the plugin contract parent;
- `qrpc_core::PluginKind` remains exported through `qrpc_core/src/lib.rs` via the existing `pub use plugin::*`;
- `ExtensionPoint` follows the same export path;
- serde rename rules, enum variants, mapping strings, and supported extension/capability mappings are unchanged;
- `PluginManifest::validate` and `PluginRegistry` behavior are unchanged.

## Parent-Child Rule

Allowed dependency preserved:

- taxonomy child -> plugin contract parent type `PluginCapabilityContract` through `super::PluginCapabilityContract`.

No direct sibling import was introduced. The capability contract still belongs to the parent residual queue until its own child extraction.

## Files Changed

| File | Change |
| --- | --- |
| `qrpc_core/src/plugin.rs` | Added private child module declaration and public re-export; removed the taxonomy/extension code now owned by the child. |
| `qrpc_core/src/plugin/taxonomy_extension.rs` | Added extracted taxonomy and extension-point contract owner. |

## Non-Claims

This extraction does not claim:

- manifest validation logic changed;
- capability contract parser or strings changed;
- execution/security/dependency DTOs changed;
- registry behavior changed;
- tests were rewritten;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001PV-03 `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
