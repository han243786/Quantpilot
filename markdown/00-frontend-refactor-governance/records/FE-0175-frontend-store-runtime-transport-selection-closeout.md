# FE-0175 Frontend Store Runtime Transport Selection Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Closed leaf: `frontend.store.runtime_transport_selection`
- Code surfaces:
  - `frontend/src/store/graphStoreRuntimeTransport.js`
  - `frontend/src/store/graphStoreRuntimeTransport.test.js`
  - `frontend/src/store/graphStoreRuntimeSimulationActions.js`
  - `frontend/src/store/graphStore.runtimeErrors.test.js`

## Whitebox Boundary

- Inputs:
  - Runtime run id, API base URL, browser `EventSource`, reconnect callback, and browser online events.
- Processing:
  - Build the runtime event stream URL.
  - Create the `EventSource` instance.
  - Track manual close state.
  - Schedule exponential reconnects with a maximum delay.
  - Forward runtime/account/completion/error handlers to the reconnected source.
- Outputs:
  - Runtime event stream URL.
  - Runtime `EventSource` instance with close/reconnect helpers.
  - Reconnected event source passed to `onReconnect`.

## Parent Communication

- `graphStoreRuntimeSimulationActions.js` imports this leaf to open simulation SSE streams.
- Runtime history/session children do not call this leaf directly unless they own a streaming transport flow.
- This leaf must not mutate graph store state.

## Recursive Split Decision

- No further split is required now.
- The leaf has one transport protocol and one test file.
- URL construction and reconnect behavior are tightly coupled to the same SSE transport.
- The unused retry-exhaustion callback remains part of the compatibility signature because the current protocol intentionally reconnects indefinitely.

## Equivalence Baseline

- Event stream URLs still use `${API_BASE}/runtime/runs/{runId}/events`.
- Manual close still prevents reconnect.
- Reconnect still forwards runtime, account, completion, and error handlers.
- Reconnect remains indefinite with exponential backoff capped at 60 seconds.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
