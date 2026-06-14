# FE-0093 Frontend Graph Editor Editor Store Edge Removal Actions Closeout

Status: closed.

## Child Node

`frontend.graph_editor.editor_store_actions.edge_removal_actions`

## Boundary

This leaf owns graph store edge creation and selected-node/selected-edge removal. `graphStoreEditorActions.js` remains the parent facade that exposes the existing public action names and composes this child with the rest of editor store actions.

## Changed Files

- `frontend/src/store/graphStoreEditorActions.js`
- `frontend/src/store/graphStoreEditorEdgeActions.js`
- `frontend/src/store/graphStoreEditorEdgeActions.test.js`

## Public Methods

- `addEdge`
- `removeSelected`

## Preserved Behavior

- Edge creation still builds the same edge shape, validates and persists graph state, records touched node ids, clears compile result, and refreshes graph source draft.
- Selected node removal still removes the node and incident edges, filters recent node ids, clears selection, clears compile result, and preserves runtime state shape.
- Selected edge removal still removes only the selected edge and keeps graph nodes intact.
- No child-to-child store action calls were introduced; the parent facade composes the child action factory.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; edge/removal has independent graph topology mutation and selection cleanup behavior from node mutation and draft/source actions.
- `leaf_split_positive_trigger`: `semantic_boundary`, `independent_failure_mode`, `testability_gain`, and `blast_radius_reduction`.
- `leaf_split_stop_condition`: reached for `edge_removal_actions`; the child owns a cohesive graph topology edit cluster with direct white-box tests and existing canvas/store regressions.
- `leaf_split_decision_result`: all planned `frontend.graph_editor.editor_store_actions` subchildren are closed; perform parent closeout next.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/store/graphStoreEditorEdgeActions.test.js src/store/graphStore.editorActions.test.js src/store/graphStore.recentNodes.test.js src/components/StrategyCanvas.interaction.test.jsx`: passed, 4 files / 15 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Step

Parent closeout for `frontend.graph_editor.editor_store_actions`.
