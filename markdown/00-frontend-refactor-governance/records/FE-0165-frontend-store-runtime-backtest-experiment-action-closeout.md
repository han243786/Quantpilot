# FE-0165 Frontend Store Runtime Backtest Experiment Action Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.runtime_session`
- Closed leaf: `frontend.store.runtime_session.backtest_experiment_action`
- Code surfaces:
  - `frontend/src/store/graphStoreRuntimeBacktestExperimentActions.js`
  - `frontend/src/store/graphStoreRuntimeSessionActions.js`
  - `frontend/src/store/graphStoreRuntimeSessionShared.js`
  - `frontend/src/pages/StrategyWorkspaceExperimentCard.test.jsx`
  - `frontend/src/store/graphStore.runtimeErrors.test.js`

## Change

- Extracted `startBacktestExperiment` into `graphStoreRuntimeBacktestExperimentActions.js`.
- Kept parameter-grid normalization, sweep request payload construction, experiment detail loading, and experiment error projection in one leaf.
- Kept `graphStoreRuntimeSessionActions.js` as the runtime session parent composer.

## Whitebox Boundary

- Inputs:
  - Current graph, graph runnability, capability state, current compile flow, parameter sweep options, backend sweep endpoint, and experiment detail loader.
- Processing:
  - Gate on graph runnability and `run_parameter_sweep` capability permission.
  - Compile current graph before creating the sweep request.
  - Normalize fee, slippage, and latency arrays into the backend parameter grid.
  - POST `/runtime/experiments/backtest-sweep`, then load the resulting experiment detail.
- Outputs:
  - Backend experiment response on success.
  - `selectedExperimentStatus` loading/error state.
  - Runtime backend error message on capability block or request failure.
- Parent communication:
  - `graphStoreRuntimeSessionActions.js` composes this leaf.
  - This leaf may call the public store surface `compileCurrentGraph` and `loadExperimentDetail`.
  - No runtime session child calls another runtime session child directly.

## Recursive Split Decision

- No further split is required now.
- The leaf has one public method and one backend request lifecycle.
- Parameter-grid normalization is small and only meaningful inside this request contract.
- Continue the parent queue through `frontend.store.runtime_session.lifecycle_stop_reset_actions`.

## Equivalence Baseline

- `startBacktestExperiment` still returns `null` on non-runnable graph, capability block, failed compile, or backend failure.
- Capability block still sets `selectedExperimentStatus` to `error` and records the backend error.
- Experiment requests still include experiment name, actor, capability context, runtime config, runtime targets, graph JSON, backtest options, and parameter grid.
- Successful request still calls `loadExperimentDetail(response.experiment_id)` and returns the response.
- Failed request still stores a humanized experiment error and returns `null`.

## Verification

- `npm.cmd test -- --run src/pages/StrategyWorkspaceExperimentCard.test.jsx src/store/graphStore.runtimeErrors.test.js`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
