# FE-0149 Frontend Store Compile Flow Baseline

Status: baseline established.

## Scope

- Parent node: `frontend.store`
- Active child parent: `frontend.store.compile_flow`
- This is a docs-only recursive baseline for the store compile flow.

## Owned Files

- `frontend/src/store/graphStoreCompileActions.js`
- `frontend/src/store/graphStoreCompileApi.js`
- `frontend/src/store/graphStoreCompileFlow.js`
- `frontend/src/store/graphStoreCompileHelpers.js`
- `frontend/src/store/graphStoreCompileOutcomeMapping.js`
- `frontend/src/store/graphStoreCompileOutcomeProjection.js`
- `frontend/src/store/graphStoreCompileOutcomeProjection.test.js`
- `frontend/src/store/graphStoreCompileProtocolFlow.js`
- `frontend/src/store/graphStoreCompileProtocolMapping.js`
- `frontend/src/store/graphStoreCompileState.js`
- `frontend/src/store/graphStore.export.test.js`
- `frontend/src/store/graphStore.strategyIrCompile.test.js`
- `frontend/src/store/graphStoreCompileOutcomeProjection.test.js`
- `frontend/src/components/CompilePanel.integration.test.jsx`

## Whitebox Boundary

- Inputs:
  - Current graph, registry, source drafts, formal source override, capability status/source/message, capability list, compile result, and action lock.
  - UI calls from toolbar compile/export actions and source apply actions.
  - Backend compile protocol responses for Strategy IR, Formal QuantScript, and runtime compile.
- Processing:
  - Source apply actions normalize Formal QuantScript and Strategy IR drafts.
  - Export actions provide QuantScript and runtime config exports.
  - Current graph compile gates validation/action lock/capabilities, runs the local compile, verifies Strategy IR, compiles runtime source, persists the next graph, and projects success/failure state.
  - Protocol flow owns backend API ordering and Formal QuantScript fallback.
  - Outcome/state helpers own graph compile summaries, runtime target projection, drafts, notices, and compile result shapes.
- Outputs:
  - Updated graph, compile summary, compile result, runtime config/targets, source drafts, compile notices, action lock state, backend diagnostics, and graph persistence.
- Parent communication:
  - Public methods are exposed through `graphStore.js` via `createGraphStoreCompileActions`.
  - Compile-flow children must communicate through `graphStoreCompileActions.js` or the `frontend.store` parent.

## Recursive Child Queue

- `frontend.store.compile_flow.source_apply_actions`
- `frontend.store.compile_flow.export_actions`
- `frontend.store.compile_flow.current_graph_compile_action`
- `frontend.store.compile_flow.backend_compile_flow_contract`
- `frontend.store.compile_flow.outcome_state_contract`
- `frontend.store.compile_flow.compile_helper_contract`

## Split Decision

- This parent is worth recursive split.
- Hard-rule assessment:
  - The parent exposes multiple public contracts that change for unrelated reasons.
  - Source apply, export, current compile, backend protocol, outcome projection, and helper contracts have independent failure modes.
  - The helper and protocol files are large enough to hide compile diagnostics or fallback hallucinations if not white-boxed.
  - Each child can be verified with existing compile, export, Strategy IR, outcome projection, and integration tests.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
