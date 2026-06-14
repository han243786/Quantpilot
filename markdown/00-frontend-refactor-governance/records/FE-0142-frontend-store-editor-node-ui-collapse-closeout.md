# FE-0142 Frontend Store Editor Node UI Collapse Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.editor_actions.node_mutation_actions`
- Closed leaf: `frontend.store.editor_actions.node_mutation_actions.node_ui_collapse`
- Code surfaces:
  - `frontend/src/store/graphStoreEditorNodeUiActions.js`
  - `frontend/src/store/graphStoreEditorNodeActions.js`
  - `frontend/src/store/graphStoreEditorNodeActions.test.js`
  - `frontend/src/nodes/BaseNodeCard.jsx`

## Change

- Extracted `toggleNodeCollapse` into `graphStoreEditorNodeUiActions.js`.
- Reduced `graphStoreEditorNodeActions.js` to a composition facade for node creation, position/viewport, config/identity, and UI collapse leaves.

## Whitebox Boundary

- Inputs:
  - Node id.
  - Current registry and graph state.
- Processing:
  - Toggle the target node `ui_state.collapsed` value.
  - Record recent node ids.
  - Attach validation and persist graph storage.
- Outputs:
  - Updated graph.
  - Cleared compile result.
  - refreshed graph-source `quantScriptDraft`.
- Parent communication:
  - `graphStoreEditorNodeActions.js` composes this leaf.
  - `graphStore.js` exposes `toggleNodeCollapse` through `createGraphStoreEditorActions`.

## Recursive Split Decision

- No further split is required.
- The leaf owns one public UI-state mutation and one coherent persistence path.
- The nested subchild queue for `frontend.store.editor_actions.node_mutation_actions` is now closed.

## Equivalence Baseline

- Toggling collapse flips only the target node collapsed state, refreshes validation and graph source, clears compile result, persists graph storage, and records the node as recent.

## Verification

- `npm.cmd test -- --run src/store/graphStoreEditorNodeActions.test.js src/nodes/BaseNodeCard.test.jsx src/store/graphStore.recentNodes.test.js`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
