# FE-0133 Frontend Store Editor Actions Baseline

Status: baseline established.

## Scope

- Parent node: `frontend.store`
- Active child parent: `frontend.store.editor_actions`
- This is a docs-only recursive baseline for the editor action group.

## Owned Files

- `frontend/src/store/graphStoreEditorActions.js`
- `frontend/src/store/graphStoreEditorSelectionActions.js`
- `frontend/src/store/graphStoreEditorSelectionActions.test.js`
- `frontend/src/store/graphStoreEditorDraftActions.js`
- `frontend/src/store/graphStoreEditorDraftActions.test.js`
- `frontend/src/store/graphStoreEditorTemplateActions.js`
- `frontend/src/store/graphStoreEditorTemplateActions.test.js`
- `frontend/src/store/graphStoreEditorNodeActions.js`
- `frontend/src/store/graphStoreEditorNodeActions.test.js`
- `frontend/src/store/graphStoreEditorEdgeActions.js`
- `frontend/src/store/graphStoreEditorEdgeActions.test.js`
- `frontend/src/store/graphStore.editorActions.test.js`

## Whitebox Boundary

- Inputs:
  - Current graph, registry, selections, source drafts, compile result, and runtime shell state from `get()`.
  - UI events from canvas, property panel, workspace issue navigation, event stream node focus, and template selection.
- Processing:
  - `createGraphStoreEditorActions` composes public editor store methods for the root store.
  - Selection actions own node/edge/compile diagnostic focus.
  - Draft actions own QuantScript/Formal QuantScript/Strategy IR draft mutation and graph-source application.
  - Template actions own strategy template graph replacement and related reset state.
  - Node actions own node creation, positioning, config/name mutation, and collapse state.
  - Edge actions own graph edge creation and selected node/edge removal.
- Outputs:
  - Updated graph, graph validation, selected node/edge state, compile diagnostic target, source drafts, compile result reset, recent node ids, and graph persistence.
- Parent communication:
  - Public methods are exposed through `graphStore.js`.
  - Editor subleaves must communicate through the `frontend.store.editor_actions` facade or the `frontend.store` parent, not by importing sibling store parents.

## Recursive Child Queue

- `frontend.store.editor_actions.facade_boundary`
- `frontend.store.editor_actions.selection_focus`
- `frontend.store.editor_actions.draft_source_actions`
- `frontend.store.editor_actions.template_loading_actions`
- `frontend.store.editor_actions.node_mutation_actions`
- `frontend.store.editor_actions.edge_mutation_actions`

## Split Decision

- This parent is worth recursive split.
- The facade currently composes editor, compile, and persistence actions together, so the first leaf should tighten the parent boundary and move sibling-parent composition back to `frontend.store`.
- The remaining action files already map to cohesive leaf groups with targeted tests.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
