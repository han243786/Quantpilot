# FE-0152 Frontend Store Compile Current Graph Action Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.compile_flow`
- Closed leaf: `frontend.store.compile_flow.current_graph_compile_action`
- Code surfaces:
  - `frontend/src/store/graphStoreCompileCurrentGraphActions.js`
  - `frontend/src/store/graphStoreCompileActions.js`
  - `frontend/src/store/graphStore.strategyIrCompile.test.js`
  - `frontend/src/store/graphStore.runtimeActionLock.test.js`
  - `frontend/src/store/graphStore.runtimeErrors.test.js`
  - `frontend/src/components/CompilePanel.integration.test.jsx`

## Change

- Extracted `compileCurrentGraph` and its capability-block state builder into `graphStoreCompileCurrentGraphActions.js`.
- Reduced `graphStoreCompileActions.js` to the compile-flow parent composer for source, export, and current graph compile leaves.

## Whitebox Boundary

- Inputs:
  - Current graph, registry, capability status/source/message/capabilities, action lock, Formal QuantScript override, and Strategy IR draft.
- Processing:
  - Reject invalid graphs and concurrent action locks before compile starts.
  - Apply capability permission boundary checks before backend compile flow.
  - Run graph compile flow, persist the next graph, discard stale compile results after graph-id changes, and map validation/failure/success outcomes to store state.
  - Always release the compile action lock in `finally`.
- Outputs:
  - Backend-verified compile result on success.
  - Validation or backend failure state on compile failures.
  - Capability-blocked compile state when the permission boundary blocks compile.
  - Compile result notice and runtime/action-lock state updates.
- Parent communication:
  - `graphStoreCompileActions.js` composes this leaf.
  - Export actions continue calling `compileCurrentGraph` through the composed store public surface.
  - `graphStore.js` exposes current graph compile through `createGraphStoreCompileActions`.

## Recursive Split Decision

- No further split is required for this leaf now.
- Outcome builders, backend compile flow, and helper contracts remain explicit queued siblings under `frontend.store.compile_flow`.
- Continue the parent queue through `frontend.store.compile_flow.backend_compile_flow_contract`.

## Equivalence Baseline

- Compile still exits on invalid graph state or existing action lock.
- Compile still blocks on capability boundary denial and records `CAPABILITY_BOUNDARY` diagnostics.
- Compile still persists the next graph, discards stale results, maps validation/failure/success outcomes, and clears `actionLock` after every path.

## Verification

- `npm.cmd test -- --run src/store/graphStore.strategyIrCompile.test.js src/store/graphStore.runtimeActionLock.test.js src/store/graphStore.runtimeErrors.test.js src/components/CompilePanel.integration.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
