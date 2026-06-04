# v4.16.0 root.contracts.qrpc_core.plugin_contract.manifest_validation baseline plan

> Batch: BE-001QB-01
> Node: `root.contracts.qrpc_core.plugin_contract.manifest_validation`
> Parent: `root.contracts.qrpc_core.plugin_contract`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract.manifest_validation` is frozen as the plugin manifest schema and validation owner after BE-001QA-01 selection.

BE-001QB-01 does not move code. It defines the exact baseline and allowed movement for BE-001QB-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/plugin.rs`

Current selected boundary:

- `PLUGIN_MANIFEST_V1_VERSION`;
- `PluginType`;
- `AtomRef`;
- `PluginDisplay`;
- `PluginManifest`;
- `PluginManifest::validate`.

Current parent callers:

- `PluginRegistry::register` calls `PluginManifest::validate`;
- tests construct `PluginManifest` and call `validate`;
- runtime/plugin callers receive the same manifest DTO through `qrpc_core::*` and `qrpc_core::plugin::*`.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Stop after extraction. The child owns one public behavior method: `PluginManifest::validate`. |
| Mixed responsibility | Stop after extraction. Manifest fields and validation rules form one manifest schema contract. |
| Physical owner | Continue now. The selected surface still lives in `qrpc_core/src/plugin.rs` and should be isolated. |
| Private helper pressure | Defer. Splitting each validation rule into separate leaves would fragment one caller-facing manifest contract. |
| Future reopen rule | Allowed only when a concrete manifest field, serde attribute, version string, validation rule, or validation error text change is proposed. |

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Manifest version | `PLUGIN_MANIFEST_V1_VERSION` remains `quantpilot/plugin-manifest/v1`. |
| Plugin type serde shape | `PluginType::{Atom,Suite}` keeps explicit `atom` and `suite` serde names. |
| Atom reference DTO | `AtomRef` keeps `atom_id`, `version`, and `kind`. |
| Display DTO | `PluginDisplay` keeps `name` and `summary`. |
| Manifest DTO | `PluginManifest` keeps all fields, serde defaults, `deny_unknown_fields`, and field ordering semantics. |
| Validation behavior | `validate()` keeps all existing rule conditions, duplicate detection, parser calls, entrypoint checks, dependency version checks, and error strings. |

## Allowed BE-001QB-02 Movement

BE-001QB-02 may:

- create `qrpc_core/src/plugin/manifest_validation.rs`;
- add a private `mod manifest_validation;` declaration in `qrpc_core/src/plugin.rs`;
- re-export the selected child surface from the plugin contract parent with `pub use manifest_validation::*;`;
- move only `PLUGIN_MANIFEST_V1_VERSION`, `PluginType`, `AtomRef`, `PluginDisplay`, `PluginManifest`, and `PluginManifest::validate` into the child module;
- import closed child surfaces through `super::{...}` from the plugin contract parent;
- keep all public imports from `qrpc_core::plugin::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001QB-02 Movement

BE-001QB-02 must not move or rewrite:

- closed taxonomy, capability, or execution/security/dependency child modules;
- `PluginRegistry` and registry tests;
- validation rule conditions or error text;
- physical `plugins/*`, Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Parent-Child Rule

Allowed call paths:

- plugin contract parent -> private `manifest_validation` child re-export;
- registry parent -> `PluginManifest::validate` through the parent-local public surface;
- manifest child -> closed child DTO/type surfaces through the plugin contract parent.

Forbidden call paths:

Any manifest child import from registry, runtime, plugin metadata, or future sibling modules that bypasses the plugin contract parent.

## Proof

BE-001QB-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001QB-02 `root.contracts.qrpc_core.plugin_contract.manifest_validation` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
