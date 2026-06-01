# v4.16.0 backend.graph_compile.compile equivalence baseline and extraction plan

> Batch: BE-001HI-01
> Node: `backend.graph_compile.compile`
> Parent: `backend.graph_compile`
> Stage: `baseline_plan`
> Movement: no code movement.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001HI-02 `backend.graph_compile.compile` `extract_closeout`

---

## Summary

This baseline freezes the compile route/API residual before moving it out of
the old root-level `src/compile_api.rs` owner. The next movement may extract
only compile route registration, compile handlers, compile cache/semaphore, and
the QS runtime protocol helper into `src/backend/graph_compile/compile.rs`.

`backend.graph_compile.graph` and `src/graph_api.rs` remain untouched.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001HI-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / compile route boundary | compile residual freeze |
| Guidance matrix | `root.backend.graph_compile.compile` | planned real owner |
| Module tree | `backend.graph_compile -> compile` | child edge becomes real implementation owner |

---

## Equivalence Baseline

Frozen owner before extraction:

```text
src/compile_api.rs
```

Frozen child facade:

```text
src/backend/graph_compile/compile.rs
```

Frozen public route surface:

```text
POST /api/runtime/compile
POST /api/strategy-ir/compile
POST /api/quantscript/formal/compile
```

Frozen parent-facing helpers:

```text
register_compile_routes
compile_runtime_protocol_via_qs
```

Frozen private implementation cluster:

```text
COMPILE_SEMAPHORE
COMPILE_CACHE
CompileCacheEntry
compute_compile_cache_key
compile_runtime_request
graph_json_from_runtime_config
compile_strategy_ir_request
compile_formal_quantscript_request
```

Frozen behavior:

```text
Runtime compile keeps cache-key semantics, compile semaphore timeout behavior, capability validation, contract diagnostics, QS-only runtime protocol path, artifact bundle generation, and CompileRuntimeResponse shape.
Strategy IR compile keeps restricted custom lowering, diagnostic mapping, core IR compile, artifact generation, and response shape.
Formal QuantScript compile keeps parse/analyze/lower diagnostic mapping, authoring view projection, artifact generation, and response shape.
External callers of compile_runtime_protocol_via_qs continue to compile the same graph JSON through QS generation -> parse -> formal conversion -> runtime lowering.
```

Frozen non-goals:

```text
No graph_api movement.
No graph persistence/version/audit/reveal movement.
No quantscript_graph child movement.
No compile diagnostics or artifact builder semantic changes.
No public API path, schema, lock order, persistence root, or state-machine change.
No sibling horizontal connection.
No release-transition optimization.
```

---

## Extraction Plan

Planned real owner:

```text
src/backend/graph_compile/compile.rs
```

Planned old-owner result:

```text
src/compile_api.rs removed, unless a compile-time compatibility shim is proven necessary during BE-001HI-02.
```

Planned parent/root wiring:

```rust
pub mod compile;

pub(crate) fn register_compile_routes(router: Router<AppState>) -> Router<AppState> {
    compile::register_routes(router)
}

pub(crate) use backend::graph_compile::compile::compile_runtime_protocol_via_qs;
```

Planned child surface:

```rust
pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState>
pub(crate) fn compile_runtime_protocol_via_qs(
    graph_json: &Value,
) -> Result<RuntimeProtocolCoreConfig, (StatusCode, String)>
```

Planned caller updates:

```text
route facade callers continue through backend.graph_compile -> compile.
runtime/backtest/run callers use the root parent re-export.
migration_sender uses the same root parent re-export.
```

The child owns only:

```text
compile route registration
runtime compile handler
strategy IR compile handler
formal QuantScript compile handler
compile cache/semaphore helper state
compile_runtime_protocol_via_qs
compile_api local tests
```

The parent/root keeps ownership of:

```text
backend.graph_compile route group mediation
root-level compatibility exports for existing non-route callers
compile diagnostics and artifact builder modules
quantscript_graph parent wrappers
graph_api residual
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid:

1. The child is already named and isolated as `backend.graph_compile.compile`.
2. The old owner has a compact route/API surface with clear public helpers.
3. The QS graph dependency is already closed and available through parent/root exports.
4. Existing focused compile tests can verify cache, formal QS compile, and runtime compile behavior.

## Boundary

**Real files**:
- `src/backend/graph_compile/compile.rs`
- `src/compile_api.rs`
- `src/lib.rs`
- `src/migration_sender.rs`
- `src/runtime/backtest/legacy_dispatch.rs`
- `src/runtime/backtest/start_orchestration.rs`
- `src/runtime/backtest/v4_request_resolution.rs`
- `src/runtime/run/session_start.rs`

**Markers**:
- `compile baseline_frozen`
- `compile plan_frozen`
- `graph_residual_unchanged`

**Next step**:
BE-001HI-02 backend.graph_compile.compile extract_closeout

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
