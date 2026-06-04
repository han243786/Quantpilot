# v4.16.0 root.contracts.qrpc_core.plugin_contract.manifest_validation extract closeout

> Batch: BE-001QB-02
> Node: `root.contracts.qrpc_core.plugin_contract.manifest_validation`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `extract_closeout`
> Movement: Rust code moved under the plugin contract parent.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.manifest_validation` has been physically extracted from the plugin contract parent into a private child module.

Moved code:

- `PLUGIN_MANIFEST_V1_VERSION`;
- `PluginType`;
- `AtomRef`;
- `PluginDisplay`;
- `PluginManifest`;
- `PluginManifest::validate`.

New child owner:

- `qrpc_core/src/plugin/manifest_validation.rs`

Parent facade:

- `qrpc_core/src/plugin.rs`

## Equivalence Claim

The public contract remains equivalent:

- `qrpc_core::plugin::PluginManifest` and related manifest surfaces remain exported through the plugin contract parent;
- `qrpc_core::PluginManifest` and related manifest surfaces remain exported through `qrpc_core/src/lib.rs` via the existing `pub use plugin::*`;
- manifest version, field shape, serde defaults, `deny_unknown_fields`, validation conditions, and validation error strings are unchanged;
- `PluginRegistry::register` continues to call `PluginManifest::validate`;
- tests continue constructing and validating manifests through the same public import path.

## Parent-Child Rule

Allowed dependency preserved:

- manifest child -> closed taxonomy/capability/execution DTO surfaces through the plugin contract parent;
- registry parent -> manifest validation through the plugin contract parent re-export.

No direct sibling import was introduced. The manifest child does not import registry, runtime, plugin metadata, or future sibling modules.

## Files Changed

| File | Change |
| --- | --- |
| `qrpc_core/src/plugin.rs` | Added private child module declaration and public re-export; removed manifest schema and validation code now owned by the child. |
| `qrpc_core/src/plugin/manifest_validation.rs` | Added extracted manifest schema DTOs, version constant, and `PluginManifest::validate` owner. |

## Non-Claims

This extraction does not claim:

- validation logic changed;
- taxonomy mapping changed;
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

BE-001QB-03 `root.contracts.qrpc_core.plugin_contract.manifest_validation` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
