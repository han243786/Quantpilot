# FE-0135 Frontend Store Editor Selection Focus Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.editor_actions`
- Closed leaf: `frontend.store.editor_actions.selection_focus`
- Primary files:
  - `frontend/src/store/graphStoreEditorSelectionActions.js`
  - `frontend/src/store/graphStoreEditorSelectionActions.test.js`
  - `frontend/src/store/graphStore.editorActions.test.js`

## Whitebox Boundary

- Inputs:
  - Node id, edge id, compile diagnostic target, current graph, and current Strategy IR draft.
  - UI calls from canvas selection, diagnostics focus, workspace issue focus, event/runtime node focus, and property-panel Strategy IR focus.
- Processing:
  - `setSelectedNode` selects a node and clears edge and compile diagnostic focus.
  - `setSelectedEdge` selects an edge and clears node and compile diagnostic focus.
  - `focusCompileDiagnostic` normalizes compile targets, maps node/edge diagnostics to editor selections, and preserves graph/Strategy IR diagnostics in `selectedCompileDiagnosticTarget`.
- Outputs:
  - `selectedNodeId`
  - `selectedEdgeId`
  - `selectedCompileDiagnosticTarget`
  - refreshed `strategyIrDraft` for Strategy IR diagnostics.
- Parent communication:
  - Public methods are exposed through `graphStore.js` via `createGraphStoreEditorActions`.

## Recursive Split Decision

- No further subleaf split is required.
- Hard-rule assessment:
  - The file is a small cohesive focus-state adapter.
  - The public method set is tightly coupled by the invariant that only one focus target may be active.
  - Existing tests cover node, edge, graph, and Strategy IR target paths.
  - Splitting would create artificial pass-through helpers without reducing dependency risk.
- Next queued leaf: `frontend.store.editor_actions.draft_source_actions`.

## Equivalence Baseline

- Node selection clears edge and diagnostic focus.
- Edge selection clears node and diagnostic focus.
- Node/edge compile diagnostics become normal editor selections.
- Graph and Strategy IR diagnostics remain diagnostic-focus records.

## Verification

- `npm.cmd test -- --run src/store/graphStoreEditorSelectionActions.test.js src/store/graphStore.editorActions.test.js src/store/graphStore.strategyIrDraft.test.js src/components/DiagnosticsPanel.test.jsx src/components/StrategyCanvas.interaction.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
