# FE-0158 Frontend Store Compile Protocol Mapping Bridge Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.compile_flow.compile_helper_contract`
- Closed leaf: `frontend.store.compile_flow.compile_helper_contract.strategy_ir_protocol_mapping_bridge`
- Code surfaces:
  - `frontend/src/store/graphStoreCompileProtocolMapping.js`
  - `frontend/src/store/graphStoreCompileHelpers.js`
  - `frontend/src/store/graphStore.strategyIrDraft.test.js`
  - `frontend/src/store/graphStore.strategyIrCompile.test.js`
  - `frontend/src/store/graphStoreRootState.test.js`
  - `frontend/src/store/graphStore.diagnostics.test.js`

## Change

- Closed the Strategy IR protocol mapping bridge as an already-isolated leaf.
- Kept `graphStoreCompileProtocolMapping.js` as the implementation surface for Strategy IR/Core IR artifact mapping, JSON parse/stringify, draft/source resolution, and label target mapping.
- Kept `graphStoreCompileHelpers.js` as the compatibility re-export parent for existing callers.

## Whitebox Boundary

- Inputs:
  - Graph metadata artifacts, Strategy IR artifact variants, Core IR artifacts, Strategy IR JSON/source strings, QuantScript label targets, and Strategy IR documents.
- Processing:
  - Attach Core IR artifacts to graph metadata.
  - Parse and stringify JSON safely.
  - Resolve Strategy IR artifacts into document, draft, and compile-source forms.
  - Build Strategy IR label targets for metadata, execution, risk rules, data requirements, and signals.
  - Merge generated and explicit Strategy IR label targets.
  - Expose QuantScript label targets for diagnostic mapping.
- Outputs:
  - Graphs with Core IR artifacts attached.
  - Strategy IR document/source/draft values.
  - Strategy IR and QuantScript label target maps.
  - Public helper exports preserved through `graphStoreCompileHelpers.js` and `graphStoreHelpers.js`.

## Recursive Split Decision

- No further split is required now.
- The implementation is a cohesive pure mapping surface with no store mutation, no backend calls, and no sibling-to-sibling writes.
- Splitting it further would increase bridge files without reducing runtime or state coupling.
- The nested compile helper child queue is now closed and ready for parent closeout.

## Equivalence Baseline

- Strategy IR artifact variants still resolve from string, source, document, strategy_ir, or `ir_version` object forms.
- Strategy IR drafts still prefer artifact source and fall back to stringified document.
- Strategy IR compile source still preserves object/source/string variants.
- Label target maps still combine generated Strategy IR targets with explicit artifact targets.
- QuantScript label targets still come from graph metadata artifacts.

## Verification

- `npm.cmd test -- --run src/store/graphStore.strategyIrDraft.test.js src/store/graphStore.strategyIrCompile.test.js src/store/graphStoreRootState.test.js src/store/graphStore.diagnostics.test.js`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
