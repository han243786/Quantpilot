# FE-0095 Frontend Graph Editor Legacy Editor Page Shell Closeout

Status: closed.

## Child Node

`frontend.graph_editor.legacy_editor_page_shell`

## Boundary

This leaf owns the legacy `EditorPage` route shell that composes the graph editor workspace surfaces and lazily mounts the event stream panel. It remains a compatibility shell while route-owned strategy workspace surfaces continue to own the primary editing flow.

## Changed Files

- `frontend/src/pages/EditorPage.test.jsx`

## Public Surface

- `EditorPage`
- Event stream `onOpenBacktestDetail` bridge to `backtestDetailPath` and `navigateTo`.

## Preserved Behavior

- The page still renders `TopToolbar`, `ModuleSidebar`, `StrategyCanvas`, and `PropertyPanel`.
- The event stream panel still mounts after the deferred first-screen scheduling path.
- Backtest detail opening still carries the current graph id into the route helper and dispatches navigation through the router boundary.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: evaluated; the leaf is a compact compatibility shell and does not contain enough independent behavior to justify more children now.
- `leaf_split_positive_trigger`: `testability_gain` only; adding a white-box equivalent baseline closes the previous untested shell risk.
- `leaf_split_stop_condition`: reached for `legacy_editor_page_shell`; no deeper split now.
- `leaf_split_decision_result`: all planned `frontend.graph_editor` children are now closed. Perform parent closeout next.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/pages/EditorPage.test.jsx`: passed, 1 file / 2 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Step

Parent closeout for `frontend.graph_editor`.
