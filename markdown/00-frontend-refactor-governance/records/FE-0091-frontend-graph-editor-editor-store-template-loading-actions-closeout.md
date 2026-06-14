# FE-0091 Frontend Graph Editor Editor Store Template Loading Actions Closeout

Status: closed.

## Child Node

`frontend.graph_editor.editor_store_actions.template_loading_actions`

## Boundary

This leaf owns graph store template loading and its associated editor, version-preview, compile, and active runtime focus resets. `graphStoreEditorActions.js` remains the parent facade that exposes `loadStrategyTemplate` and composes this child with the rest of editor store actions.

## Changed Files

- `frontend/src/store/graphStoreEditorActions.js`
- `frontend/src/store/graphStoreEditorTemplateActions.js`
- `frontend/src/store/graphStoreEditorTemplateActions.test.js`

## Public Methods

- `loadStrategyTemplate`

## Preserved Behavior

- Template loading still builds through `buildStrategyTemplateGraph`, attaches validation, persists the graph, and returns the loaded graph.
- Loading a template still clears editor node/edge/diagnostic focus, compile result, formal source drafts, graph version preview/compare state, and active runtime selections.
- Runtime history, backtest history, and experiment history remain preserved while active runtime focus fields are reset.
- No child-to-child store action calls were introduced; the parent facade composes the child action factory.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; template loading has a high side-effect cluster distinct from draft editing, selection focus, node mutation, and edge mutation.
- `leaf_split_positive_trigger`: `semantic_boundary`, `independent_failure_mode`, `testability_gain`, and `blast_radius_reduction`.
- `leaf_split_stop_condition`: reached for `template_loading_actions`; the child owns one cohesive public action with direct white-box coverage.
- `leaf_split_decision_result`: continue splitting `frontend.graph_editor.editor_store_actions` through node mutation actions next.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/store/graphStoreEditorTemplateActions.test.js src/store/graphStore.templates.test.js src/store/graphStore.editorActions.test.js src/store/graphStoreEditorDraftActions.test.js`: passed, 4 files / 10 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Leaf

`frontend.graph_editor.editor_store_actions.node_mutation_actions`
