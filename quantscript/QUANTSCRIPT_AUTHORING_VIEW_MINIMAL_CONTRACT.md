# QuantScript Authoring View Minimal Contract

This document defines the first minimal contract for a backend authoring artifact
that can support readable frontend strategy presentation and safe lightweight
editing workflows for QuantScript `V1`.

The artifact name in this contract is:

- `quantscript_authoring_view`

## 1. Purpose

The purpose of `quantscript_authoring_view` is to give frontend and AI consumers
a stable, source-first, structured view of a QuantScript strategy without
turning parser shape, comment text, or lowered runtime config into a new truth
source.

It exists to support:

- readable frontend strategy display
- explicit module ordering
- visible data-flow explanation
- stable section highlighting
- future lightweight patch-oriented strategy edits

It does **not** exist to replace:

- formal QuantScript source
- semantic analysis
- runtime config
- core IR
- Strategy IR

## 1.1 Current landed subset

The current landed subset is:

- generated on the successful formal QuantScript compile path
- attached at `artifacts.strategy.metadata.quantscript_authoring_view`
- line-based
- snippet-based
- section-header-aware
- semantically classified with resolve/analysis-informed heuristics
- augmented with minimal edge generation
- augmented with a derived read-only `pool_pipeline` block when lowering can extract
  an `InstrumentPoolSpec`
- consumed by the frontend source workspace as:
  - source-order module view
  - pipeline-order flow view
  - pool-pipeline view

The current landed subset does **not** yet include:

- parser-native byte spans
- selection spans
- rewrite-safe anchors

## 2. Source-of-truth rule

`quantscript_authoring_view` is a derived artifact.

It is **not** a source of truth.

The truth-source rule is:

1. formal QuantScript source is the only editable source
2. resolve / analysis define executable semantic meaning
3. lowering may supply derived overlay facts
4. `quantscript_authoring_view` is a source-first presentation/indexing layer

Frontend and AI systems must never treat this artifact as the editable strategy
body.
Any edit must eventually resolve to a patch against formal source.

## 3. Current implementation constraint

Current QuantScript `V1` code does **not** yet expose stable AST/HIR source
ranges with line/column spans.

That means the first contract must not assume:

- full parser-native `source_span`
- stable `selection_span`
- source-map-grade rewrite anchors

So the first retained contract is intentionally line-based and snippet-based.

## 4. Generation layer

The artifact should be generated in three stages:

1. raw source scan
2. resolve / analysis classification
3. lowering overlay

Recommended ownership:

- raw source scan establishes section boundaries from source text and section
  headers
- resolve / analysis establish semantic classification and symbol usage
- lowering supplements derived agent / profile attachment facts where current
  `V1` semantics are only partially explicit in source

The artifact should be attached to the compile artifact bundle, not stored as a
separate independent truth source.

## 5. Minimal artifact shape

The first minimal contract should be:

```json
{
  "kind": "quantscript_authoring_view",
  "source_hash": "sha256:...",
  "source_order": ["risk", "execution", "data", "intent", "agent"],
  "pipeline_order": ["data", "intent", "agent", "risk", "execution"],
  "sections": [
    {
      "id": "section:data:1",
      "declared_kind": "data",
      "effective_kind": "data",
      "origin": "authored",
      "status": "ok",
      "start_line": 8,
      "end_line": 14,
      "snippet": "...",
      "symbols_defined": ["series_closes"],
      "symbols_used": []
    }
  ],
  "edges": [
    {
      "from": "section:data:1",
      "to": "section:intent:1",
      "relation": "dataflow",
      "reason": "intent_reads_data"
    }
  ],
  "pool_pipeline": {
    "order": ["source", "eligibility", "features", "selection", "weighting", "rebalance"],
    "stages": [
      {
        "kind": "source",
        "status": "present",
        "summary": "universe(exchange=binance, market=spot, quote=USDT)",
        "details": ["exchange=binance", "market=spot", "quote=USDT"],
        "related_section_ids": ["section:data:1"]
      }
    ]
  }
}
```

## 6. Top-level fields

### `kind`

Must always be:

- `quantscript_authoring_view`

### `source_hash`

Required.
This binds the artifact to the exact formal source text that produced it.

### `source_order`

Required.
This represents author-facing source ordering.

For current `V1`, the retained source order should be:

- `risk`
- `execution`
- `data`
- `intent`
- `agent`

This order reflects current runtime/lowering constraints, especially the
top-level placement requirement for `risk.profile(...)` and
`execution.profile(...)`.

### `pipeline_order`

Required.
This represents conceptual strategy flow for frontend display.

For current `V1`, the retained conceptual order should be:

- `data`
- `intent`
- `agent`
- `risk`
- `execution`

The contract intentionally keeps both orders.

## 7. Section contract

Each section must contain:

- `id`
- `declared_kind`
- `effective_kind`
- `origin`
- `status`
- `start_line`
- `end_line`
- `snippet`
- `symbols_defined`
- `symbols_used`

### `declared_kind`

This is the author-declared section kind inferred from section headers such as:

- `# risk`
- `# execution`
- `# data`
- `# intent`
- `# agent`

If no explicit header exists, `declared_kind` may be inferred from surrounding
structure or set equal to the first stable semantic classification.

### `effective_kind`

This is the semantic section kind inferred from resolve / analysis, with
lowering-derived help where needed.

Retained `V1` kinds:

- `risk`
- `execution`
- `data`
- `intent`
- `agent`
- `mixed`
- `unknown`

### `origin`

Allowed values:

- `authored`
- `derived`
- `hybrid`

Use:

- `authored` when the source section is explicit and semantically aligned
- `derived` when the section exists as a semantic/runtime consequence rather
  than a direct authored block
- `hybrid` when section meaning depends on both authored structure and derived
  lowering context

### `status`

Allowed minimal values:

- `ok`
- `mismatch`
- `partial`

Use:

- `ok` when authored and effective meaning align
- `mismatch` when declared and effective kinds differ
- `partial` when diagnostics or incomplete code allow only best-effort section
  extraction

### `start_line` / `end_line`

These are required in `V1`.

They are line-based because the current parser/AST does not yet expose stable
source spans.

They should cover the full section range, including its author-facing comment
header when one exists.

### `snippet`

Required.
This must be the original source slice for the section, not normalized or
lowered text.

`snippet` is for display, linking, and future patch generation support.
It must not become a second editable truth source.

### `symbols_defined` / `symbols_used`

Minimal `V1` form:

- arrays of symbol names

Preferred semantic source:

- resolve / bindings / semantic analysis

Fallback:

- source-level inference when semantic information is unavailable

These arrays should describe author-facing symbols, not runtime config ids.

## 8. Edge contract

Each edge must contain:

- `from`
- `to`
- `relation`
- `reason`

### `relation`

Retained minimal relation types:

- `dataflow`
- `decision_flow`
- `policy_attachment`
- `execution_attachment`

Meaning:

- `dataflow`: data section feeds intent logic
- `decision_flow`: intent logic drives agent behavior
- `policy_attachment`: risk attaches to agent behavior
- `execution_attachment`: execution attaches to agent behavior

### `reason`

Minimal `V1` form:

- stable string code

Recommended retained reason codes:

- `intent_reads_data`
- `agent_uses_intent`
- `risk_governs_agent`
- `execution_applies_to_agent`

Do not use free-form prose as the only reason value in the first contract.

## 8.1 Pool pipeline contract

When lowering can derive an `InstrumentPoolSpec`, the authoring view may include
an optional `pool_pipeline` block.

Minimal retained fields:

- `order`
- `stages[*].kind`
- `stages[*].status`
- `stages[*].summary`
- `stages[*].details`
- `stages[*].related_section_ids`

Retained stage order:

- `source`
- `eligibility`
- `features`
- `selection`
- `weighting`
- `rebalance`

This block is read-only and derived.
It must not be treated as a second editable pool DSL.

## 9. Section extraction strategy

Phase 1 section extraction should follow this order:

1. scan raw source for explicit section headers
2. map source lines into coarse section ranges
3. classify each section semantically using resolve / analysis
4. overlay lowering-derived facts where needed

Conflict rule:

- authored section headers are hints for boundary grouping
- semantic classification determines `effective_kind`
- mismatch must be surfaced through `status = mismatch`

Do not silently rewrite authored structure into a prettier semantic layout.

## 10. Agent handling rule

In current QuantScript `V1`, the `agent` layer is often only partially explicit
in source.

So the contract must allow:

- `agent` sections with `origin = authored`
- `agent` sections with `origin = hybrid`
- `agent` sections with `origin = derived`

The artifact should not pretend that all agent behavior is explicitly authored
as a standalone source block.

## 11. Best-effort generation rule

`quantscript_authoring_view` should be produced on a best-effort basis whenever
possible, even when compile/lowering fails.

That target means:

- diagnostics may coexist with a partial authoring view
- partial section extraction is better than dropping the artifact entirely

This target is necessary for:

- frontend readability during editing
- AI-assisted repair
- quick user micro-adjustment

Current landed status:

- successful formal compile responses emit the artifact
- failed formal compile responses now emit
  `partial_artifacts.quantscript_authoring_view` on a best-effort basis
- current failed-compile best-effort output is still line-based/snippet-based and
  only includes `pool_pipeline` when internal extraction succeeds
- the frontend source workspace now consumes the same artifact from both
  successful compile metadata and failed-compile `partial_artifacts`
- when the frontend is consuming failed-compile fallback output, source
  workspace should surface an explicit partial-artifact status notice rather
  than silently pretending the compile succeeded
- current source-workspace authoring-view display/highlighting is ahead of the
  original `strategy_graph` editor: a separate editable/apply-able formal source
  lane is now required and should own authoring-view section highlighting

## 12. Frontend consumption rule

Frontend should consume the artifact in two modes:

### Source mode

Use:

- `source_order`
- `sections[*].snippet`
- `start_line`
- `end_line`

This supports:

- code reading
- section highlighting
- jump-to-source

### Pipeline mode

Use:

- `pipeline_order`
- `edges`
- `effective_kind`
- `origin`
- `status`

This supports:

- conceptual flow display
- module cards
- data-flow explanation
- stable user-facing strategy readability

### Pool pipeline mode

Use:

- `pool_pipeline.order`
- `pool_pipeline.stages[*].summary`
- `pool_pipeline.stages[*].details`
- `pool_pipeline.stages[*].related_section_ids`

This supports:

- visible pool construction stages
- selection / weighting / rebalance readability
- frontend linking from pool stages back to authored source sections

Frontend should not reconstruct section meaning from raw strings if
`quantscript_authoring_view` is present.

## 13. Explicit non-goals for Phase 1

Do not include these in the first contract:

- full parser-native byte spans
- selection spans
- per-expression source maps
- editable runtime config copies
- full Strategy IR copies
- full Core IR copies
- per-trade or timeline compare structures
- free-form report narrative duplication
- a second editable strategy representation

## 14. Phase 1 acceptance rule

The first contract is acceptable only if:

- formal source remains the only editable truth source
- frontend can render source order and conceptual pipeline order separately
- sections are line-based and stable enough for current `V1`
- semantic classification comes from resolve / analysis, not just comments
- agent derivation can be represented honestly
- the artifact does not duplicate runtime/core-ir truth unnecessarily

## 15. Current implementation status

This contract is now landed at the first retained subset:

1. `quantscript_authoring_view` is generated in the formal QuantScript compile path
2. the first version is line-based and snippet-based
3. it is exposed through the compile artifact bundle at `artifacts.strategy.metadata.quantscript_authoring_view`
4. the artifact now includes a read-only `pool_pipeline` block when internal
   lowering can derive `InstrumentPoolSpec`
5. the frontend source workspace now renders source-order, conceptual flow, and
   pool-pipeline views from the same artifact
6. failed formal compile responses now expose
   `partial_artifacts.quantscript_authoring_view` when best-effort extraction succeeds
7. source workspace now includes a dedicated editable/apply-able formal
   QuantScript lane; authoring-view section/stage/edge highlighting should bind
   to that editor rather than the `strategy_graph` draft editor
8. workspace compile controls now explicitly show whether compile is using the
   graph-generated formal source or an applied formal override

The next step is not more syntax work.
The next step is frontend consumption:

1. broader frontend interaction and quick-edit affordances
2. richer pool-feature population once pool semantics move beyond helper-only extraction
3. more stable failed-compile best-effort coverage beyond current successful parse/resolve cases
