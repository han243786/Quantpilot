# FE-0134 Frontend Store Editor Facade Boundary Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.editor_actions`
- Closed leaf: `frontend.store.editor_actions.facade_boundary`
- Code surfaces:
  - `frontend/src/store/graphStore.js`
  - `frontend/src/store/graphStoreEditorActions.js`

## Change

- Moved `createGraphStoreCompileActions` composition from `graphStoreEditorActions.js` to `graphStore.js`.
- Moved `createGraphStorePersistenceActions` composition from `graphStoreEditorActions.js` to `graphStore.js`.
- Kept `createGraphStoreEditorActions` scoped to editor-only subactions:
  - draft actions
  - selection actions
  - template actions
  - node actions
  - edge actions

## Whitebox Boundary

- Inputs:
  - Root store `set` and `get`.
  - Existing editor, compile, persistence, startup, and runtime action factories.
- Processing:
  - `graphStore.js` now owns sibling-parent composition for editor, compile, persistence, startup, and runtime actions.
  - `graphStoreEditorActions.js` now only composes editor child actions.
- Outputs:
  - Public `useGraphStore` action set remains equivalent.
  - Compile and persistence public methods remain available through `useGraphStore`.

## Recursive Split Decision

- No further split is needed inside this facade leaf.
- The leaf exists to enforce the parent-child communication rule: sibling parent actions are composed by `frontend.store`, not by `frontend.store.editor_actions`.
- Next queued leaf: `frontend.store.editor_actions.selection_focus`.

## Equivalence Baseline

- Editor actions remain available from `useGraphStore`.
- Compile flow actions remain available from `useGraphStore`.
- Persistence actions remain available from `useGraphStore`.
- Object spread order remains compatible with the previous public action order: editor actions first, then compile actions, then persistence actions.

## Verification

- `npm.cmd test -- --run src/store/graphStore.editorActions.test.js src/store/graphStoreEditorSelectionActions.test.js src/store/graphStoreEditorDraftActions.test.js src/store/graphStoreEditorTemplateActions.test.js src/store/graphStoreEditorNodeActions.test.js src/store/graphStoreEditorEdgeActions.test.js src/store/graphStore.strategyIrCompile.test.js src/store/graphStorePersistenceConsistency.test.js src/store/graphStore.saveGraphRollback.test.js`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
