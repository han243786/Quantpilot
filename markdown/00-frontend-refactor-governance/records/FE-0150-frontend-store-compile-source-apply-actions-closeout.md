# FE-0150 Frontend Store Compile Source Apply Actions Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.compile_flow`
- Closed leaf: `frontend.store.compile_flow.source_apply_actions`
- Code surfaces:
  - `frontend/src/store/graphStoreCompileSourceActions.js`
  - `frontend/src/store/graphStoreCompileActions.js`
  - `frontend/src/store/graphStore.strategyIrDraft.test.js`
  - `frontend/src/store/graphStore.strategyIrCompile.test.js`
  - `frontend/src/components/CompilePanel.integration.test.jsx`

## Change

- Extracted `applyFormalQuantScriptSource` and `applyStrategyIrSource` into `graphStoreCompileSourceActions.js`.
- Kept `graphStoreCompileActions.js` as the compile-flow parent composer while export and current compile actions remain queued.

## Whitebox Boundary

- Inputs:
  - Formal QuantScript source override or draft.
  - Strategy IR JSON source override or draft.
  - Current graph and registry state.
- Processing:
  - Apply Formal QuantScript source to the draft/override state and clear compile result.
  - Parse and normalize Strategy IR JSON.
  - Attach Strategy IR artifact metadata, label targets, validation, graph persistence, and draft projection.
- Outputs:
  - Updated formal source draft and override.
  - Updated graph source mode and Strategy IR artifact.
  - Cleared compile result and compile diagnostic focus.
  - Refreshed graph-source and Strategy IR drafts.
- Parent communication:
  - `graphStoreCompileActions.js` composes this leaf.
  - `graphStore.js` exposes source apply actions through `createGraphStoreCompileActions`.

## Recursive Split Decision

- No further split is required now.
- The two public source apply methods share one source-application boundary and are verified by Strategy IR/source compile tests.
- Continue the parent queue through `frontend.store.compile_flow.export_actions`.

## Equivalence Baseline

- Formal source apply still rejects blank source and updates draft/override state.
- Strategy IR apply still rejects invalid JSON, persists graph storage, sets source mode, stores normalized Strategy IR artifacts, and refreshes derived drafts.

## Verification

- `npm.cmd test -- --run src/store/graphStore.strategyIrDraft.test.js src/store/graphStore.strategyIrCompile.test.js src/components/CompilePanel.integration.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
