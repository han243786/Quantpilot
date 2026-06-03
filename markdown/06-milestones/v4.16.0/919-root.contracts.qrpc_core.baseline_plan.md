# v4.16.0 root.contracts.qrpc_core baseline plan

> Batch: BE-001PN-01
> Node: `root.contracts.qrpc_core`
> Parent: `root.contracts`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core` is frozen as the next contracts parent after BE-001PM-01 selection.

Decision:

`baseline_frozen: true`

Next selected child:

`root.contracts.qrpc_core.error_contract`

## Physical Scope

| Child | Physical area | Current owner note |
| --- | --- | --- |
| `contracts.qrpc_core.error_contract` | `qrpc_core/src/error.rs` | Typed core error enum, display/source behavior, and IO conversion. |
| `contracts.qrpc_core.event_envelope_proto` | `qrpc_core/src/event_envelope.proto` | Internal event envelope protobuf schema. |
| `contracts.qrpc_core.plugin_contract` | `qrpc_core/src/plugin.rs` | Rust plugin manifest, capability, extension point, execution, security, dependency, and registry contracts. |
| `contracts.qrpc_core.strategy_ir` | `qrpc_core/src/strategy_ir.rs` | Legacy Strategy IR structures, indicator kind surface, validation, gap annotations, and unknown preservation. |
| `contracts.qrpc_core.protocol_primitives` | `qrpc_core/src/lib.rs` | Version constants, exchange/symbol/market/runtime primitive enums, risk/source status primitives, and core defaults. |
| `contracts.qrpc_core.runtime_protocol_config` | `qrpc_core/src/lib.rs` | Runtime protocol config structs, global risk/profile defaults, and compiled runtime protocol container. |
| `contracts.qrpc_core.artifact_specs` | `qrpc_core/src/lib.rs` | Canonical JSON digest, run/backtest specs, strategy/core/compile artifact specs, and artifact bundle contracts. |
| `contracts.qrpc_core.runtime_io_contract` | `qrpc_core/src/lib.rs` | Market data snapshots, signals, decisions, execution plans, fill results, portfolio state, runtime events, sessions, and backtest output contracts. |
| `contracts.qrpc_core.rfc_execution_contracts` | `qrpc_core/src/lib.rs` | Data request, allocation, order, execution feedback, and handoff snapshot RFC-style contracts. |

`contracts.plugin_metadata` remains queued outside this baseline for the physical `plugins/*` registry placeholders. `contracts.qrpc_core.plugin_contract` only owns Rust manifest/capability data structures currently in `qrpc_core/src/plugin.rs`.

## Key Public Surfaces To Track

| Child | Public surface examples |
| --- | --- |
| `error_contract` | `QuantPilotError`, `Display`, `std::error::Error::source`, and `From<std::io::Error>`. |
| `event_envelope_proto` | `EventEnvelope`, `ChainStage`, `Severity`, `RetentionClass`. |
| `plugin_contract` | `PluginManifest::validate`, `PluginKind::as_str`, `PluginKind::supported_extension_points`, `PluginKind::supported_capability_contracts`, `ExtensionPoint::as_str`, `PluginCapabilityContract::as_str`, `PluginCapabilityContract::parse`, `PluginRegistry::{register,get,remove,manifests_for_extension_point,manifests}`. |
| `strategy_ir` | `KnownOrUnknown::is_unknown`, `StrategyIr::validation_errors`, `StrategyIr::validate`, `declared_indicator_kinds`, `supported_indicator_kinds`, public IR structs/enums. |
| `protocol_primitives` | version constants such as `RUN_SPEC_V1_VERSION` and `EVENT_ENVELOPE_PROTO_VERSION`, `Symbol::parse`, `Symbol::as_str`, primitive enums and defaults. |
| `runtime_protocol_config` | `RuntimeProtocolCoreConfig::default`, `CompiledRuntimeProtocol`, config structs and global/profile constants. |
| `artifact_specs` | `canonical_json_sha256_digest`, `DatasetSpec::from`, `ExecutionAssumptionSpec::from`, `MarketDataSnapshotSpec::from_runtime_protocol`, `RunSpec::from_runtime_protocol`, `BacktestSpec::new`, artifact structs. |
| `runtime_io_contract` | `PortfolioState::new`, `PortfolioState::debug_assert_invariants`, runtime/backtest output structs and execution state DTOs. |
| `rfc_execution_contracts` | `Allocation::apply_to_targets`, `OrderStatus::can_transition_to`, `HandoffSnapshot::validate_completeness`, RFC request/order/feedback structs. |

## Recursive Child Queue

| Order | Child | Stage to enter | Split note |
| --- | --- | --- | --- |
| 1 | `root.contracts.qrpc_core.error_contract` | `single_leaf_closeout` | Small typed error owner; likely `stop_split: true`. |
| 2 | `root.contracts.qrpc_core.event_envelope_proto` | `single_leaf_closeout` | Single proto schema owner; likely `stop_split: true`. |
| 3 | `root.contracts.qrpc_core.plugin_contract` | `baseline_plan` | Large Rust plugin contract; may need recursive children. |
| 4 | `root.contracts.qrpc_core.strategy_ir` | `baseline_plan` | Large Strategy IR and validation owner; likely recursive. |
| 5 | `root.contracts.qrpc_core.protocol_primitives` | `baseline_plan` | Extractable region inside `lib.rs`; must preserve serde and default semantics. |
| 6 | `root.contracts.qrpc_core.runtime_protocol_config` | `baseline_plan` | Config owner inside `lib.rs`; may become a physical module later. |
| 7 | `root.contracts.qrpc_core.artifact_specs` | `baseline_plan` | Digest and artifact spec owner inside `lib.rs`; high schema sensitivity. |
| 8 | `root.contracts.qrpc_core.runtime_io_contract` | `baseline_plan` | Large runtime DTO/output owner inside `lib.rs`. |
| 9 | `root.contracts.qrpc_core.rfc_execution_contracts` | `baseline_plan` | RFC-style request/order/handoff contracts inside `lib.rs`. |

## Hard Boundaries

This baseline must not:

- edit `qrpc_core/src/*`;
- change public struct/enum fields, serde attributes, version strings, validation behavior, digest behavior, proto schema, or tests;
- move plugin registry placeholder directories from `plugins/*`;
- change `qrpc_core_ir`, `qrpc_compiler`, `qrpc_runtime`, `quantscript`, backend, executor, frontend, E2E, or test asset behavior;
- introduce release transition sibling links.

## Equivalence Evidence

No Rust or proto code is changed in this batch. Equivalence is proven by unchanged source files plus the standard gates:

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`

## Next Step

BE-001PO-01 `root.contracts.qrpc_core` parent_residual_judgment selects `contracts.qrpc_core.error_contract`.
