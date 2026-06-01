# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion single leaf closeout
> Version type: MINOR governance / closeout
> Execution tier: lightweight
> Batch: BE-001FT-04
> Baseline: `486-backend.graph_compile.quantscript_graph.formal_module_conversion抽离记录.md`
> Target leaf: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Decision: equivalent, keep splitting
> Module tree coordinate: `root.backend.graph_compile.quantscript_graph.formal_module_conversion`
> Code action: no code movement
> Next step: BE-001FU-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` parent residual judgment

---

## Matrix Impact

| Matrix | Changed node | Change type |
| --- | --- | --- |
| Process matrix | BE-001FT-04 `formal_module_conversion` single leaf closeout | closeout / split decision |
| Standard matrix | equivalence confirmed / stop_split false / no sibling horizontal link / release transition guard | lightweight tier |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion` | child owner confirmed |
| Module tree | `backend.graph_compile.quantscript_graph.formal_module_conversion` | keep recursive split queue open |

---

## Closeout Evidence

BE-001FT-03 completed the intended actual extraction:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
formal_module_conversion actual_extraction_done
```

The parent keeps the caller-facing interface through:

```rust
mod formal_module_conversion;
pub(crate) use formal_module_conversion::convert_graph_json_to_script_module;
```

No compile / graph / runtime sibling directly imports the child.

---

## Equivalence Judgment

The extracted child remains equivalent because:

1. `convert_graph_json_to_script_module` still accepts graph `Value` and returns `ScriptModule`.
2. `graph.nodes` and `graph.edges` validation still occurs before QS source generation.
3. `data`, `risk`, `execution`, and `intent` branch semantics remain unchanged.
4. unknown non-ignored node logging still uses `safe_eprintln!`.
5. unsupported intent module handling still uses `anyhow::bail!`.
6. terminal parsing still uses `parse_quant_script_module(&qs_source)`.
7. `src/compile_api.rs` and `src/lib.rs` caller surfaces did not change.

---

## Split Decision

This leaf should not close permanently yet:

```text
formal_module_conversion stop_split: false
```

Reason:

1. The child is still a multi-responsibility conversion unit.
2. It contains independent graph shape validation, data source lowering, profile lowering, intent lowering, unsupported module failure, and terminal parse responsibilities.
3. Intent lowering contains several independent built-in strategy branches.
4. Further split can reduce future regression blast radius while preserving parent-only communication.

The next recursive step must be a parent residual judgment before choosing any sub-leaf.

---

## Not In Scope

BE-001FT-04 does not:

1. move Rust code.
2. split `data`, `risk`, `execution`, or `intent` lowering yet.
3. change route surface.
4. change graph-to-QS generation.
5. change artifact target projection.
6. change strategy graph parser.
7. change `src/compile_api.rs` or `src/lib.rs`.
8. start release transition guard.
9. add sibling horizontal links.

---

## Next Boundary

The next step can only be:

```text
BE-001FU-01
backend.graph_compile.quantscript_graph.formal_module_conversion
root.backend.graph_compile.quantscript_graph.formal_module_conversion
parent residual judgment
```

BE-001FU-01 may only inspect residual responsibilities and select the next sub-leaf candidate. It must not directly move code or declare `backend.graph_compile.quantscript_graph` closed.

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
```

---

## Hallucination Guard

AI claiming BE-001FT-04 is done must state:

1. This batch is `no code movement`.
2. BE-001FT-03 actual extraction remains the code evidence.
3. `formal_module_conversion stop_split: false`.
4. Next step is BE-001FU-01 parent residual judgment.
5. It must not claim `backend.graph_compile.quantscript_graph`, `backend.graph_compile`, `backend`, or Rust-wide restructuring is complete.

---

## Acceptance Criteria

1. `487-backend.graph_compile.quantscript_graph.formal_module_conversion单叶closeout.md` enters milestone index, module tree, full feature tree, and governance gate.
2. `formal_module_conversion closeout_done` is recorded.
3. `formal_module_conversion stop_split: false` is recorded.
4. Next step is fixed as BE-001FU-01 parent residual judgment.
5. Governance gates, full tree, UTF-8, Rust fmt/check, and `git diff --check` pass.
