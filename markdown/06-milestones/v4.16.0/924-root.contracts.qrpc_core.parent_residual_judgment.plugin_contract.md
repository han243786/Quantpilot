# v4.16.0 root.contracts.qrpc_core parent residual judgment selects plugin_contract

> Batch: BE-001PS-01
> Node: `root.contracts.qrpc_core`
> Selected child: `root.contracts.qrpc_core.plugin_contract`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core` returns to its child queue after `contracts.qrpc_core.event_envelope_proto` closeout.

Decision:

`next_child: root.contracts.qrpc_core.plugin_contract`

## Closed Qrpc Core Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.error_contract` | Closed with `stop_split: true`; owns `qrpc_core/src/error.rs`. |
| `contracts.qrpc_core.event_envelope_proto` | Closed with `stop_split: true`; owns `qrpc_core/src/event_envelope.proto`. |

## Open Qrpc Core Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.plugin_contract` | Selected next. Owns Rust plugin manifest, capability, extension point, execution, security, dependency, and registry contracts. |
| `contracts.qrpc_core.strategy_ir` | Queued. Owns Strategy IR structures and validation behavior. |
| `contracts.qrpc_core.protocol_primitives` | Queued. Owns primitives and version constants inside `lib.rs`. |
| `contracts.qrpc_core.runtime_protocol_config` | Queued. Owns runtime protocol config structures inside `lib.rs`. |
| `contracts.qrpc_core.artifact_specs` | Queued. Owns digest/run/backtest/artifact specs inside `lib.rs`. |
| `contracts.qrpc_core.runtime_io_contract` | Queued. Owns runtime DTO/output contracts inside `lib.rs`. |
| `contracts.qrpc_core.rfc_execution_contracts` | Queued. Owns RFC-style request/order/handoff contracts inside `lib.rs`. |

## Selection Rationale

`contracts.qrpc_core.plugin_contract` is selected because it is the next independent Rust contract file after the closed error and proto leaves:

- physical file: `qrpc_core/src/plugin.rs`;
- manifest/version constants: `PLUGIN_MANIFEST_V1_VERSION`, `PLUGIN_CAPABILITY_CONTRACT_V1_VERSION`;
- manifest and plugin metadata structures: `PluginManifest`, `PluginType`, `AtomRef`, `PluginDisplay`, `PluginDependency`;
- plugin taxonomy and capability structures: `PluginKind`, `ExtensionPoint`, `PluginCapabilityContract`, `PluginCapabilityDeclaration`;
- execution/security/compatibility structures: `PluginExecution`, `PluginExecutionEngine`, `PluginCompatibility`, `PluginSecurity`;
- registry structure and behavior: `PluginRegistry::{register,get,remove,manifests_for_extension_point,manifests}`.

This selection does not own physical plugin registry directories under `plugins/*`; those remain queued under `contracts.plugin_metadata`.

## Hard Boundaries

The next `root.contracts.qrpc_core.plugin_contract` baseline must not:

- edit `qrpc_core/src/plugin.rs`;
- change manifest fields, serde shape, version constants, validation rules, registry behavior, or tests;
- move physical plugin registry placeholders from `plugins/*`;
- change Strategy IR, event proto, `lib.rs` protocol DTOs, compiler/runtime/backend/executor behavior, or release transition.

## Next Step

BE-001PT-01 `root.contracts.qrpc_core.plugin_contract` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
