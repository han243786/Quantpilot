# FE-0084 Frontend Graph Editor QuantScript Formal Generation Closeout

Status: closed.

## Child Node

`frontend.graph_editor.quantscript_bridge.formal_generation`

## Boundary

This leaf owns formal QuantScript source generation and stable formal runtime binding identifiers. `quantscript.js` remains the parent facade for graph artifacts, graph source generation, and graph source parsing.

## Owned Files

- `frontend/src/graph/quantscript.js`
- `frontend/src/graph/quantscriptFormal.js`
- `frontend/src/graph/quantscriptFormal.test.js`
- `frontend/src/graph/quantscript.test.js`
- `frontend/src/graph/compileGraph.diagnostics.test.js`
- `frontend/src/store/graphStore.strategyIrCompile.test.js`
- `frontend/src/templates/strategyTemplates.test.js`
- `frontend/src/store/graphStore.editorActions.test.js`

## Public Methods

- `generateFormalQuantScript`
- `canGenerateFormalQuantScript`
- `formalDataRuntimeId`
- `formalDataBindingName`
- `formalIntentRuntimeId`
- `formalIntentBindingBase`
- `formalIntentSignalBindingName`
- `formalDataNodes`
- `attachQuantScriptArtifacts`

## Preserved Behavior

- `attachQuantScriptArtifacts` still stores `formal_source`, `label_targets`, and `runtime_targets` under `metadata.artifacts.quantscript`.
- `compileGraph` and Strategy IR compile integration continue using the same QuantScript artifacts through the existing `quantscript.js` facade.
- Unsupported formal graph shapes still return an empty formal source.
- Formal runtime IDs keep the same local identifier normalization.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; `quantscript.js` mixed formal lowering, graph source generation, artifact target mapping, and source parsing.
- `leaf_split_positive_trigger`: `testability_gain`, `independent_failure_mode`, `semantic_boundary`, and `blast_radius_reduction`.
- `leaf_split_stop_condition`: not reached for `frontend.graph_editor.quantscript_bridge`; continue with graph source generation, artifact target mapping, and graph source parser children.
- `leaf_split_decision_result`: `frontend.graph_editor.quantscript_bridge` is now treated as a small parent while the active graph editor parent remains the outer owner.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/graph/quantscriptFormal.test.js src/graph/quantscript.test.js src/graph/compileGraph.diagnostics.test.js src/store/graphStore.strategyIrCompile.test.js src/templates/strategyTemplates.test.js src/store/graphStore.editorActions.test.js`: passed, 6 files / 14 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.quantscript_bridge.graph_source_generation`
