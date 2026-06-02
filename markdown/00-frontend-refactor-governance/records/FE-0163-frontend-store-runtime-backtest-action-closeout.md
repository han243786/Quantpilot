# FE-0163 Frontend Store Runtime Backtest Action Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.runtime_session`
- Closed leaf: `frontend.store.runtime_session.backtest_action`
- Code surfaces:
  - `frontend/src/store/graphStoreRuntimeBacktestActions.js`
  - `frontend/src/store/graphStoreRuntimeSessionActions.js`
  - `frontend/src/store/graphStoreRuntimeSessionShared.js`
  - `frontend/src/store/graphStore.runtimeErrors.test.js`
  - `frontend/src/store/graphStore.runtimeActionLock.test.js`
  - `frontend/src/store/graphStore.backtestArtifacts.test.js`
  - `frontend/src/components/TopToolbar.failureNotices.test.jsx`

## Change

- Extracted `startBacktest` into `graphStoreRuntimeBacktestActions.js`.
- Kept `graphStoreRuntimeSessionActions.js` as the runtime session parent composer for simulation, backtest, v4 simulation, experiment, and lifecycle leaves.
- Reused `graphStoreRuntimeSessionShared.js` for the runtime capability-block path.

## Whitebox Boundary

- Inputs:
  - Current graph, graph runnability, action lock, capability state, compile result cache, current compile flow, backend backtest endpoint, and runtime target projection.
- Processing:
  - Gate on action lock, graph runnability, and capability permission boundary.
  - Reuse the latest compile result when the graph compile id still matches.
  - Compile current graph before backtest when the compile cache is stale.
  - Stop the active runtime before entering the backtest request flow.
  - POST `/runtime/backtest`, build the backtest completion state, persist the next graph, and project runtime selection state.
- Outputs:
  - Updated graph metadata/artifacts.
  - Selected node id and backtest runtime state.
  - Runtime node error projection on failure.
  - Cleared action lock after success or failure.
- Parent communication:
  - `graphStoreRuntimeSessionActions.js` composes this leaf.
  - This leaf may call the public parent store surface `stopRuntime`.
  - No runtime session child calls another runtime session child directly.

## Recursive Split Decision

- No further split is required now.
- The backtest leaf has one public method and one backend request lifecycle.
- Compile cache reuse, request payload construction, completion projection, and error projection are tightly coupled to `startBacktest`; separating them would add cross-leaf protocol overhead without improving ownership.
- Continue the parent queue through `frontend.store.runtime_session.v4_simulation_action`.

## Equivalence Baseline

- `startBacktest` still returns early on action lock, non-runnable graph, failed compile, or capability block.
- Backtest still reuses the compile cache when compile ids match.
- Backtest still posts to `/runtime/backtest` with actor, capability context, runtime config, runtime targets, graph JSON, replay options, runtime kind, and symbols.
- Successful backtest still applies `buildBacktestCompletionState`, persists the next graph, updates selected node id, and projects runtime state.
- Failed backtest still records a humanized backend error, clears transient backtest artifacts/diagnostics/governance, updates the runtime node to error, and clears `actionLock`.

## Verification

- `npm.cmd test -- --run src/store/graphStore.runtimeErrors.test.js src/store/graphStore.runtimeActionLock.test.js src/store/graphStore.backtestArtifacts.test.js src/components/TopToolbar.failureNotices.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
