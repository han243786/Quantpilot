# FE-0141 Frontend Store Editor Node Config Identity Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.editor_actions.node_mutation_actions`
- Closed leaf: `frontend.store.editor_actions.node_mutation_actions.node_config_identity`
- Code surfaces:
  - `frontend/src/store/graphStoreEditorNodeConfigActions.js`
  - `frontend/src/store/graphStoreEditorNodeActions.js`
  - `frontend/src/store/graphStoreEditorNodeActions.test.js`
  - `frontend/src/store/graphStore.recentNodes.test.js`
  - `frontend/src/components/propertyPanelEntityCards.jsx`
  - `frontend/src/components/propertyPanelSectionComposers.jsx`
  - `frontend/src/nodes/BaseNodeCard.jsx`

## Change

- Extracted `updateNodeConfig` and `updateNodeName` into `graphStoreEditorNodeConfigActions.js`.
- Kept `graphStoreEditorNodeActions.js` as the node mutation facade.

## Whitebox Boundary

- Inputs:
  - Node id.
  - Config key/value.
  - Node display name.
  - Current registry and graph state.
- Processing:
  - `updateNodeConfig` mutates a node config entry, records the node as recent, attaches validation, persists graph storage, clears compile result, and refreshes graph-source draft.
  - `updateNodeName` mutates node identity text with the same validation/persistence/draft refresh path.
- Outputs:
  - Updated graph.
  - Cleared compile result.
  - refreshed graph-source `quantScriptDraft`.
  - updated recent node ids.
- Parent communication:
  - `graphStoreEditorNodeActions.js` composes this leaf.
  - `graphStore.js` exposes both public methods through `createGraphStoreEditorActions`.

## Recursive Split Decision

- No further split is required.
- Config and display identity share the same node edit persistence invariant and property-panel consumer path.
- Next queued leaf: `frontend.store.editor_actions.node_mutation_actions.node_ui_collapse`.

## Equivalence Baseline

- Node config edits update the target node config, refresh validation, reset compile result, refresh graph source, and record recent node ids.
- Node name edits update target node identity with the same persistence and validation behavior.

## Verification

- `npm.cmd test -- --run src/store/graphStoreEditorNodeActions.test.js src/store/graphStore.recentNodes.test.js src/store/graphStore.editorActions.test.js src/components/propertyPanelEntityCards.test.jsx src/components/propertyPanelSectionComposers.test.jsx src/nodes/BaseNodeCard.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
