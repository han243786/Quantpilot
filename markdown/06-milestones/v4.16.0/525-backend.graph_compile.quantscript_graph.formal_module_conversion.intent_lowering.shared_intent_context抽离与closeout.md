# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context extract closeout

> Version type: MINOR architecture / governance
> Execution tier: lightweight
> Batch: BE-001GL-02
> Baseline plan: `524-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context等价基线与抽离方案.md`
> Target child: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context`
> Stage: `extract_closeout`
> Movement: Rust context extraction + single leaf closeout
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GM-01 `intent_lowering` parent residual judgment

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GL-02 extract + closeout | lightweight two-step stage 2 |
| Norm matrix | parent-child communication / leaf split gate / equivalence proof | hard rule execution |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context` | child white-box node closed |
| Module tree | `intent_lowering -> shared_intent_context` | new one-way child edge |

---

## Actual Extraction

Created child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/shared_intent_context.rs
```

Parent now keeps:

```rust
mod shared_intent_context;
let ctx = shared_intent_context::resolve_intent_lowering_context(node, edges);
match ctx.module_key { ... }
```

Child owns:

```rust
pub(super) struct IntentLoweringContext<'a> {
    pub(super) module_key: &'a str,
    pub(super) cfg: &'a Value,
    pub(super) instrument: &'a str,
    pub(super) node_id: &'a str,
    pub(super) source_var: String,
}

pub(super) fn resolve_intent_lowering_context<'a>(
    node: &'a Value,
    edges: &'a [Value],
) -> IntentLoweringContext<'a>
```

Frozen semantics preserved:

```text
module_key default ""
cfg default Value::Null
instrument default BTCUSDT
node_id default ""
source_id default data
source_var = source_id.replace(['-', '.'], "_")
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | `shared_intent_context` now has a named child file, context struct, and resolver helper. |
| parent_child_communication_kept | pass | Parent calls the context helper and then mediates every branch call; no child calls siblings. |
| equivalence_baseline_freezable | pass | BE-001GL-01 froze every default and transform. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No route, API, schema, persistence, or lock owner changed. |
| state_machine_phase | no | No state transition exists in this helper. |
| strategy_branch | no | This helper owns shared context, not a strategy branch. |
| independent_failure_mode | no | Unsupported intent failure remains outside this child. |
| reuse_pressure | yes | All branch children consume this resolved context through parent mediation. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | yes | Further split would produce instrument fallback, source lookup, and source normalization micro-leaves. |
| communication_cost_rises | yes | Further split would add tiny helper hops before every branch dispatch. |
| local_proof_missing | no | Current helper can be proven by compile/check and formal built-in intent tests. |
| line_count_only | no | Extraction was based on shared context ownership. |

leaf_split_decision_result

```text
stop_split_true
shared_intent_context actual_extraction_done
shared_intent_context closeout_done
shared_intent_context stop_split: true
```

next_recursive_step

```text
BE-001GM-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
parent_residual_judgment
```

---

## Parent Child Rule

Allowed:

```text
intent_lowering -> shared_intent_context
intent_lowering -> built-in branch children
```

Still forbidden:

```text
shared_intent_context -> branch children
branch children -> shared_intent_context
runtime sibling -> shared_intent_context
frontend -> shared_intent_context
sibling horizontal link
release transition
```

---

## Gates

This batch must pass:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view
```
