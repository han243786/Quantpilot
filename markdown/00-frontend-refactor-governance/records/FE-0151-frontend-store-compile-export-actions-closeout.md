# FE-0151 Frontend Store Compile Export Actions Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.compile_flow`
- Closed leaf: `frontend.store.compile_flow.export_actions`
- Code surfaces:
  - `frontend/src/store/graphStoreCompileExportActions.js`
  - `frontend/src/store/graphStoreCompileActions.js`
  - `frontend/src/store/graphStore.export.test.js`
  - `frontend/src/components/TopToolbar.exportFailure.test.jsx`

## Change

- Extracted `exportRuntimeConfig` and `exportQuantScript` into `graphStoreCompileExportActions.js`.
- Kept `graphStoreCompileActions.js` as the compile-flow parent composer while current graph compile remains queued.

## Whitebox Boundary

- Inputs:
  - Current graph, registry, strategy IR draft, compile result, and `compileCurrentGraph`.
- Processing:
  - Runtime config export reuses backend-verified compile output when available and falls back to current compile state otherwise.
  - QuantScript export performs local graph compile, attaches validation, stores compile result, refreshes graph source draft, and resolves Strategy IR draft.
- Outputs:
  - Runtime config export payload or fallback payload.
  - QuantScript source text.
  - Updated graph, compile result, graph-source draft, and Strategy IR draft for QuantScript export.
- Parent communication:
  - `graphStoreCompileActions.js` composes this leaf.
  - `graphStore.js` exposes export actions through `createGraphStoreCompileActions`.

## Recursive Split Decision

- No further split is required now.
- The two public export methods form one compact export boundary and are covered by export/store and toolbar failure tests.
- Continue the parent queue through `frontend.store.compile_flow.current_graph_compile_action`.

## Equivalence Baseline

- Runtime config export still delegates to `compileCurrentGraph` and falls back when compile returns null.
- QuantScript export still locally compiles the graph, updates compile state, refreshes drafts, and returns generated QuantScript.

## Verification

- `npm.cmd test -- --run src/store/graphStore.export.test.js src/store/graphStore.strategyIrCompile.test.js src/components/TopToolbar.exportFailure.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
