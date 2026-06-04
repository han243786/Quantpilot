# v4.16.0 root.contracts.qrpc_core.plugin_contract.registry baseline plan

> Batch: BE-001QD-01
> Node: `root.contracts.qrpc_core.plugin_contract.registry`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.registry` is frozen as the in-memory plugin registry behavior owner after BE-001QC-01 selection.

BE-001QD-01 does not move code. It defines the exact baseline and allowed movement for BE-001QD-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/plugin.rs`

Current selected boundary:

- `PluginRegistry`;
- `PluginRegistry::register`;
- `PluginRegistry::get`;
- `PluginRegistry::remove`;
- `PluginRegistry::manifests_for_extension_point`;
- `PluginRegistry::manifests`.

Current parent and child dependencies:

- `PluginRegistry` stores `PluginManifest` in a private `BTreeMap<String, PluginManifest>`;
- `register` calls `PluginManifest::validate` through the plugin contract parent surface;
- `manifests_for_extension_point` filters using `ExtensionPoint`;
- tests exercise registration, duplicate-id rejection, lookup, and extension-point filtering.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Stop after extraction. The child owns one cohesive registry API with five public methods. |
| Mixed responsibility | Stop after extraction. Registry storage and query behavior are one in-memory registry contract. |
| Physical owner | Continue now. The selected surface still lives in `qrpc_core/src/plugin.rs` and should be isolated. |
| Private helper pressure | Defer. No private helper split is needed until registry gains independent persistence, indexing, or lifecycle policy. |
| Future reopen rule | Allowed only when a concrete registry method signature, storage invariant, ordering rule, duplicate-id behavior, lookup/removal semantics, or extension-point filtering change is proposed. |

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Storage | `PluginRegistry` keeps a private `BTreeMap<String, PluginManifest>` and derives `Debug`, `Default`, and `Clone`. |
| Registration | `register` validates the manifest first, rejects duplicate ids with the existing error string, then inserts by manifest id. |
| Lookup | `get` returns `Option<&PluginManifest>` by plugin id without mutation. |
| Removal | `remove` returns `Option<PluginManifest>` after deleting by plugin id. |
| Extension filtering | `manifests_for_extension_point` returns manifests whose `extension_points` contains the requested `ExtensionPoint`. |
| List projection | `manifests` returns all manifest references in the current map iteration order. |

## Allowed BE-001QD-02 Movement

BE-001QD-02 may:

- create `qrpc_core/src/plugin/registry.rs`;
- add a private `mod registry;` declaration in `qrpc_core/src/plugin.rs`;
- re-export the selected child surface from the plugin contract parent with `pub use registry::*;`;
- move only `PluginRegistry` and its impl into the child module;
- import `BTreeMap`, `ExtensionPoint`, and `PluginManifest` through the child or plugin contract parent as needed;
- keep all public imports from `qrpc_core::plugin::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001QD-02 Movement

BE-001QD-02 must not move or rewrite:

- closed taxonomy, capability, execution/security/dependency, or manifest validation child modules;
- registry method signatures, validation call order, duplicate-id behavior, lookup semantics, removal semantics, filtering semantics, list projection semantics, or error text;
- plugin contract tests unless needed only to preserve module visibility;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Parent-Child Rule

Allowed call paths:

- plugin contract parent -> private `registry` child re-export;
- registry child -> `PluginManifest` and `ExtensionPoint` through the plugin contract parent surface;
- external callers -> `PluginRegistry` through `qrpc_core::plugin::*` or `qrpc_core::*`.

Forbidden call paths:

Any registry child import from manifest child file path, runtime, plugin metadata, or future sibling modules that bypasses the plugin contract parent.

## Proof

BE-001QD-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001QD-02 `root.contracts.qrpc_core.plugin_contract.registry` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
