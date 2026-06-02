# FE-0156 Frontend Store Compile Diagnostic Helpers Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.compile_flow.compile_helper_contract`
- Closed leaf: `frontend.store.compile_flow.compile_helper_contract.diagnostic_helpers`
- Code surfaces:
  - `frontend/src/store/graphStoreCompileDiagnostics.js`
  - `frontend/src/store/graphStoreCompileHelpers.js`
  - `frontend/src/store/graphStore.diagnostics.test.js`
  - `frontend/src/store/graphStore.strategyIrCompile.test.js`

## Change

- Extracted compile diagnostic target mapping, source normalization, target normalization, QuantScript diagnostic text parsing, and backend error diagnostic normalization into `graphStoreCompileDiagnostics.js`.
- Kept `graphStoreCompileHelpers.js` as the compatibility parent for existing imports and re-exports.

## Whitebox Boundary

- Inputs:
  - Backend compile diagnostics, backend error details, error messages, graph artifacts, QuantScript labels, Strategy IR labels, and optional compile source.
- Processing:
  - Normalize diagnostic source into graph, Strategy IR, Formal QuantScript, or runtime domains.
  - Resolve string and span-label targets back into graph nodes/edges/fields or Strategy IR search terms.
  - Parse QuantScript diagnostic lines into structured diagnostics.
  - Convert backend error details into normalized diagnostics with sanitized message and hint fields.
- Outputs:
  - Normalized diagnostic objects.
  - Structured diagnostics derived from backend error details or QuantScript messages.
  - Public helper exports preserved through `graphStoreCompileHelpers.js` and `graphStoreHelpers.js`.
- Parent communication:
  - Summary helpers call diagnostics helpers through `graphStoreCompileDiagnostics.js`.
  - Existing callers can continue importing from `graphStoreHelpers.js`.

## Recursive Split Decision

- No further split is required now.
- Diagnostic helpers are a compact, deterministic utility family with one directional dependency on protocol label mappings.
- Continue the parent queue through `frontend.store.compile_flow.compile_helper_contract.summary_resolution_helpers`.

## Equivalence Baseline

- Graph/QuantScript span labels still map to graph targets.
- Strategy IR span labels still produce Strategy IR search terms when a Strategy IR artifact exists.
- Backend error details still produce normalized diagnostics.
- QuantScript diagnostic text still parses `QS####`/`Q*` messages.
- Unknown diagnostic sources still fall back to `graph`.

## Verification

- `npm.cmd test -- --run src/store/graphStore.diagnostics.test.js src/store/graphStore.strategyIrCompile.test.js`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
