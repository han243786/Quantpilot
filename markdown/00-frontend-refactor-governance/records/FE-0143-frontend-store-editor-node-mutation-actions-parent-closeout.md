# FE-0143 Frontend Store Editor Node Mutation Actions Parent Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.editor_actions`
- Closed nested child parent: `frontend.store.editor_actions.node_mutation_actions`
- This is a docs-only parent closeout. No frontend source files changed in this step.

## Closed Leaves

- `frontend.store.editor_actions.node_mutation_actions.node_creation`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0139-frontend-store-editor-node-creation-closeout.md`
  - Public surface: `frontend/src/store/graphStoreEditorNodeCreationActions.js`
- `frontend.store.editor_actions.node_mutation_actions.node_position_viewport`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0140-frontend-store-editor-node-position-viewport-closeout.md`
  - Public surface: `frontend/src/store/graphStoreEditorNodePositionActions.js`
- `frontend.store.editor_actions.node_mutation_actions.node_config_identity`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0141-frontend-store-editor-node-config-identity-closeout.md`
  - Public surface: `frontend/src/store/graphStoreEditorNodeConfigActions.js`
- `frontend.store.editor_actions.node_mutation_actions.node_ui_collapse`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0142-frontend-store-editor-node-ui-collapse-closeout.md`
  - Public surface: `frontend/src/store/graphStoreEditorNodeUiActions.js`

## Public Surfaces Preserved

- `frontend/src/store/graphStoreEditorNodeActions.js`
- `frontend/src/store/graphStoreEditorNodeCreationActions.js`
- `frontend/src/store/graphStoreEditorNodePositionActions.js`
- `frontend/src/store/graphStoreEditorNodeConfigActions.js`
- `frontend/src/store/graphStoreEditorNodeUiActions.js`

## Recursive Decision

- The nested subchild queue is closed.
- The node mutation parent now composes four white-box leaves: creation, position/viewport, config/identity, and UI collapse.
- Each leaf owns distinct public actions and no child calls a sibling child directly.
- The parent returns control to `frontend.store.editor_actions`.
- Next queued child: `frontend.store.editor_actions.edge_mutation_actions`.

## Equivalence Evidence

- FE-0139 through FE-0142 each landed with targeted store/UI tests and full pre-commit verification.
- This parent closeout only changes frontend-local governance files and records the already-verified nested child queue completion.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
