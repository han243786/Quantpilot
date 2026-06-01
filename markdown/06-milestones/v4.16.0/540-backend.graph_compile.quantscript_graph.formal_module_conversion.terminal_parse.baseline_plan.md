# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse baseline plan

> Batch: BE-001GW-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `baseline_plan`
> Movement: no code movement.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GW-02 `terminal_parse` `extract_closeout`

---

## Summary

This baseline freezes the terminal parse responsibility currently embedded in
`formal_module_conversion.rs`. The next movement may extract only the closing
brace append, newline join, and `parse_quant_script_module` invocation.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GW-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / terminal phase ownership | terminal parse freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse` | planned child white-box node |
| Module tree | `formal_module_conversion -> terminal_parse` | planned child edge |

---

## Equivalence Baseline

Frozen owner:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
```

Frozen terminal sequence:

```rust
qs_lines.push("}".to_string());
let qs_source = qs_lines.join("\n");
parse_quant_script_module(&qs_source)
```

Frozen output:

```text
anyhow::Result<ScriptModule>
```

Frozen non-goals:

```text
No generated line content changes.
No data_source_lowering movement.
No profile_lowering movement.
No input_shape_validation movement.
No intent_lowering movement.
No unsupported node logging movement.
No public API, route, schema, persistence, lock owner, or state-machine change.
No sibling module call.
No release-transition optimization.
```

---

## Extraction Plan

Planned child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/terminal_parse.rs
```

Planned parent additions:

```rust
mod terminal_parse;

terminal_parse::parse_generated_qs_lines(qs_lines)
```

Planned child surface:

```rust
pub(super) fn parse_generated_qs_lines(qs_lines: Vec<String>) -> anyhow::Result<ScriptModule>
```

The parent keeps ownership of:

```text
overall QS source assembly order before terminal parse
input_shape_validation call
data_source_lowering call
profile_lowering call
unsupported node logging until selected separately
intent_lowering call
```

The child owns only:

```text
closing brace append
QS line join delimiter
parse_quant_script_module call
terminal ScriptModule return
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid:

1. The child is a narrow terminal phase with no public surface change.
2. Parent-child communication is one direct call consuming `qs_lines`.
3. Existing compile endpoint tests exercise the terminal parse path.
4. No route, schema, persistence, lock owner, public API, or state-machine change occurs.

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/terminal_parse.rs` (planned)

**Markers**:
- `terminal_parse baseline_plan`

**Next step**:
BE-001GW-02 terminal_parse extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view`
