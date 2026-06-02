# FE-0162 Frontend Store Runtime Simulation Stream Action Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.runtime_session`
- Closed leaf: `frontend.store.runtime_session.simulation_stream_action`
- Code surfaces:
  - `frontend/src/store/graphStoreRuntimeSimulationActions.js`
  - `frontend/src/store/graphStoreRuntimeSessionShared.js`
  - `frontend/src/store/graphStoreRuntimeSessionActions.js`
  - `frontend/src/store/graphStore.runtimeErrors.test.js`
  - `frontend/src/store/graphStore.runtimeActionLock.test.js`
  - `frontend/src/components/TopToolbar.failureNotices.test.jsx`

## Change

- Extracted `startRuntime` into `graphStoreRuntimeSimulationActions.js`.
- Extracted shared runtime capability-block helpers into `graphStoreRuntimeSessionShared.js`.
- Kept `graphStoreRuntimeSessionActions.js` as the runtime session parent composer for simulation, backtest, v4 simulation, experiment, and lifecycle leaves.

## Whitebox Boundary

- Inputs:
  - Current graph, graph runnability, action lock, capability state, compiled runtime result, runtime backend endpoint, runtime event source, and runtime controller.
- Processing:
  - Gate on action lock, graph runnability, and capability permission boundary.
  - Compile current graph before simulation start.
  - Stop existing runtime before opening a new runtime event stream.
  - Request `/runtime/test-run`, bind runtime run id to graph metadata, create SSE source, batch runtime/account events, handle completion, reconnect state, reconnect exhaustion, and controller close cleanup.
- Outputs:
  - Runtime controller and simulation runtime state.
  - Runtime event/account projections.
  - Graph runtime binding and runtime node state.
  - Cleared action lock after success or failure.
- Parent communication:
  - `graphStoreRuntimeSessionActions.js` composes this leaf.
  - Other runtime session leaves reuse `graphStoreRuntimeSessionShared.js` for capability-block behavior.
  - No runtime session child calls another runtime session child directly except through the public store surface (`stopRuntime`).

## Recursive Split Decision

- No further split is required now.
- The simulation stream leaf is cohesive around one public method and one SSE lifecycle.
- Stream batching and controller cleanup remain inside the same leaf because splitting them would create tight hidden coupling.
- Continue the parent queue through `frontend.store.runtime_session.backtest_action`.

## Equivalence Baseline

- `startRuntime` still returns early on action lock, non-runnable graph, failed compile, or capability block.
- Simulation still posts to `/runtime/test-run`, opens the runtime event stream, batches runtime/account events, handles completion/reconnect/error, and clears `actionLock`.
- Runtime controller close still clears reconnect timers, batch timer, batched events, and active source.
- Capability-blocked simulation still updates runtime error state and runtime node error state.

## Verification

- `npm.cmd test -- --run src/store/graphStore.runtimeErrors.test.js src/store/graphStore.runtimeActionLock.test.js src/components/TopToolbar.failureNotices.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
