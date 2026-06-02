# FE-0140 Frontend Store Editor Node Position Viewport Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.editor_actions.node_mutation_actions`
- Closed leaf: `frontend.store.editor_actions.node_mutation_actions.node_position_viewport`
- Code surfaces:
  - `frontend/src/store/graphStoreEditorNodePositionActions.js`
  - `frontend/src/store/graphStoreEditorNodeActions.js`
  - `frontend/src/store/graphStoreEditorNodeActions.test.js`
  - `frontend/src/components/StrategyCanvas.jsx`

## Change

- Extracted `updateNodePosition` and `updateEditorViewport` into `graphStoreEditorNodePositionActions.js`.
- Kept `graphStoreEditorNodeActions.js` as the node mutation facade.

## Whitebox Boundary

- Inputs:
  - Node id and next node position.
  - Editor viewport.
  - `persist` flag.
  - Current registry and graph state.
- Processing:
  - `updateNodePosition` updates node layout, attaches validation, optionally records recent node ids and persists graph storage.
  - `updateEditorViewport` updates editor viewport metadata, optionally updates `updated_at`, and optionally persists graph storage.
- Outputs:
  - Updated graph.
  - Cleared compile result for node position changes.
  - refreshed graph-source `quantScriptDraft` for node position changes.
  - updated editor viewport metadata.
- Parent communication:
  - `graphStoreEditorNodeActions.js` composes this leaf.
  - `graphStore.js` exposes both public methods through `createGraphStoreEditorActions`.

## Recursive Split Decision

- No further split is required.
- The leaf owns the editor layout plane and viewport persistence transition.
- Next queued leaf: `frontend.store.editor_actions.node_mutation_actions.node_config_identity`.

## Equivalence Baseline

- Dragging a node without persistence updates position without saving graph storage.
- Persisted node position updates refresh validation, clear compile result, refresh graph source, and update recent node ids.
- Viewport updates mutate metadata and persist only when requested.

## Verification

- `npm.cmd test -- --run src/store/graphStoreEditorNodeActions.test.js src/components/StrategyCanvas.interaction.test.jsx src/components/StrategyCanvas.focus.test.jsx src/components/strategyCanvasViewport.test.js`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
