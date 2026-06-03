# v4.16.0 root.contracts.qrpc_core.plugin_contract baseline plan

> Batch: BE-001PT-01
> Node: `root.contracts.qrpc_core.plugin_contract`
> Parent: `root.contracts.qrpc_core`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.plugin_contract` is frozen as the Rust plugin contract owner after BE-001PS-01 selection.

Decision:

`baseline_frozen: true`

Next candidate:

`root.contracts.qrpc_core.plugin_contract.taxonomy_extension`

## Current Owner

Current physical owner:

- `qrpc_core/src/plugin.rs`

Current selected boundary:

- plugin manifest version constants;
- plugin manifest and atom/suite metadata structures;
- manifest validation rules;
- plugin kind taxonomy and extension point mapping;
- capability contract strings and parser;
- execution, compatibility, security, and dependency DTOs;
- in-memory plugin registry behavior;
- local plugin contract tests.

Physical `plugins/*` registry placeholder directories remain queued under `contracts.plugin_metadata`; they are not owned by this node.

## Key Public Surfaces To Track

| Surface | Public contract |
| --- | --- |
| Manifest validation | `PLUGIN_MANIFEST_V1_VERSION`, `PLUGIN_CAPABILITY_CONTRACT_V1_VERSION`, `PluginType`, `AtomRef`, `PluginManifest`, `PluginManifest::validate`, `PluginDisplay`. |
| Taxonomy and extension points | `PluginKind::{as_str,supported_extension_points,supported_capability_contracts}`, `ExtensionPoint::as_str`. |
| Capability contract | `PluginCapabilityDeclaration`, `PluginCapabilityContract::{as_str,parse}`. |
| Execution and security DTOs | `PluginExecution`, `PluginExecutionEngine`, `PluginCompatibility`, `PluginSecurity`, `PluginDependency`. |
| Registry | `PluginRegistry::{register,get,remove,manifests_for_extension_point,manifests}`. |

## Recursive Child Queue

| Order | Child | Stage to enter | Split note |
| --- | --- | --- | --- |
| 1 | `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` | `baseline_plan` | Compact enum/mapping owner; good first extraction candidate. |
| 2 | `root.contracts.qrpc_core.plugin_contract.capability_contract` | `baseline_plan` | Capability IDs, parser, and declaration DTOs. |
| 3 | `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` | `baseline_plan` | Execution engine, compatibility, security, and dependency DTOs. |
| 4 | `root.contracts.qrpc_core.plugin_contract.manifest_validation` | `baseline_plan` | Manifest structure and validation logic; depends on taxonomy/capability DTO surfaces. |
| 5 | `root.contracts.qrpc_core.plugin_contract.registry` | `baseline_plan` | In-memory registry behavior around manifest validation and lookup. |

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Continue split. The node owns multiple independent public methods and validation entrypoints. |
| Mixed responsibility | Continue split. Taxonomy mapping, capability parsing, DTO schema, validation, and registry behavior can be reasoned about independently. |
| Physical owner | Continue split. A single file currently contains several white-box owners. |
| Parent-child communication | Hard rule. Children must communicate through the `plugin_contract` parent facade and public re-exports, not through sibling imports. |
| Release transition | Closed. No sibling shortcut or performance connection may be proposed without an explicit developer release-transition decision. |

## Allowed Future Movement

Future extraction steps may:

- introduce private child modules under `qrpc_core/src/plugin/` or an equivalent `plugin_contract` module layout;
- keep `qrpc_core/src/plugin.rs` or `qrpc_core/src/plugin/mod.rs` as the parent facade;
- move one selected child owner at a time while preserving all public re-exports from `qrpc_core::plugin::*` and `qrpc_core::*`;
- move or colocate tests only when the selected child owns the tested behavior;
- preserve every serde shape, version constant, validation error condition, registry ordering, and public method signature.

## Forbidden Movement

This baseline and its immediate child selection must not:

- edit Rust source code;
- change manifest fields, serde attributes, version strings, validation rules, registry behavior, or tests;
- move physical plugin registry placeholders from `plugins/*`;
- change Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor/frontend behavior, or release transition;
- create direct child-to-child imports that bypass the plugin contract parent.

## Equivalence Evidence

No Rust source is changed in this batch. Equivalence is proven by unchanged source files plus the standard gates:

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`

## Next Step

BE-001PU-01 `root.contracts.qrpc_core.plugin_contract` parent_residual_judgment selects `contracts.qrpc_core.plugin_contract.taxonomy_extension`.
