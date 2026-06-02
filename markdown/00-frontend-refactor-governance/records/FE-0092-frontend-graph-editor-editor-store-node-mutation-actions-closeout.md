# FE-0092 Frontend Graph Editor Editor Store Node Mutation Actions Closeout

Status: closed.

## Child Node

`frontend.graph_editor.editor_store_actions.node_mutation_actions`

## Boundary

This leaf owns graph store node creation, node position/config/name/collapse mutation, and editor viewport mutation. `graphStoreEditorActions.js` remains the parent facade that exposes the existing public action names and composes this child with the rest of editor store actions.

## Changed Files

- `frontend/src/store/graphStoreEditorActions.js`
- `frontend/src/store/graphStoreEditorNodeActions.js`
- `frontend/src/store/graphStoreEditorNodeActions.test.js`

## Public Methods

- `createNode`
- `updateNodePosition`
- `updateEditorViewport`
- `updateNodeConfig`
- `updateNodeName`
- `toggleNodeCollapse`

## Preserved Behavior

- Node creation still ignores unknown or unsupported module keys, creates nodes through the module factory, validates and persists the graph, selects the new node, and refreshes graph source draft.
- Node position, config, name, and collapse mutations still validate graph state, refresh recent-node tracking, clear compile result, persist when required, and refresh graph source draft.
- Editor viewport updates still mutate only `metadata.editor.viewport` and persist only when requested.
- No child-to-child store action calls were introduced; the parent facade composes the child action factory.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; node mutation has independent graph-state write paths from draft editing, selection focus, template loading, edge mutation, and removal.
- `leaf_split_positive_trigger`: `semantic_boundary`, `testability_gain`, `public_method_cluster`, and `blast_radius_reduction`.
- `leaf_split_stop_condition`: reached for `node_mutation_actions`; the child owns a cohesive graph-node edit cluster with direct white-box tests and existing canvas/store regressions.
- `leaf_split_decision_result`: continue splitting `frontend.graph_editor.editor_store_actions` through edge/removal actions next.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/store/graphStoreEditorNodeActions.test.js src/store/graphStore.editorActions.test.js src/store/graphStore.recentNodes.test.js src/components/StrategyCanvas.interaction.test.jsx src/graph/createNode.test.js`: passed, 5 files / 16 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.editor_store_actions.edge_removal_actions`
