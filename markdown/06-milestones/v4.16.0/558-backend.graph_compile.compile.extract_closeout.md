# v4.16.0 backend.graph_compile.compile actual extraction and closeout complete

> Batch: BE-001HI-02
> Node: `backend.graph_compile.compile`
> Parent: `backend.graph_compile`
> Stage: `extract_closeout`
> Movement: code movement.

---

## Summary

backend.graph_compile.compile actual extraction and closeout complete.

`src/backend/graph_compile/compile.rs` now owns the compile route/API implementation.
The old root-level `src/compile_api.rs` implementation owner was reduced to a
compatibility marker so existing public module imports still compile.

The root parent keeps only a controlled crate-level export of
`compile_runtime_protocol_via_qs` so runtime/backtest/run/migration callers can
continue using the same graph JSON -> QS -> runtime protocol path without adding
direct sibling links.

---

## Movement

Moved into `src/backend/graph_compile/compile.rs`:

```text
compile route registration
COMPILE_SEMAPHORE
COMPILE_CACHE
CompileCacheEntry
compute_compile_cache_key
compile_runtime_protocol_via_qs
compile_runtime_request
graph_json_from_runtime_config
compile_strategy_ir_request
compile_formal_quantscript_request
compile local cache test
```

Adjusted root wiring:

```text
src/lib.rs keeps pub mod compile_api as an empty compatibility marker
src/lib.rs exports backend.graph_compile.compile::compile_runtime_protocol_via_qs for crate callers
src/migration_sender.rs uses the root parent export
```

Compatibility marker kept:

```text
src/compile_api.rs
```

Unchanged residual:

```text
src/graph_api.rs
```

---

## Equivalence Evidence

Route equivalence:

```text
backend.graph_compile.register_compile_routes -> compile::register_routes
POST /api/runtime/compile
POST /api/strategy-ir/compile
POST /api/quantscript/formal/compile
```

Helper equivalence:

```text
compile_runtime_protocol_via_qs keeps QS generation, parse, graph-to-module conversion, and runtime lowering.
runtime/backtest/run/migration callers keep using the crate root parent export.
```

Parent-child communication:

```text
compile child calls quantscript_graph helpers only through root/parent-controlled exports.
No direct connection to backend.graph_compile.graph or graph_api was added.
No release-transition optimization was introduced.
```

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | `backend.graph_compile.compile` now owns compile route/API behavior. |
| parent_child_communication_kept | pass | Route registration still flows through `backend.graph_compile`; non-route compile helper callers use the crate root parent export. |
| equivalence_baseline_freezable | pass | BE-001HI-01 baseline froze route paths, helper behavior, cache/semaphore behavior, and compile response shapes before movement. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | closed | Runtime compile, strategy IR compile, and formal QS compile handlers are now in the child owner. |
| state_machine_phase | false | No remaining compile state-machine phase is left in the parent wrapper. |
| strategy_branch | closed | The three compile branches remain together as one compile API owner. |
| independent_failure_mode | closed | Compile diagnostics and formal QS errors remain in the compile child. |
| reuse_pressure | closed | `compile_runtime_protocol_via_qs` is exported through the root parent instead of old root owner module. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | true | Further split inside compile would require a new child decision because runtime, strategy IR, and formal QS compile branches each carry handler behavior. |
| communication_cost_rises | false | Current extraction reduces old-owner delegation and does not add sibling calls. |
| local_proof_missing | false | `cargo check` passed before closeout; targeted cargo tests are listed as gates. |
| line_count_only | true | Any further split must be branch/handler driven, not line-count driven. |

leaf_split_decision_result

`stop_split: true` for `backend.graph_compile.compile` at this level.

Further internal compile branch splitting requires a future parent residual judgment.

next_recursive_step

BE-001HJ-01 backend.graph_compile parent residual judgment

## Boundary

**Real files**:
- `src/backend/graph_compile/compile.rs`
- `src/compile_api.rs`
- `src/lib.rs`
- `src/migration_sender.rs`
- `src/backend/graph_compile.rs`

**Markers**:
- `compile actual_extraction_done`
- `compile closeout_done`
- `compile stop_split: true`
- `compile_api old implementation owner removed`
- `compile_api compatibility marker kept`

**Next step**:
BE-001HJ-01 backend.graph_compile parent residual judgment

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot compile_cache_key_includes_runtime_config --lib`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_returns_success`
- `cargo test -p quantpilot compile_endpoint_returns_artifact_bundle`
