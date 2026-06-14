# FE-0155 Frontend Store Compile Helper Contract Baseline

Status: baseline established.

## Scope

- Parent node: `frontend.store.compile_flow`
- Active nested child parent: `frontend.store.compile_flow.compile_helper_contract`
- This is a docs-only recursive baseline for compile helper extraction.

## Owned Files

- `frontend/src/store/graphStoreCompileHelpers.js`
- `frontend/src/store/graphStoreCompileProtocolMapping.js`
- `frontend/src/store/graphStoreCompileOutcomeMapping.js`
- `frontend/src/store/graphStoreCompileOutcomeProjection.js`
- `frontend/src/store/graphStoreCompileProtocolFlow.js`
- `frontend/src/store/graphStore.diagnostics.test.js`
- `frontend/src/store/graphStore.strategyIrCompile.test.js`
- `frontend/src/store/graphStoreCompileOutcomeProjection.test.js`

## Whitebox Boundary

- Inputs:
  - Local and backend compile summaries.
  - Backend compile diagnostics and error payloads.
  - Graph QuantScript/Strategy IR artifacts and label targets.
  - Runtime source selection and Strategy IR artifact presence.
  - Core IR artifacts, Strategy IR document/source/draft, and compile protocol payloads.
- Processing:
  - Normalize diagnostic source, target, message, hint, and span label.
  - Map backend diagnostic labels back into graph or Strategy IR targets.
  - Parse QuantScript diagnostic text into structured diagnostics.
  - Merge local and backend compile summaries.
  - Build compile failure summaries and artifact resolution summaries.
  - Re-export protocol mapping helpers for existing store callers.
- Outputs:
  - Normalized diagnostics.
  - Compile summary success/failure projections.
  - Strategy IR check and artifact resolution summaries.
  - Strategy IR/Core IR artifact mapping helpers.

## Recursive Child Queue

- `frontend.store.compile_flow.compile_helper_contract.diagnostic_helpers`
- `frontend.store.compile_flow.compile_helper_contract.summary_resolution_helpers`
- `frontend.store.compile_flow.compile_helper_contract.strategy_ir_protocol_mapping_bridge`

## Split Decision

- This leaf is worth recursive split.
- Hard-rule assessment:
  - `graphStoreCompileHelpers.js` exposes multiple public helper families with distinct invariants.
  - Diagnostics, summary/resolution, and protocol mapping have different callers and different failure modes.
  - Diagnostics helpers are reusable by summary failure helpers but should not be entangled with summary construction.
  - Protocol mapping already has a dedicated implementation file and can be closed as a bridge after summary/diagnostic extraction.
  - Each subleaf can be verified with targeted compile diagnostics, Strategy IR compile, and outcome projection tests.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
