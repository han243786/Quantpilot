# FE-0161 Frontend Store Runtime Session Baseline

Status: baseline established.

## Scope

- Parent node: `frontend.store`
- Active nested child parent: `frontend.store.runtime_session`
- This is a docs-only recursive baseline for runtime session action extraction.

## Owned Files

- `frontend/src/store/graphStoreRuntimeSessionActions.js`
- `frontend/src/store/graphStoreRuntimeSessionState.js`
- `frontend/src/store/graphStoreRuntimeTransport.js`
- `frontend/src/store/graphStore.runtimeErrors.test.js`
- `frontend/src/store/graphStore.runtimeActionLock.test.js`
- `frontend/src/pages/StrategyWorkspaceExperimentCard.jsx`
- `frontend/src/pages/StrategyWorkspaceExperimentCard.test.jsx`
- `frontend/src/components/TopToolbar.jsx`
- `frontend/src/components/TopToolbar.failureNotices.test.jsx`
- `frontend/src/components/TopToolbar.capabilities.test.jsx`

## Whitebox Boundary

- Inputs:
  - Current graph validation/runnability state.
  - Compile result or `compileCurrentGraph` output.
  - Capability status/source/message/capabilities.
  - Runtime transport event stream, runtime backend endpoints, runtime controller, and graph actor.
  - Formal QuantScript source/draft/override for v4 simulation.
  - Backtest experiment options and parameter grids.
- Processing:
  - Start simulation runtime, open SSE transport, batch runtime/account events, handle reconnect exhaustion, completion, and errors.
  - Start backtest runtime, optionally reuse cached compile result, request backend backtest, persist completion graph, and select backtest result.
  - Start v4 simulation runtime from Formal QuantScript source and project transient v4 output/events.
  - Start backtest experiment sweeps and load experiment detail.
  - Stop or reset active runtime controllers and runtime graph state.
- Outputs:
  - Runtime controller, runtime status, run kind, run id, account, events, timeline, diagnostics, governance, selected history/backtest ids, highlighted nodes, graph runtime binding, and action lock state.

## Recursive Child Queue

- `frontend.store.runtime_session.simulation_stream_action`
- `frontend.store.runtime_session.backtest_action`
- `frontend.store.runtime_session.v4_simulation_action`
- `frontend.store.runtime_session.backtest_experiment_action`
- `frontend.store.runtime_session.lifecycle_stop_reset_actions`

## Split Decision

- This leaf is worth recursive split.
- Hard-rule assessment:
  - The action file exposes multiple public methods with different endpoints and lifecycle invariants.
  - Simulation SSE has stream batching/reconnect/controller behavior that is distinct from backtest request completion.
  - v4 simulation has a separate source contract and transient output projection.
  - Experiment sweep has a separate parameter-grid and detail-loading contract.
  - Stop/reset lifecycle is shared by UI and graph lifecycle actions but should remain a small public lifecycle leaf.
  - Each subleaf can be verified with targeted runtime store and UI tests.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
