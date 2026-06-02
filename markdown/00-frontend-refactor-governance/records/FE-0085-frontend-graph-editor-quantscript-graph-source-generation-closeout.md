# FE-0085 Frontend Graph Editor QuantScript Graph Source Generation Closeout

Status: closed.

## Child Node

`frontend.graph_editor.quantscript_bridge.graph_source_generation`

## Boundary

This leaf owns local `strategy_graph` source generation for nodes and full graphs. `quantscript.js` remains the parent facade and re-exports graph source and formal generation entry points for compatibility.

## Owned Files

- `frontend/src/graph/quantscript.js`
- `frontend/src/graph/quantscript.test.js`
- `frontend/src/graph/quantscriptGraphSource.js`
- `frontend/src/graph/quantscriptGraphSource.test.js`
- `frontend/src/graph/quantscriptFormal.js`
- `frontend/src/graph/compileGraph.diagnostics.test.js`
- `frontend/src/store/graphStore.strategyIrCompile.test.js`
- `frontend/src/templates/strategyTemplates.test.js`
- `frontend/src/store/graphStore.editorActions.test.js`

## Public Methods

- `generateNodeQuantScript`
- `generateGraphQuantScript`
- `generateFormalQuantScript`
- `attachQuantScriptArtifacts`

## Preserved Behavior

- `compileGraph` can still import `generateGraphQuantScript` from `quantscript.js`.
- Existing consumers can still import `generateFormalQuantScript`, `generateNodeQuantScript`, and `generateGraphQuantScript` from the parent facade.
- `attachQuantScriptArtifacts` still creates graph source, node source, formal source, label targets, and runtime targets from the same facade path.
- Graph source output keeps metadata, runtime mode, node config serialization, incoming input references, and graph connections equivalent.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; graph source serialization is independent from formal lowering and parser import recovery.
- `leaf_split_positive_trigger`: `testability_gain`, `semantic_boundary`, and `blast_radius_reduction`.
- `leaf_split_stop_condition`: reached for `graph_source_generation`; no deeper split now.
- `leaf_split_decision_result`: continue `frontend.graph_editor.quantscript_bridge` through artifact target mapping and graph source parser children.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/graph/quantscriptGraphSource.test.js src/graph/quantscriptFormal.test.js src/graph/quantscript.test.js src/graph/compileGraph.diagnostics.test.js src/store/graphStore.strategyIrCompile.test.js src/templates/strategyTemplates.test.js src/store/graphStore.editorActions.test.js`: passed, 7 files / 16 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.quantscript_bridge.artifact_targets`
