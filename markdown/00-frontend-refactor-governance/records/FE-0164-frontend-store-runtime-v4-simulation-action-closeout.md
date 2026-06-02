# FE-0164 Frontend Store Runtime V4 Simulation Action Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.runtime_session`
- Closed leaf: `frontend.store.runtime_session.v4_simulation_action`
- Code surfaces:
  - `frontend/src/store/graphStoreRuntimeV4SimulationActions.js`
  - `frontend/src/store/graphStoreRuntimeSessionActions.js`
  - `frontend/src/store/graphStoreRuntimeSessionShared.js`
  - `frontend/src/store/graphStore.runtimeErrors.test.js`
  - `frontend/src/store/graphStore.runtimeActionLock.test.js`
  - `frontend/src/components/TopToolbar.formalSourceMode.test.jsx`
  - `frontend/src/components/TopToolbar.failureNotices.test.jsx`

## Change

- Extracted `startV4Simulation` into `graphStoreRuntimeV4SimulationActions.js`.
- Moved v4 QuantScript source resolution, source shape validation, and runtime output event mapping into the same leaf.
- Kept `graphStoreRuntimeSessionActions.js` as the runtime session parent composer.

## Whitebox Boundary

- Inputs:
  - Action lock, capability state, formal QuantScript override, graph QuantScript artifact, formal source draft, and `/runtime/v4/run` backend response.
- Processing:
  - Gate on action lock and `start_v4_simulation` capability permission.
  - Resolve the runnable v4 QuantScript source from override, graph artifact, or draft.
  - Validate the source has a `v4_strategy` declaration before calling the backend.
  - Stop active runtime, mark connecting state, POST `/runtime/v4/run`, map output events, and project v4 runtime artifacts.
- Outputs:
  - Completed v4 runtime state, diagnostics, events, highlighted node ids, memory snapshot, output payload, and handoff payload.
  - Runtime node error projection on backend failure.
  - Cleared action lock after success or failure.
- Parent communication:
  - `graphStoreRuntimeSessionActions.js` composes this leaf.
  - This leaf may call the public parent store surface `stopRuntime`.
  - No runtime session child calls another runtime session child directly.

## Recursive Split Decision

- No further split is required now.
- The v4 simulation leaf has one public method and small private helpers dedicated to that method.
- Splitting source resolution or event mapping would create helper leaves with no independent public contract.
- Continue the parent queue through `frontend.store.runtime_session.backtest_experiment_action`.

## Equivalence Baseline

- `startV4Simulation` still returns early on action lock or capability block.
- The source priority remains formal override, graph QuantScript artifact, then formal source draft.
- Invalid v4 source still sets a v4 simulation capability-blocked runtime error.
- Successful v4 simulation still posts to `/runtime/v4/run`, maps output events in reverse order, projects diagnostics and v4 runtime artifacts, and clears `actionLock`.
- Failed v4 simulation still records a humanized backend error, updates the runtime node to error, and clears `actionLock`.

## Verification

- `npm.cmd test -- --run src/store/graphStore.runtimeErrors.test.js src/store/graphStore.runtimeActionLock.test.js src/components/TopToolbar.formalSourceMode.test.jsx src/components/TopToolbar.failureNotices.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
