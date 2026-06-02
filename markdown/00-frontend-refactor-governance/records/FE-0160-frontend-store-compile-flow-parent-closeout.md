# FE-0160 Frontend Store Compile Flow Parent Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Closed child parent: `frontend.store.compile_flow`
- This is a docs-only parent closeout. No frontend source files changed in this step.

## Closed Leaves

- `frontend.store.compile_flow.source_apply_actions`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0150-frontend-store-compile-source-apply-actions-closeout.md`
  - Public surface: `frontend/src/store/graphStoreCompileSourceActions.js`
- `frontend.store.compile_flow.export_actions`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0151-frontend-store-compile-export-actions-closeout.md`
  - Public surface: `frontend/src/store/graphStoreCompileExportActions.js`
- `frontend.store.compile_flow.current_graph_compile_action`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0152-frontend-store-compile-current-graph-action-closeout.md`
  - Public surface: `frontend/src/store/graphStoreCompileCurrentGraphActions.js`
- `frontend.store.compile_flow.backend_compile_flow_contract`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0153-frontend-store-compile-backend-flow-contract-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreCompileFlow.js`
    - `frontend/src/store/graphStoreCompileProtocolFlow.js`
    - `frontend/src/store/graphStoreCompileApi.js`
- `frontend.store.compile_flow.outcome_state_contract`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0154-frontend-store-compile-outcome-state-contract-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreCompileState.js`
    - `frontend/src/store/graphStoreCompileOutcomeMapping.js`
    - `frontend/src/store/graphStoreCompileOutcomeProjection.js`
- `frontend.store.compile_flow.compile_helper_contract`
  - Parent closeout record: `markdown/00-frontend-refactor-governance/records/FE-0159-frontend-store-compile-helper-contract-parent-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreCompileHelpers.js`
    - `frontend/src/store/graphStoreCompileDiagnostics.js`
    - `frontend/src/store/graphStoreCompileSummary.js`
    - `frontend/src/store/graphStoreCompileProtocolMapping.js`

## Public Surfaces Preserved

- `frontend/src/store/graphStoreCompileActions.js`
- `frontend/src/store/graphStoreCompileSourceActions.js`
- `frontend/src/store/graphStoreCompileExportActions.js`
- `frontend/src/store/graphStoreCompileCurrentGraphActions.js`
- `frontend/src/store/graphStoreCompileFlow.js`
- `frontend/src/store/graphStoreCompileProtocolFlow.js`
- `frontend/src/store/graphStoreCompileApi.js`
- `frontend/src/store/graphStoreCompileState.js`
- `frontend/src/store/graphStoreCompileOutcomeMapping.js`
- `frontend/src/store/graphStoreCompileOutcomeProjection.js`
- `frontend/src/store/graphStoreCompileHelpers.js`

## Recursive Decision

- The compile flow child queue is closed.
- Nested helper contract is closed after its own recursive subchild queue finished.
- `graphStoreCompileActions.js` remains the parent composition boundary for source, export, and current graph compile actions.
- Backend compile flow, outcome projection, state builders, diagnostics, summaries, and protocol mapping now have explicit white-box surfaces.
- The parent returns control to `frontend.store`.
- Next queued store child: `frontend.store.runtime_session`.

## Equivalence Evidence

- FE-0150 through FE-0159 each landed with targeted tests or docs-only closeout gates.
- Source-changing compile flow leaves landed with build verification and full frontend pre-commit verification.
- This parent closeout only changes frontend-local governance files and records the already-verified child queue completion.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
