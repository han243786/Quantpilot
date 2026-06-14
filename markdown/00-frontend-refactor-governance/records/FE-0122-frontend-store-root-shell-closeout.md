# FE-0122 - Frontend Store Root Shell Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Closed child leaf: `frontend.store.root_shell`
- Public surface:
  - `frontend/src/store/graphStore.js`
  - `frontend/src/store/graphStoreRootState.js`

## Extraction

- Added `frontend/src/store/graphStoreRootState.js`.
- Added `frontend/src/store/graphStoreRootState.test.js`.
- Updated `frontend/src/store/graphStore.js` to consume `createInitialGraphStoreState()`.

## Whitebox Contract

- Inputs:
  - Default capability and registry snapshots from `graphStoreHelpers`.
  - Fallback graph factory and strategy IR draft resolver.
- Outputs:
  - Fresh initial runtime state.
  - Initial graph store shell state for the Zustand root.
  - Stable initial registry, capability status, graph index status, graph state, compile state, runtime controller, source drafts, and strategy IR draft.
- Parent communication:
  - `graphStore.js` remains the Zustand root and action aggregation owner.
  - The root-state leaf owns initial state projection only.
  - Startup recovery, graph index refresh, capability refresh, editor actions, compile actions, and runtime actions remain queued in later store leaves.

## Preserved Behavior

- Existing store startup recovery, editor action, and export tests remain green.
- Initial runtime nested arrays and objects are created fresh.
- The initial strategy IR draft still derives from the same fallback graph that the root state exposes.

## Further Split Decision

- `frontend.store.root_shell` is closed for the initial-state extraction.
- Remaining root-shell-adjacent startup and persistence responsibilities are intentionally queued under `frontend.store.persistence_startup`.

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/store/graphStoreRootState.test.js src/store/graphStore.startupRecovery.test.js src/store/graphStore.editorActions.test.js src/store/graphStore.export.test.js`
  - Result: passed, 4 files / 11 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
