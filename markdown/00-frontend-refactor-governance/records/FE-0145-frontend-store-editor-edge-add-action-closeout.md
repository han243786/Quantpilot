# FE-0145 Frontend Store Editor Edge Add Action Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.editor_actions.edge_mutation_actions`
- Closed leaf: `frontend.store.editor_actions.edge_mutation_actions.add_edge_action`
- Code surfaces:
  - `frontend/src/store/graphStoreEditorEdgeCreationActions.js`
  - `frontend/src/store/graphStoreEditorEdgeActions.js`
  - `frontend/src/store/graphStoreEditorEdgeActions.test.js`
  - `frontend/src/components/StrategyCanvas.jsx`

## Change

- Extracted `addEdge` into `graphStoreEditorEdgeCreationActions.js`.
- Kept `graphStoreEditorEdgeActions.js` as the edge mutation parent composer while `removeSelected` remains queued for the next leaf.

## Whitebox Boundary

- Inputs:
  - React Flow connection payload with source/target node ids and handle ids.
  - Current registry and graph state.
- Processing:
  - Create a graph edge with stable endpoint and port fields.
  - Update graph metadata timestamp.
  - Record touched source and target node ids.
  - Attach validation and persist graph storage.
- Outputs:
  - Updated graph.
  - Cleared compile result.
  - Refreshed graph-source `quantScriptDraft`.
- Parent communication:
  - `graphStoreEditorEdgeActions.js` composes this leaf.
  - `graphStore.js` exposes `addEdge` through `createGraphStoreEditorActions`.

## Recursive Split Decision

- No further split is required.
- The leaf owns one public edge creation action and one coherent persistence/validation path.
- Continue the parent queue through `frontend.store.editor_actions.edge_mutation_actions.remove_selected_action`.

## Equivalence Baseline

- Adding an edge still appends one graph edge, records both endpoint node ids as recent, clears compile result, refreshes graph source, and preserves validation.

## Verification

- `npm.cmd test -- --run src/store/graphStoreEditorEdgeActions.test.js src/store/graphStore.recentNodes.test.js src/components/StrategyCanvas.interaction.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
