# FE-0144 Frontend Store Editor Edge Mutation Actions Baseline

Status: baseline established.

## Scope

- Parent node: `frontend.store.editor_actions`
- Active nested child parent: `frontend.store.editor_actions.edge_mutation_actions`
- This is a docs-only recursive baseline for edge mutation actions.

## Owned Files

- `frontend/src/store/graphStoreEditorEdgeActions.js`
- `frontend/src/store/graphStoreEditorEdgeActions.test.js`
- `frontend/src/store/graphStore.recentNodes.test.js`
- `frontend/src/store/graphStore.editorActions.test.js`
- `frontend/src/components/StrategyCanvas.jsx`
- `frontend/src/components/PropertyPanel.jsx`
- `frontend/src/components/propertyPanelEntityCards.jsx`
- `frontend/src/components/propertyPanelSectionComposers.jsx`
- `frontend/src/hooks/usePropertyPanelActions.js`

## Whitebox Boundary

- Inputs:
  - React Flow connection payloads, selected node id, selected edge id, current registry, and current graph state.
  - UI calls from graph canvas connect events and property-panel delete actions.
- Processing:
  - Edge creation builds a graph edge from the connection payload, records touched endpoint nodes, refreshes validation, persists the graph, and resets compile output.
  - Selected removal deletes either the selected node plus incident edges or the selected edge only, filters recent node ids through the resulting graph, refreshes validation, persists the graph, and clears selections.
- Outputs:
  - Updated graph, validation state, compile result reset, graph-source draft, selection state, recent node ids, and graph persistence.
- Parent communication:
  - Public methods are exposed through `graphStore.js` via `createGraphStoreEditorActions`.
  - Edge mutation subleaves must communicate through `graphStoreEditorEdgeActions.js` or the `frontend.store.editor_actions` parent.

## Recursive Child Queue

- `frontend.store.editor_actions.edge_mutation_actions.add_edge_action`
- `frontend.store.editor_actions.edge_mutation_actions.remove_selected_action`

## Split Decision

- This leaf is worth recursive split.
- Hard-rule assessment:
  - The file exposes two independently meaningful public methods.
  - Edge creation and selected deletion are separate user workflows with different UI callers.
  - Creation mutates graph topology by adding a connection; deletion mutates topology and selection cleanup.
  - Each subleaf has a direct targeted edge action test path.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
