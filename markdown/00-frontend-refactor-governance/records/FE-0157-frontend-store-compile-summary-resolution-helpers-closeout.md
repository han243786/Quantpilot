# FE-0157 Frontend Store Compile Summary Resolution Helpers Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.compile_flow.compile_helper_contract`
- Closed leaf: `frontend.store.compile_flow.compile_helper_contract.summary_resolution_helpers`
- Code surfaces:
  - `frontend/src/store/graphStoreCompileSummary.js`
  - `frontend/src/store/graphStoreCompileHelpers.js`
  - `frontend/src/store/graphStore.strategyIrCompile.test.js`
  - `frontend/src/store/graphStoreCompileOutcomeProjection.test.js`

## Change

- Extracted compile summary merge, Strategy IR check summary, artifact resolution summary, and compile failure summary into `graphStoreCompileSummary.js`.
- Kept `graphStoreCompileHelpers.js` as a compatibility re-export parent for existing `graphStoreHelpers.js` imports.

## Whitebox Boundary

- Inputs:
  - Local compile summary, backend compile response, graph, compile source, backend error, Strategy IR compile response, runtime source, and Strategy IR artifact presence.
- Processing:
  - Merge backend diagnostics into local compile summary with backend verification metadata.
  - Build Strategy IR check summaries for absent or performed checks.
  - Resolve runtime source labels and source-of-truth notes for runtime/Formal/fallback paths.
  - Humanize backend compile errors and build failure summaries with normalized diagnostics from the diagnostics helper leaf.
- Outputs:
  - Backend-verified success compile summaries.
  - Compile failure summaries with backend error and diagnostics.
  - Strategy IR check summaries.
  - Artifact resolution summaries.
- Parent communication:
  - Outcome projection uses summary helpers through `graphStoreHelpers.js`.
  - Backend protocol flow uses artifact resolution summaries through `graphStoreHelpers.js`.
  - Summary helpers depend on diagnostics helpers but diagnostics helpers do not depend on summary helpers.

## Recursive Split Decision

- No further split is required now.
- Summary/resolution helpers are deterministic projection functions with a single dependency on diagnostics and compile contract labels.
- Continue the parent queue through `frontend.store.compile_flow.compile_helper_contract.strategy_ir_protocol_mapping_bridge`.

## Equivalence Baseline

- Backend success summaries still mark `backend_verified`, preserve protocol/config/output metadata, and merge diagnostics.
- Backend failure summaries still humanize errors, preserve compile source diagnostics, and mark `backend_verified` false.
- Strategy IR check summaries still distinguish absent checks from performed checks.
- Artifact resolution still labels runtime, Formal QuantScript, and runtime fallback paths.

## Verification

- `npm.cmd test -- --run src/store/graphStore.strategyIrCompile.test.js src/store/graphStoreCompileOutcomeProjection.test.js`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
