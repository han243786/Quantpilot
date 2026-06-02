# FE-0167 Frontend Store Runtime Session Parent Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Closed child parent: `frontend.store.runtime_session`
- This is a docs-only parent closeout. No frontend source files changed in this step.

## Closed Leaves

- `frontend.store.runtime_session.simulation_stream_action`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0162-frontend-store-runtime-simulation-stream-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreRuntimeSimulationActions.js`
    - `frontend/src/store/graphStoreRuntimeSessionShared.js`
- `frontend.store.runtime_session.backtest_action`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0163-frontend-store-runtime-backtest-action-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreRuntimeBacktestActions.js`
    - `frontend/src/store/graphStoreRuntimeSessionShared.js`
- `frontend.store.runtime_session.v4_simulation_action`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0164-frontend-store-runtime-v4-simulation-action-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreRuntimeV4SimulationActions.js`
    - `frontend/src/store/graphStoreRuntimeSessionShared.js`
- `frontend.store.runtime_session.backtest_experiment_action`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0165-frontend-store-runtime-backtest-experiment-action-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreRuntimeBacktestExperimentActions.js`
    - `frontend/src/store/graphStoreRuntimeSessionShared.js`
- `frontend.store.runtime_session.lifecycle_stop_reset_actions`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0166-frontend-store-runtime-lifecycle-actions-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreRuntimeLifecycleActions.js`
    - `frontend/src/store/graphStoreRuntimeSessionState.js`

## Public Surfaces Preserved

- `frontend/src/store/graphStoreRuntimeSessionActions.js`
- `frontend/src/store/graphStoreRuntimeSimulationActions.js`
- `frontend/src/store/graphStoreRuntimeBacktestActions.js`
- `frontend/src/store/graphStoreRuntimeV4SimulationActions.js`
- `frontend/src/store/graphStoreRuntimeBacktestExperimentActions.js`
- `frontend/src/store/graphStoreRuntimeLifecycleActions.js`
- `frontend/src/store/graphStoreRuntimeSessionShared.js`
- `frontend/src/store/graphStoreRuntimeSessionState.js`

## Recursive Decision

- The runtime session child queue is closed.
- `graphStoreRuntimeSessionActions.js` now remains the parent composition boundary for simulation, backtest, v4 simulation, experiment sweep, and lifecycle leaves.
- Runtime session children communicate with lifecycle behavior through the public store surface `stopRuntime`.
- No child imports another runtime session child directly.
- The parent returns control to `frontend.store`.
- Next queued store child: `frontend.store.runtime_history`.

## Equivalence Evidence

- FE-0162 through FE-0166 each landed with targeted tests, build verification, and full frontend pre-commit verification.
- Source-changing runtime session leaves landed with build verification and full frontend pre-commit verification.
- This parent closeout only changes frontend-local governance files and records the already-verified child queue completion.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
