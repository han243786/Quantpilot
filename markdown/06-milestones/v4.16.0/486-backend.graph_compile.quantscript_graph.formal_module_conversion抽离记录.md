# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion extraction record
> Version type: MINOR architecture / implementation
> Execution tier: standard
> Batch: BE-001FT-03
> Baseline: `485-backend.graph_compile.quantscript_graph.formal_module_conversion抽离方案.md`
> Target leaf: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Decision: actual extraction done
> Module tree coordinate: `root.backend.graph_compile.quantscript_graph.formal_module_conversion`
> Code action: actual extraction
> Next step: BE-001FT-04 `backend.graph_compile.quantscript_graph.formal_module_conversion` single leaf closeout

---

## Matrix Impact

| Matrix | Changed node | Change type |
| --- | --- | --- |
| Process matrix | BE-001FT-03 `formal_module_conversion` actual extraction | child actual extraction |
| Standard matrix | actual extraction / parent re-export / no sibling horizontal link / release transition guard | standard tier |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion` | child file landed |
| Module tree | `backend.graph_compile.quantscript_graph.formal_module_conversion` | conversion owner moved |

---

## Actual Move

This batch creates:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
```

and moves exactly one function from the parent:

```text
convert_graph_json_to_script_module
```

Markers:

```text
formal_module_conversion_file_created
formal_module_conversion actual_extraction_done
```

---

## Parent Wiring

The parent keeps the caller-facing surface through:

```text
src/backend/graph_compile/quantscript_graph.rs
```

```rust
mod formal_module_conversion;
pub(crate) use formal_module_conversion::convert_graph_json_to_script_module;
```

The `quantscript::{parse_quant_script_module, ScriptModule}` import moved to the child because the child now owns the terminal formal parser call.

Compile / graph / runtime siblings must still call through the parent re-export surface. BE-001FT-03 does not add any sibling horizontal link.

---

## Equivalent Invariants

This batch preserves:

1. `graph.nodes` array validation.
2. `graph.edges` array validation.
3. `data` node fetch lowering, including default exchange/instrument/timeframe/lookback.
4. `risk` node `risk.profile(...)` lowering.
5. `execution` node `execution.profile(...)` lowering.
6. ignored node types: `data`, `intent`, `agent`, `runtime`, `runtime_control`.
7. unknown non-ignored node logging via `safe_eprintln!`.
8. intent lowering for `builtin.intent.double_ma`.
9. intent lowering for `builtin.intent.rsi`.
10. intent lowering for `builtin.intent.ma_deviation`.
11. intent lowering for `builtin.intent.macd`.
12. intent lowering for `builtin.intent.momentum`.
13. intent lowering for `builtin.intent.zscore`.
14. intent lowering for `builtin.intent.spread_observer`.
15. unsupported intent module failure via `anyhow::bail!`.
16. terminal parse through `parse_quant_script_module(&qs_source)`.
17. `src/compile_api.rs` caller error mapping remains unchanged.
18. `src/lib.rs` root parent re-export surface remains unchanged.

---

## Out Of Scope

BE-001FT-03 does not move or alter:

```text
register_routes
load_graph_quantscript
parse_graph_quantscript
generate_quantscript_from_graph_value
graph_to_qs_generation child
attach_quantscript_artifacts
build_quantscript_node_sources
build_quantscript_label_targets
build_quantscript_runtime_targets
build_compile_runtime_targets_from_graph
parse_graph_quantscript_source
src/compile_api.rs
src/graph_api.rs
src/lib.rs
```

This batch does not start release transition guard and does not propose release-mode horizontal links.

---

## Next Boundary

The next step can only be:

```text
BE-001FT-04
backend.graph_compile.quantscript_graph.formal_module_conversion
root.backend.graph_compile.quantscript_graph.formal_module_conversion
```

BE-001FT-04 may only perform single leaf closeout and decide whether this leaf should split further. It must not jump to route surface, artifact target projection, strategy graph parser, `backend.graph_compile`, or release transition.

---

## Verification Requirements

Before committing this batch, run at least:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot quantscript --lib
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
```

---

## Hallucination Guard

AI claiming BE-001FT-03 is done must state:

1. `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs` exists.
2. Only `convert_graph_json_to_script_module` moved to the child.
3. Parent wiring is limited to `mod formal_module_conversion;` and the controlled re-export.
4. Route surface, graph generation, artifact projection, parser, `src/compile_api.rs`, and `src/lib.rs` did not move.
5. `backend.graph_compile.quantscript_graph stop_split: true`, `backend.graph_compile`, or Rust-wide restructuring completion must not be claimed.

---

## Acceptance Criteria

1. `486-backend.graph_compile.quantscript_graph.formal_module_conversion抽离记录.md` enters milestone index, module tree, full feature tree, and governance gate.
2. `formal_module_conversion actual_extraction_done` is recorded.
3. The child file is covered by the module tree and full tree.
4. Parent controlled re-export keeps the existing caller surface.
5. Governance gates, full tree, UTF-8, Rust fmt/check, QS narrow tests, and `git diff --check` pass.
