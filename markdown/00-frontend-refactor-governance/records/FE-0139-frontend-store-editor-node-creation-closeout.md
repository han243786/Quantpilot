# FE-0139 Frontend Store Editor Node Creation Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.editor_actions.node_mutation_actions`
- Closed leaf: `frontend.store.editor_actions.node_mutation_actions.node_creation`
- Code surfaces:
  - `frontend/src/store/graphStoreEditorNodeCreationActions.js`
  - `frontend/src/store/graphStoreEditorNodeActions.js`
  - `frontend/src/store/graphStoreEditorNodeActions.test.js`
  - `frontend/src/store/graphStore.editorActions.test.js`
  - `frontend/src/components/ModuleSidebar.jsx`

## Change

- Extracted `createNode` into `graphStoreEditorNodeCreationActions.js`.
- Kept `graphStoreEditorNodeActions.js` as the node mutation facade for current and future node subleaves.

## Whitebox Boundary

- Inputs:
  - Module key.
  - Current registry and graph state.
- Processing:
  - Resolve module definition.
  - Reject missing or unsupported modules.
  - Create a module-backed node.
  - Append it to the graph, record recent node ids, attach validation, persist storage, and select the new node.
- Outputs:
  - Updated graph.
  - `selectedNodeId` set to the new node.
  - `selectedEdgeId` cleared.
  - refreshed graph-source `quantScriptDraft`.
- Parent communication:
  - `graphStoreEditorNodeActions.js` composes this leaf.
  - `graphStore.js` exposes `createNode` through `createGraphStoreEditorActions`.

## Recursive Split Decision

- No further split is required.
- The leaf owns one public action and one coherent state transition.
- Next queued leaf: `frontend.store.editor_actions.node_mutation_actions.node_position_viewport`.

## Equivalence Baseline

- Valid module keys add a node, select it, clear edge selection, refresh validation and graph source.
- Invalid module keys do not mutate the graph.
- Unsupported modules remain rejected.

## Verification

- `npm.cmd test -- --run src/store/graphStoreEditorNodeActions.test.js src/store/graphStore.editorActions.test.js src/components/ModuleSidebar.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
