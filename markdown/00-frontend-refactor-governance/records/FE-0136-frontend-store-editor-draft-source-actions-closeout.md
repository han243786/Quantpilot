# FE-0136 Frontend Store Editor Draft Source Actions Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.editor_actions`
- Closed leaf: `frontend.store.editor_actions.draft_source_actions`
- Primary files:
  - `frontend/src/store/graphStoreEditorDraftActions.js`
  - `frontend/src/store/graphStoreEditorDraftActions.test.js`
  - `frontend/src/store/graphStore.strategyIrDraft.test.js`
  - `frontend/src/components/PropertyPanel.strategyIr.test.jsx`
  - `frontend/src/components/propertyPanelCompileSourceCards.jsx`

## Whitebox Boundary

- Inputs:
  - QuantScript draft source.
  - Formal QuantScript draft source.
  - Strategy IR draft source.
  - Current graph, registry, selected editor focus, compile result, and runtime shell state.
- Processing:
  - `updateQuantScriptDraft`, `updateFormalQuantScriptDraft`, and `updateStrategyIrDraft` mutate draft text only.
  - `resetQuantScriptDraft`, `resetFormalQuantScriptDraft`, and `resetStrategyIrDraft` restore source drafts and clear compile diagnostic state where required.
  - `applyQuantScriptSource` parses graph-source QuantScript, attaches validation, persists the graph, clears editor focus, and refreshes source drafts.
- Outputs:
  - Updated source drafts.
  - Updated graph and graph validation after applying graph-source QuantScript.
  - Cleared selection/diagnostic/compile result for apply/reset flows.
  - Preserved runtime shell object.
- Parent communication:
  - Public methods are exposed through `graphStore.js` via `createGraphStoreEditorActions`.

## Recursive Split Decision

- No further subleaf split is required.
- Hard-rule assessment:
  - The public methods are a cohesive source-draft workflow.
  - Splitting setter/reset/apply methods would separate tightly coupled invariants for draft source state and diagnostic clearing.
  - Existing tests cover draft update/reset, graph-source apply, Strategy IR focus, and property panel integration.
  - The leaf remains small enough to audit directly.
- Next queued leaf: `frontend.store.editor_actions.template_loading_actions`.

## Equivalence Baseline

- Draft update methods only change their corresponding source strings.
- Reset methods restore graph/formal/Strategy IR drafts and clear compile diagnostic state as before.
- Applying graph-source QuantScript parses into a validated graph, persists storage, clears editor focus, resets compile result, and keeps runtime state.

## Verification

- `npm.cmd test -- --run src/store/graphStoreEditorDraftActions.test.js src/store/graphStore.strategyIrDraft.test.js src/components/PropertyPanel.strategyIr.test.jsx src/components/propertyPanelCompileSourceCards.test.jsx src/components/CompilePanel.integration.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
