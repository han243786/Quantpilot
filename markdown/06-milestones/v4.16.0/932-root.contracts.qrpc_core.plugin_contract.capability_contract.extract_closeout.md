# v4.16.0 root.contracts.qrpc_core.plugin_contract.capability_contract extract closeout

> Batch: BE-001PX-02
> Node: `root.contracts.qrpc_core.plugin_contract.capability_contract`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `extract_closeout`
> Movement: Rust code moved under the plugin contract parent.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.capability_contract` has been physically extracted from the plugin contract parent into a private child module.

Moved code:

- `PLUGIN_CAPABILITY_CONTRACT_V1_VERSION`;
- `PluginCapabilityDeclaration`;
- `PluginCapabilityContract`;
- `PluginCapabilityContract::{as_str,parse}`.

New child owner:

- `qrpc_core/src/plugin/capability_contract.rs`

Parent facade:

- `qrpc_core/src/plugin.rs`

## Equivalence Claim

The public contract remains equivalent:

- `qrpc_core::plugin::PLUGIN_CAPABILITY_CONTRACT_V1_VERSION` remains exported through the plugin contract parent;
- `qrpc_core::PLUGIN_CAPABILITY_CONTRACT_V1_VERSION` remains exported through `qrpc_core/src/lib.rs` via the existing `pub use plugin::*`;
- `PluginCapabilityDeclaration` and `PluginCapabilityContract` follow the same export path;
- capability version value, serde rename rules, enum variants, capability strings, and parser behavior are unchanged;
- `PluginManifest::validate` continues to use the same capability parser, strings, and version constant;
- closed `taxonomy_extension` continues to reach `PluginCapabilityContract` through the plugin contract parent.

## Parent-Child Rule

Allowed dependency preserved:

- taxonomy child -> plugin contract parent re-export `PluginCapabilityContract`;
- manifest validation parent -> plugin contract capability child public surface.

No direct sibling import was introduced. Manifest validation and taxonomy both communicate through the plugin contract parent.

## Files Changed

| File | Change |
| --- | --- |
| `qrpc_core/src/plugin.rs` | Added private child module declaration and public re-export; removed capability code now owned by the child. |
| `qrpc_core/src/plugin/capability_contract.rs` | Added extracted capability version, declaration DTO, enum, and parser/string impl owner. |

## Non-Claims

This extraction does not claim:

- manifest validation logic changed;
- taxonomy mapping changed;
- execution/security/dependency DTOs changed;
- registry behavior changed;
- tests were rewritten;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001PX-03 `root.contracts.qrpc_core.plugin_contract.capability_contract` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
