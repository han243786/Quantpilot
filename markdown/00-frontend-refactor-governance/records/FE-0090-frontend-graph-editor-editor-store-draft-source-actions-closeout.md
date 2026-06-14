# FE-0090 Frontend Graph Editor Editor Store Draft Source Actions Closeout

Status: closed.

## Child Node

`frontend.graph_editor.editor_store_actions.draft_source_actions`

## Boundary

This leaf owns graph-source, formal-source, and Strategy IR draft updates plus local QuantScript source application inside the graph store. `graphStoreEditorActions.js` remains the parent facade that exposes the existing public action names and composes this child with other editor store children.

## Changed Files

- `frontend/src/store/graphStoreEditorActions.js`
- `frontend/src/store/graphStoreEditorDraftActions.js`
- `frontend/src/store/graphStoreEditorDraftActions.test.js`

## Public Methods

- `updateQuantScriptDraft`
- `updateFormalQuantScriptDraft`
- `updateStrategyIrDraft`
- `resetQuantScriptDraft`
- `resetFormalQuantScriptDraft`
- `resetStrategyIrDraft`
- `applyQuantScriptSource`

## Preserved Behavior

- Draft update methods still write only their matching draft field.
- Reset methods still recover graph source from current artifacts, clear formal overrides, clear compile diagnostic focus, and clear stale compile results.
- `applyQuantScriptSource` still parses QuantScript through the existing graph bridge, attaches validation, persists the graph, clears editor focus, clears compile result, refreshes the graph source draft, and preserves runtime state shape.
- No child-to-child store action calls were introduced; the parent facade composes the child action factory.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; draft/source application has an independent parse/persist failure mode from selection, template loading, node mutation, and edge mutation.
- `leaf_split_positive_trigger`: `semantic_boundary`, `independent_failure_mode`, `testability_gain`, and `blast_radius_reduction`.
- `leaf_split_stop_condition`: reached for `draft_source_actions`; the child is cohesive and has direct white-box coverage for draft resets and QuantScript source application.
- `leaf_split_decision_result`: continue splitting `frontend.graph_editor.editor_store_actions` through template loading actions next.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/store/graphStoreEditorDraftActions.test.js src/store/graphStore.editorActions.test.js src/store/graphStore.strategyIrDraft.test.js src/components/PropertyPanel.strategyIr.test.jsx src/components/propertyPanelCompileSourceCards.test.jsx`: passed, 5 files / 11 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.editor_store_actions.template_loading_actions`
