# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging baseline plan

> Batch: BE-001GY-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `baseline_plan`
> Movement: no code movement.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GY-02 `unsupported_node_logging` `extract_closeout`

---

## Summary

This baseline freezes the remaining unknown-node diagnostic path in
`formal_module_conversion.rs`. The next movement may extract only the known
node no-op classifier and unknown-node log message construction.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GY-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / nonblocking diagnostic ownership | unsupported node logging freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging` | planned child white-box node |
| Module tree | `formal_module_conversion -> unsupported_node_logging` | planned child edge |

---

## Equivalence Baseline

Frozen owner:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
```

Frozen no-op node types:

```text
data
intent
agent
runtime
runtime_control
```

Frozen unknown-node behavior:

```text
safe_eprintln!("[graph->QS] 未知节点类型 '{}', 跳过 QS 生成", node_type)
```

Frozen ordering:

```text
profile_lowering handles risk/execution first.
unsupported_node_logging runs only for non-profile nodes.
unknown node logging remains nonblocking.
```

Frozen non-goals:

```text
No graph validation movement.
No data_source_lowering movement.
No profile_lowering movement.
No intent_lowering movement.
No terminal_parse movement.
No public API, route, schema, persistence, lock owner, or state-machine change.
No sibling module call.
No release-transition optimization.
```

---

## Extraction Plan

Planned child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/unsupported_node_logging.rs
```

Planned parent additions:

```rust
mod unsupported_node_logging;

unsupported_node_logging::log_if_unsupported_node(node);
```

Planned child surface:

```rust
pub(super) fn log_if_unsupported_node(node: &Value)
```

The parent keeps ownership of:

```text
overall node iteration order
profile_lowering handled check
intent lowering call
terminal parse call
```

The child owns only:

```text
node type lookup for logging
known non-profile no-op list
unknown-node message construction
safe_eprintln! diagnostic call
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid:

1. The child is the final local diagnostic residual of this parent.
2. No public interface or conversion output changes.
3. Parent-child communication is one direct helper call.
4. Local tests can freeze no-op classification and unknown-node message text.

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/unsupported_node_logging.rs` (planned)

**Markers**:
- `unsupported_node_logging baseline_plan`

**Next step**:
BE-001GY-02 unsupported_node_logging extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot unsupported_node_logging`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view`
