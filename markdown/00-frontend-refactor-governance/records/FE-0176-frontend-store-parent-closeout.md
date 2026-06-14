# FE-0176 Frontend Store Parent Closeout

Status: closed.

## Parent Node

`frontend.store`

## Closed Children

- `frontend.store.root_shell`
- `frontend.store.persistence_startup`
- `frontend.store.capability_refresh`
- `frontend.store.editor_actions`
- `frontend.store.compile_flow`
- `frontend.store.runtime_session`
- `frontend.store.runtime_history`
- `frontend.store.runtime_transport_selection`

## Final Parent Boundary

`frontend.store` now owns root Zustand store assembly, graph persistence/startup, capability refresh, editor actions, compile flow, runtime session actions, runtime history actions, runtime transport, and their helper contracts.

Application shell, routing, page/view components, runtime panel UI, backtest views, design-system styles, and backend APIs remain outside this parent.

## Whitebox Contract

### Public Inputs

- Store `set`/`get` accessors and root state initialization.
- Graph persistence transport, storage, shape validation, and collaboration helpers.
- Editor node/edge/draft/template commands.
- Compile API responses and compile outcome projections.
- Runtime session commands, runtime history APIs, and runtime SSE transport.

### Public Outputs

- Public `useGraphStore` state and actions.
- Startup and persistence side effects.
- Editor mutation actions.
- Compile source/export/current graph actions.
- Runtime session, runtime history, and runtime transport behavior.

## Preserved Behavior

- Store public method names remain stable through parent composers and facades.
- Runtime session children communicate through the parent store surface.
- Runtime history children are split into compare, refresh, detail, artifact, and helper contracts while preserving `graphStoreRuntimeHistoryFlow.js` imports.
- Compile and editor child parents are already closed with explicit white-box records.
- No child-to-child shortcut was introduced outside the documented public surfaces.

## Further-Split Decision

No further split is useful inside `frontend.store` now. All planned store child leaves and child parents are closed. Future store splitting should require a concrete new store feature, a new persistence/runtime contract, or a developer-directed release transition.

## Verification

- FE-0122 through FE-0175 closed every store child leaf or child parent with targeted tests, build verification, docs gates, or full frontend pre-commit verification according to each step's risk.
- This parent closeout only changes frontend-local governance files.
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`

## Next Parent Candidate

`frontend.design_system_styles`
