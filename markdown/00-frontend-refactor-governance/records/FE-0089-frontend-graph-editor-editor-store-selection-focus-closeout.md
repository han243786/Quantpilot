# FE-0089 Frontend Graph Editor Editor Store Selection Focus Closeout

Status: closed.

## Child Node

`frontend.graph_editor.editor_store_actions.selection_focus`

## Boundary

This leaf owns editor selection and compile-diagnostic focus actions inside the graph store. `graphStoreEditorActions.js` remains the parent facade that exposes the existing store action API and composes this child with the rest of editor, compile, and persistence actions.

## Changed Files

- `frontend/src/store/graphStoreEditorActions.js`
- `frontend/src/store/graphStoreEditorSelectionActions.js`
- `frontend/src/store/graphStoreEditorSelectionActions.test.js`

## Public Methods

- `setSelectedNode`
- `setSelectedEdge`
- `focusCompileDiagnostic`

## Preserved Behavior

- Node selection still clears edge selection and compile diagnostic focus.
- Edge selection still clears node selection and compile diagnostic focus.
- Compile diagnostics still focus node and edge targets as editor selections.
- Strategy IR and graph-level diagnostics still stay in `selectedCompileDiagnosticTarget`, with Strategy IR draft recovery delegated through the same helper path as before.
- No child-to-child store action calls were introduced; the parent facade composes the child action factory.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; `editor_store_actions` mixes selection focus, draft/source application, template loading, node mutation, edge mutation, deletion, and parent store composition.
- `leaf_split_positive_trigger`: `semantic_boundary`, `testability_gain`, `blast_radius_reduction`, and `public_method_cluster`.
- `leaf_split_stop_condition`: reached for `selection_focus`; the child is a small cohesive public action cluster with direct white-box tests.
- `leaf_split_decision_result`: continue splitting `frontend.graph_editor.editor_store_actions` through draft/source actions next.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/store/graphStoreEditorSelectionActions.test.js src/store/graphStore.editorActions.test.js src/store/graphStore.strategyIrDraft.test.js src/components/PropertyPanel.strategyIr.test.jsx`: passed, 4 files / 11 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.editor_store_actions.draft_source_actions`
