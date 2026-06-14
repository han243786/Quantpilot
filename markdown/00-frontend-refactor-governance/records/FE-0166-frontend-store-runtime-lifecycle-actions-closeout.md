# FE-0166 Frontend Store Runtime Lifecycle Actions Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.runtime_session`
- Closed leaf: `frontend.store.runtime_session.lifecycle_stop_reset_actions`
- Code surfaces:
  - `frontend/src/store/graphStoreRuntimeLifecycleActions.js`
  - `frontend/src/store/graphStoreRuntimeSessionActions.js`
  - `frontend/src/store/graphStoreRuntimeSessionState.js`
  - `frontend/src/store/graphStore.runtimeActionLock.test.js`
  - `frontend/src/components/TopToolbar.capabilities.test.jsx`

## Change

- Extracted `stopRuntime` and `resetRuntime` into `graphStoreRuntimeLifecycleActions.js`.
- Kept controller closing and runtime stopped/reset state projection in one lifecycle leaf.
- Reduced `graphStoreRuntimeSessionActions.js` to a parent composer for all runtime session leaves.

## Whitebox Boundary

- Inputs:
  - Runtime controller and current runtime state.
- Processing:
  - Close the runtime controller through `closeController`.
  - Project stopped state with the `"Runtime stopped."` message.
  - Project reset state when reset is requested.
- Outputs:
  - Runtime stopped or reset state.
  - Closed controller side effects.
- Parent communication:
  - `graphStoreRuntimeSessionActions.js` composes this leaf.
  - Other runtime session leaves call lifecycle behavior only through the public store surface `stopRuntime`.
  - No runtime session child imports this leaf directly.

## Recursive Split Decision

- No further split is required now.
- Stop and reset share the same controller-close lifecycle boundary and are both small state projections.
- Splitting them would create two tiny leaves with no meaningful independent protocol.
- Runtime session subchild queue is now closed and ready for parent closeout.

## Equivalence Baseline

- `stopRuntime` still closes the current runtime controller and applies `buildRuntimeStoppedState(state, "Runtime stopped.")`.
- `resetRuntime` still closes the current runtime controller and applies `buildRuntimeResetState(state)`.
- Store public method names remain unchanged.
- Runtime simulation, backtest, and v4 simulation leaves can still call `get().stopRuntime()`.

## Verification

- `npm.cmd test -- --run src/store/graphStore.runtimeActionLock.test.js src/components/TopToolbar.capabilities.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
