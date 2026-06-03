# v4.16.0 root.contracts baseline plan

> Batch: BE-001PE-01
> Node: `root.contracts`
> Parent: `root`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts` is frozen as the next Rust-facing top-level parent after `root.backend` closeout.

This baseline does not change OpenAPI, AsyncAPI, QRPC, Core IR, compiler, runtime support, plugin, QuantScript, or executor-session behavior. It only creates the L2 white-box queue for recursive handling.

Decision:

`baseline_frozen: true`

Next selected child:

`root.contracts.api_surface`

## Physical Scope

| Child | Physical area | Current owner note |
| --- | --- | --- |
| `contracts.api_surface` | `contracts/openapi/root.yaml`, `contracts/asyncapi/runtime-events.yaml` | HTTP and runtime event schema surfaces. |
| `contracts.qrpc_core` | `qrpc_core/src/lib.rs`, `qrpc_core/src/error.rs`, `qrpc_core/src/strategy_ir.rs`, `qrpc_core/src/event_envelope.proto` | Runtime protocol structs, artifact/version constants, digest helpers, Strategy IR, event envelope proto, and core errors. |
| `contracts.core_ir` | `qrpc_core_ir/src/lib.rs`, `qrpc_core_ir/src/v4.rs` | Core IR and v4 machine graph/backtest artifact data contracts. |
| `contracts.compiler_bridge` | `qrpc_compiler/src/lib.rs` | Runtime protocol and Strategy IR validation/lowering into Core IR and compiled artifacts. |
| `contracts.runtime_support` | `qrpc_runtime/src/*` | Runtime support library, coordinator, sandbox, v4 runtime types, slippage, hotswap, plugin runtime registry, and compatibility support. |
| `contracts.quantscript` | `quantscript/src/*`, `quantscript/*.md`, `quantscript/*.qs`, `quantscript/authoring_samples/*`, `quantscript/boundary_samples/*` | Formal QuantScript parser, HIR, lowering, diagnostics, static audit, handoff report, and authoring samples. |
| `contracts.plugin_metadata` | `qrpc_core/src/plugin.rs`, `plugins/*` | Plugin manifest/metadata contract and physical plugin registry placeholders. |

## Key Public Surfaces To Track

These are not moved in this batch. They are recorded so the next child baselines must cover public methods and structs instead of only file paths.

| Child | Public surface examples |
| --- | --- |
| `contracts.qrpc_core` | version constants such as `RUN_SPEC_V1_VERSION`, `BACKTEST_SPEC_V1_VERSION`, `CORE_IR_ARTIFACT_V1_VERSION`; `Symbol::parse`, `Symbol::as_str`; `canonical_json_sha256_digest`; `RunSpec::from_runtime_protocol`; `BacktestSpec::new`; `MarketDataSnapshotSpec::from_runtime_protocol`; exported Strategy IR and plugin contract types. |
| `contracts.core_ir` | `CoreStrategyIr::new`, `CoreStrategyIr::validate_dag`, expression helper builders such as `close_series_expr`, `moving_average_series_expr`, `moving_average_compare_expr`, and `indicator_threshold_compare_expr`; v4 Core IR exports. |
| `contracts.compiler_bridge` | `validate_runtime_protocol_config`, `compile_runtime_protocol_config`, `compile_runtime_protocol_config_with_metadata`, and `lower_runtime_protocol_to_core_ir`. |
| `contracts.runtime_support` | `RuntimeCoordinator::new`, `RuntimeCoordinator::from_core_ir`, module-provider constructors, `run_session`, `run_slow_cycle`, `run_fast_cycle`, execution submit/update methods, runtime state getters, risk-mode setters, hotswap/runtime support/slippage/v4 runtime exports. |
| `contracts.quantscript` | formal parser/analyzer/lowering exports such as `parse_quant_script_module`, `parse_formal_quant_script_config`, `parse_formal_quant_script_typed_hir`, `analyze_formal_quant_script`, `extract_formal_instrument_pool_spec`, `audit_v4_quant_script_static`, and compatibility QuantScript config functions. |
| `contracts.plugin_metadata` | plugin manifest structs re-exported from `qrpc_core::plugin`, plus `plugins/installed`, `plugins/disabled`, and `plugins/cache` registry placeholders. |

## Recursive Child Queue

| Order | Child | Stage to enter | Split note |
| --- | --- | --- | --- |
| 1 | `root.contracts.api_surface` | `single_leaf_closeout` | Smallest surface; pure schema registration; no code movement. |
| 2 | `root.contracts.qrpc_core` | `baseline_plan` | Large protocol type owner; likely needs recursive child judgment after baseline. |
| 3 | `root.contracts.core_ir` | `baseline_plan` | Core IR and v4 graph contracts need separate white-box tracking. |
| 4 | `root.contracts.compiler_bridge` | `baseline_plan` | Lowering/validation bridge; must not change runtime protocol semantics. |
| 5 | `root.contracts.runtime_support` | `baseline_plan` | Large runtime support crate; must not steal backend or executor state ownership. |
| 6 | `root.contracts.quantscript` | `baseline_plan` | Large QS language contract; likely recursive. |
| 7 | `root.contracts.plugin_metadata` | `baseline_plan` | Metadata and physical registry placeholders; no plugin runtime behavior change. |

## Hard Boundaries

This baseline must not:

- change OpenAPI or AsyncAPI schema semantics;
- change QRPC, Core IR, plugin, or QuantScript data structures;
- change compiler, runtime, or parser behavior;
- move `qrpc_session` or executor session ownership into contracts;
- move backend AppState, route, lock-order, storage, or test ownership;
- start frontend extraction or E2E cleanup;
- introduce release transition sibling links.

## Equivalence Evidence

No source code is changed in this batch. Equivalence is proven by unchanged source files plus the standard gates:

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`

## Next Step

BE-001PF-01 `root.contracts` parent_residual_judgment selects `root.contracts.api_surface`.
