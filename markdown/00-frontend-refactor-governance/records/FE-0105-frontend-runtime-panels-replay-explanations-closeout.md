# FE-0105 Frontend Runtime Panels Replay Explanations Closeout

Status: closed.

## Child Node

`frontend.runtime_panels.replay_and_explanations`

## Boundary

This leaf owns persisted event replay paging and runtime explanation helpers used by replay rows, event stream explanation rows, and history explanation cards. It covers source readiness, run/backtest replay loading, sequence and legacy cursor paging, load failure surfacing, event explanation summary selection, and diagnostics explanation fallback.

## Changed Files

- `frontend/src/components/EventReplaySection.test.jsx`
- `frontend/src/utils/runtimeExplanation.test.js`
- `markdown/00-frontend-refactor-governance/frontend-module-tree.md`
- `markdown/00-frontend-refactor-governance/frontend-full-feature-tree.md`
- `markdown/00-frontend-refactor-governance/frontend-recursive-state.json`
- `markdown/00-frontend-refactor-governance/records/FE-0105-frontend-runtime-panels-replay-explanations-closeout.md`

## Public Surface

- `EventReplaySection`
- `getEventExplanationSummary`
- `buildDiagnosticsExplanationEntries`
- `HistoryExplanationCard`

## Preserved Behavior

- Event replay still stays dormant until a persisted run or backtest source is selected.
- Backtest and run replay windows still load on demand and render events, checkpoints, account summary, and execution explanations.
- Sequence-cursor paging remains preferred, while legacy cursor-only pagination still works.
- Replay load failures still surface in the panel without rendering stale event rows.
- Explanation helpers still prefer structured explanation summaries, fall back to legacy `reason_text`, and suppress duplicate summaries.
- Diagnostics explanation entries still use graph node names when available and fall back to node ids when the graph is incomplete.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; replay and explanation helpers are shared by event stream, run history, backtest history, and detail surfaces.
- `leaf_split_positive_trigger`: `testability_gain` and `blast_radius_reduction`.
- `leaf_split_stop_condition`: reached; replay paging and explanation helper behavior are covered for this pass.
- `leaf_split_decision_result`: close `frontend.runtime_panels.replay_and_explanations` and proceed to the `frontend.runtime_panels` parent closeout.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/components/EventReplaySection.test.jsx src/utils/runtimeExplanation.test.js src/components/EventStreamPanel.executionExplanation.test.jsx src/components/EventStreamPanel.historyExplanation.test.jsx`: passed, 4 files / 12 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Step

Close the parent node `frontend.runtime_panels`.
