# FE-0154 Frontend Store Compile Outcome State Contract Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.compile_flow`
- Closed leaf: `frontend.store.compile_flow.outcome_state_contract`
- Code surfaces:
  - `frontend/src/store/graphStoreCompileState.js`
  - `frontend/src/store/graphStoreCompileOutcomeMapping.js`
  - `frontend/src/store/graphStoreCompileOutcomeProjection.js`
  - `frontend/src/store/graphStore.strategyIrCompile.test.js`
  - `frontend/src/store/graphStore.strategyIrDraft.test.js`
  - `frontend/src/store/graphStore.export.test.js`
  - `frontend/src/store/graphStoreCompileOutcomeProjection.test.js`

## Change

- Closed the outcome/state contract as an already-isolated leaf.
- Preserved the current split:
  - `graphStoreCompileState.js` owns store-state builders for source apply, validation failure, success, failure, and runtime export fallback.
  - `graphStoreCompileOutcomeMapping.js` owns local compile context creation and the public bridge into outcome projection.
  - `graphStoreCompileOutcomeProjection.js` owns validation, success, failure, Strategy IR verification, failure-source inference, graph artifact attachment, and summary projection.

## Whitebox Boundary

- Inputs:
  - Local compile result, next graph, registry, Strategy IR draft, backend runtime compile result, Strategy IR compile result, compile error, and existing store state.
- Processing:
  - Build local compile context from graph compile output and artifact metadata.
  - Project validation failure outcomes without backend compile state.
  - Project success outcomes with runtime config, runtime targets, backend compile, Strategy IR compile, core IR artifacts, runtime binding, and resolved Strategy IR draft.
  - Project failure outcomes with source-specific diagnostics and compile-source inference.
  - Convert compile outcomes into Zustand-compatible store state.
  - Provide runtime export fallback from the current graph or compile result.
- Outputs:
  - Outcome objects consumed by `runGraphCompileFlow`.
  - Store-state patches consumed by `compileCurrentGraph`, source apply, and export actions.
  - Updated graph compile summary, runtime binding, core IR artifacts, drafts, and compile result payloads.
- Parent communication:
  - `graphStoreCompileFlow.js` uses outcome mapping/projection for flow results.
  - `graphStoreCompileCurrentGraphActions.js` uses store-state builders after compile outcomes.
  - `graphStoreCompileSourceActions.js` and `graphStoreCompileExportActions.js` use the source and fallback builders.

## Recursive Split Decision

- No further split is required now.
- Store state builders and compile outcome projections already live in separate files with distinct callers.
- Public methods are compact, deterministic projection functions and are covered by compile, draft, export, and outcome projection tests.
- Continue the parent queue through `frontend.store.compile_flow.compile_helper_contract`.

## Equivalence Baseline

- Source apply still clears compile result and refreshes drafts.
- Validation failure still records a local compile result with no backend compile error.
- Success still records runtime config, runtime targets, backend compile, Strategy IR compile, graph artifacts, and resolved drafts.
- Failure still infers Strategy IR/Formal/runtime source and preserves Formal QuantScript backend errors.
- Runtime export fallback still derives compile summary from graph or compile result.

## Verification

- `npm.cmd test -- --run src/store/graphStore.strategyIrCompile.test.js src/store/graphStore.strategyIrDraft.test.js src/store/graphStore.export.test.js src/store/graphStoreCompileOutcomeProjection.test.js`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
