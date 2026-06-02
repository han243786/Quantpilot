# FE-0086 Frontend Graph Editor QuantScript Artifact Targets Closeout

Status: closed.

## Child Node

`frontend.graph_editor.quantscript_bridge.artifact_targets`

## Boundary

This leaf owns local QuantScript artifact label targets and approximate runtime targets. `quantscript.js` remains the parent facade that attaches graph source, formal source, node source, label targets, and runtime targets.

## Owned Files

- `frontend/src/graph/quantscript.js`
- `frontend/src/graph/quantscript.test.js`
- `frontend/src/graph/quantscriptArtifactTargets.js`
- `frontend/src/graph/quantscriptArtifactTargets.test.js`
- `frontend/src/graph/quantscriptFormal.js`
- `frontend/src/graph/quantscriptGraphSource.js`
- `frontend/src/store/graphStore.strategyIrCompile.test.js`
- `frontend/src/templates/strategyTemplates.test.js`
- `frontend/src/store/graphStore.editorActions.test.js`

## Public Methods

- `buildQuantScriptLabelTargets`
- `buildQuantScriptRuntimeTargets`
- `attachQuantScriptArtifacts`

## Preserved Behavior

- Artifact attachment still stores `label_targets` and `runtime_targets` under `metadata.artifacts.quantscript`.
- Label targets still include node ids, node names, config fields, and formal data/intent bindings.
- Runtime targets still map formal data/intent ids and agent/risk script aliases to graph node ids.
- Runtime and execution endpoint ids remain derived from graph runtime/execution nodes.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; artifact target projection has a separate failure mode from source generation and parsing.
- `leaf_split_positive_trigger`: `testability_gain`, `semantic_boundary`, and `independent_failure_mode`.
- `leaf_split_stop_condition`: reached for `artifact_targets`; no deeper split now.
- `leaf_split_decision_result`: continue `frontend.graph_editor.quantscript_bridge` through graph source parser.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/graph/quantscriptArtifactTargets.test.js src/graph/quantscriptGraphSource.test.js src/graph/quantscriptFormal.test.js src/graph/quantscript.test.js src/store/graphStore.strategyIrCompile.test.js src/templates/strategyTemplates.test.js src/store/graphStore.editorActions.test.js`: passed, 7 files / 17 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.quantscript_bridge.graph_source_parser`
