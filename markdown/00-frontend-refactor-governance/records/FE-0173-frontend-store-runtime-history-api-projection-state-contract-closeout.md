# FE-0173 Frontend Store Runtime History API Projection State Contract Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.runtime_history`
- Closed leaf: `frontend.store.runtime_history.api_projection_state_contract`
- Code surfaces:
  - `frontend/src/store/graphStoreRuntimeHistoryApi.js`
  - `frontend/src/store/graphStoreRuntimeHistoryProjection.js`
  - `frontend/src/store/graphStoreRuntimeHistoryState.js`
  - `frontend/src/store/graphStoreRuntimeHistoryFailure.js`
  - `frontend/src/store/graphStoreRuntimeHistoryFlow.js`
  - `frontend/src/store/graphStoreRuntimeHistoryActions.js`
  - `frontend/src/store/graphStoreRuntimeHistoryFlow.test.js`

## Contract

- `graphStoreRuntimeHistoryApi.js`
  - Owns HTTP endpoints for runtime history lists, detail records, replay/report/mutation APIs, and save/discard side effects.
  - Uses `fetchJson`, `postJson`, `deleteJson`, and `unwrapPage` from persistence helpers.
  - Exposes API-level primitives only; it must not mutate graph store state.
- `graphStoreRuntimeHistoryProjection.js`
  - Owns graph projection for loaded run/backtest detail.
  - Converts detail events into highlighted nodes, node runtime overlays, and runtime binding metadata.
  - It must not call APIs or store setters.
- `graphStoreRuntimeHistoryState.js`
  - Owns pure runtime history state projection helpers.
  - Projects list loading/ready/error state, selected experiment state, selected run/backtest persisted runtime state, and shared backend error state.
  - It may call compare-selection helpers and persisted runtime selection helpers, but must not call APIs.
- `graphStoreRuntimeHistoryFailure.js`
  - Owns runtime-history failure text and backend reason preservation.
  - It is shared by refresh, detail, and artifact leaves.
- `graphStoreRuntimeHistoryFlow.js`
  - Is now a parent facade only.
  - It re-exports refresh, detail, artifact, and failure surfaces to preserve existing imports.

## Whitebox Boundary

- Inputs:
  - Runtime API requests/responses, graph detail payloads, runtime state snapshots, compare selection state, and shared persistence helpers.
- Processing:
  - Keep API transport, graph projection, state projection, and flow composition separated.
  - Let child flow leaves compose these helpers through explicit imports.
  - Preserve old public import path compatibility through the facade.
- Outputs:
  - Stable helper surfaces for runtime history children.
  - Closed runtime history helper contract ready for parent closeout.

## Recursive Split Decision

- No further split is required now.
- API, projection, state, failure, and facade files are already single-purpose helper contracts.
- Additional recursive leaves here would mostly split small pure helper groups without reducing coupling.
- Runtime history child queue is now complete and ready for parent closeout.

## Equivalence Baseline

- Runtime history API paths remain unchanged.
- Run/backtest graph projection semantics remain unchanged.
- Runtime history list/detail/error state projection semantics remain unchanged.
- Failure messages still preserve backend reasons through `buildActionFailureMessage`.
- Existing imports from `graphStoreRuntimeHistoryFlow.js` remain supported.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
