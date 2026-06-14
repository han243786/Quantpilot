# FE-0146 Frontend Store Editor Edge Remove Selected Action Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.editor_actions.edge_mutation_actions`
- Closed leaf: `frontend.store.editor_actions.edge_mutation_actions.remove_selected_action`
- Code surfaces:
  - `frontend/src/store/graphStoreEditorEdgeRemovalActions.js`
  - `frontend/src/store/graphStoreEditorEdgeActions.js`
  - `frontend/src/store/graphStoreEditorEdgeActions.test.js`
  - `frontend/src/components/PropertyPanel.jsx`
  - `frontend/src/components/propertyPanelEntityCards.jsx`
  - `frontend/src/components/propertyPanelSectionComposers.jsx`

## Change

- Extracted `removeSelected` into `graphStoreEditorEdgeRemovalActions.js`.
- Reduced `graphStoreEditorEdgeActions.js` to a composition facade for edge creation and selected-removal leaves.

## Whitebox Boundary

- Inputs:
  - Selected node id, selected edge id, current registry, and current graph state.
- Processing:
  - Return early when no node or edge is selected.
  - Remove a selected node and its incident edges, or remove the selected edge only.
  - Filter recent node ids through the resulting graph.
  - Attach validation and persist graph storage.
- Outputs:
  - Updated graph.
  - Cleared selected node and edge ids.
  - Cleared compile result.
  - Refreshed graph-source `quantScriptDraft`.
  - Preserved runtime shell state.
- Parent communication:
  - `graphStoreEditorEdgeActions.js` composes this leaf.
  - `graphStore.js` exposes `removeSelected` through `createGraphStoreEditorActions`.

## Recursive Split Decision

- No further split is required.
- The leaf owns one public selected-removal action with one shared validation and selection cleanup path.
- The nested subchild queue for `frontend.store.editor_actions.edge_mutation_actions` is now closed.

## Equivalence Baseline

- Removing a selected node still deletes the node and incident edges, filters recent node ids, clears selections, clears compile result, refreshes graph source, and preserves runtime state.
- Removing a selected edge still deletes only that edge and leaves nodes intact.

## Verification

- `npm.cmd test -- --run src/store/graphStoreEditorEdgeActions.test.js src/store/graphStore.recentNodes.test.js src/components/StrategyCanvas.interaction.test.jsx src/components/propertyPanelEntityCards.test.jsx src/components/propertyPanelSectionComposers.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
