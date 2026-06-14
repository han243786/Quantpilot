# FE-0137 Frontend Store Editor Template Loading Actions Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.editor_actions`
- Closed leaf: `frontend.store.editor_actions.template_loading_actions`
- Primary files:
  - `frontend/src/store/graphStoreEditorTemplateActions.js`
  - `frontend/src/store/graphStoreEditorTemplateActions.test.js`
  - `frontend/src/store/graphStore.templates.test.js`
  - `frontend/src/hooks/useStrategyDirectoryModel.js`
  - `frontend/src/templates/strategyTemplates.js`

## Whitebox Boundary

- Inputs:
  - Template id.
  - Current registry.
  - Current runtime history, backtest history, and experiment lists.
- Processing:
  - `loadStrategyTemplate` builds a template graph, attaches validation, persists it, clears editor/compile focus, resets version preview/compare state, and resets active runtime focus.
- Outputs:
  - New validated graph.
  - Fresh graph-source and Strategy IR drafts.
  - Cleared editor selections, compile result, formal source override, version state, and active runtime context.
  - Preserved runtime history, backtest history, and experiments arrays.
- Parent communication:
  - Public method is exposed through `graphStore.js` via `createGraphStoreEditorActions`.

## Recursive Split Decision

- No further subleaf split is required.
- Hard-rule assessment:
  - The public behavior is one cohesive template-replacement workflow.
  - State reset details are part of the same atomic transition and should remain together for equivalence auditing.
  - Existing tests cover graph replacement, draft refresh, version reset, runtime active-state reset, and history preservation.
  - The leaf is small enough to audit directly.
- Next queued leaf: `frontend.store.editor_actions.node_mutation_actions`.

## Equivalence Baseline

- Loading a template returns a validated graph and persists it.
- Editor selection, diagnostic focus, compile result, formal draft override, graph version preview/compare, and active runtime focus are reset.
- Historical runtime/backtest/experiment collections are preserved.

## Verification

- `npm.cmd test -- --run src/store/graphStoreEditorTemplateActions.test.js src/store/graphStore.templates.test.js src/templates/strategyTemplates.test.js src/pages/StrategyHubTemplateLibrarySection.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
