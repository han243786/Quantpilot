# FE-0153 Frontend Store Compile Backend Flow Contract Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.compile_flow`
- Closed leaf: `frontend.store.compile_flow.backend_compile_flow_contract`
- Code surfaces:
  - `frontend/src/store/graphStoreCompileFlow.js`
  - `frontend/src/store/graphStoreCompileProtocolFlow.js`
  - `frontend/src/store/graphStoreCompileApi.js`
  - `frontend/src/store/graphStore.strategyIrCompile.test.js`
  - `frontend/src/store/graphStoreCompileOutcomeProjection.test.js`

## Change

- Closed the backend compile flow contract as an already-isolated leaf.
- Kept the existing three-layer boundary:
  - `graphStoreCompileFlow.js` owns the public `runGraphCompileFlow` orchestration.
  - `graphStoreCompileProtocolFlow.js` owns Strategy IR verification, Formal QuantScript compile, runtime fallback, and runtime result normalization.
  - `graphStoreCompileApi.js` owns backend request endpoints.

## Whitebox Boundary

- Inputs:
  - Local compile context, registry, graph metadata/artifacts, Formal QuantScript source, runtime config, Strategy IR artifact, and backend compile endpoints.
- Processing:
  - Build local compile context before backend calls.
  - Return validation-failure outcome when local compile is not compilable.
  - Verify Strategy IR artifacts when present.
  - Prefer Formal QuantScript backend compile when source is available.
  - Fall back to runtime compile for empty Formal source or 5xx/network Formal compile failures.
  - Preserve non-5xx Formal compile failures as Formal QuantScript errors.
- Outputs:
  - Validation-failure, backend-failure, or success compile outcome.
  - Verified Strategy IR summary when available.
  - Runtime compile result with backend runtime targets as authority when returned.
- Parent communication:
  - `graphStoreCompileCurrentGraphActions.js` invokes `runGraphCompileFlow`.
  - Outcome and state mapping remain handled by queued sibling contracts.
  - API transport remains routed through `graphStorePersistenceHelpers.postJson`.

## Recursive Split Decision

- No further split is required now.
- The backend contract already has file-level separation between orchestration, protocol behavior, and endpoint requests.
- Additional subdivision would mostly rename compact functions without reducing dependency coupling.
- Continue the parent queue through `frontend.store.compile_flow.outcome_state_contract`.

## Equivalence Baseline

- Strategy IR compile verification still runs only when a Strategy IR artifact exists.
- Formal QuantScript compile still uses backend `/quantscript/formal/compile` first when source exists.
- Empty Formal source and 5xx/network Formal failures still fall back to `/runtime/compile`.
- 4xx Formal failures still remain Formal QuantScript errors.
- Runtime targets still prefer backend-provided targets over frontend-local estimates.

## Verification

- `npm.cmd test -- --run src/store/graphStore.strategyIrCompile.test.js src/store/graphStoreCompileOutcomeProjection.test.js`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
