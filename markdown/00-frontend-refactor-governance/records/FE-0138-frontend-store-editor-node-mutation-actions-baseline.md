# FE-0138 Frontend Store Editor Node Mutation Actions Baseline

Status: baseline established.

## Scope

- Parent node: `frontend.store.editor_actions`
- Active nested child parent: `frontend.store.editor_actions.node_mutation_actions`
- This is a docs-only recursive baseline for node mutation actions.

## Owned Files

- `frontend/src/store/graphStoreEditorNodeActions.js`
- `frontend/src/store/graphStoreEditorNodeActions.test.js`
- `frontend/src/store/graphStore.editorActions.test.js`
- `frontend/src/store/graphStore.recentNodes.test.js`
- `frontend/src/components/StrategyCanvas.jsx`
- `frontend/src/components/ModuleSidebar.jsx`
- `frontend/src/nodes/BaseNodeCard.jsx`
- `frontend/src/components/propertyPanelEntityCards.jsx`
- `frontend/src/components/propertyPanelSectionComposers.jsx`

## Whitebox Boundary

- Inputs:
  - Module key, node id, node position, viewport, config key/value, node name, and current registry/graph state.
  - UI calls from module sidebar, graph canvas, node cards, and property-panel entity cards.
- Processing:
  - Node creation builds a module-backed node and selects it.
  - Position and viewport actions update node layout/editor viewport with optional persistence.
  - Config and identity actions mutate node config/name and refresh validation.
  - UI-collapse action toggles node collapsed state.
- Outputs:
  - Updated graph, validation state, selected node/edge state, compile result reset, graph-source draft, recent node ids, and graph persistence.
- Parent communication:
  - Public methods are exposed through `graphStore.js` via `createGraphStoreEditorActions`.
  - Node mutation subleaves must communicate through `graphStoreEditorNodeActions.js` or the `frontend.store.editor_actions` parent.

## Recursive Child Queue

- `frontend.store.editor_actions.node_mutation_actions.node_creation`
- `frontend.store.editor_actions.node_mutation_actions.node_position_viewport`
- `frontend.store.editor_actions.node_mutation_actions.node_config_identity`
- `frontend.store.editor_actions.node_mutation_actions.node_ui_collapse`

## Split Decision

- This leaf is worth recursive split.
- Hard-rule assessment:
  - The file exposes multiple independently meaningful public methods.
  - Creation, position/viewport, config/name, and collapse state have different consumers and state invariants.
  - The file repeats validation, recent-node, persistence, and draft refresh patterns.
  - Each subleaf can be verified with targeted node action tests.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
