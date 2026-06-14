# FE-0100 Frontend Runtime Panels History Sections Closeout

Status: closed.

## Child Node

`frontend.runtime_panels.history_sections`

## Boundary

This leaf owns the standalone run and backtest history sections that were split out of the runtime event shell. It covers history filtering controls, refresh actions, result rows, detail routing/loading, compare queue controls, and pagination.

## Changed Files

- `frontend/src/components/RunHistorySection.test.jsx`
- `frontend/src/components/BacktestHistorySection.test.jsx`
- `markdown/00-frontend-refactor-governance/frontend-module-tree.md`
- `markdown/00-frontend-refactor-governance/frontend-full-feature-tree.md`
- `markdown/00-frontend-refactor-governance/frontend-recursive-state.json`
- `markdown/00-frontend-refactor-governance/records/FE-0100-frontend-runtime-panels-history-sections-closeout.md`

## Public Surface

- `RunHistorySection`
- `BacktestHistorySection`
- Existing parent wrappers `StrategyRunsPanel` and `StrategyBacktestsPanel` keep delegating to these sections.

## Preserved Behavior

- Run history refresh, graph/compile/time/status/sort filters, current-graph shortcut, reset, detail loading, page-size changes, and pagination remain wired.
- Backtest history refresh, graph/compile/dataset/parameter/time filters, compare navigation, reset, compare toggle, routed detail opening, fallback detail loading, page-size changes, and pagination remain wired.
- Both sections still return no UI in detail mode.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; these sections own user-facing controls and callbacks that were previously embedded in the event shell.
- `leaf_split_positive_trigger`: `testability_gain`, `blast_radius_reduction`, and `white_box_boundary`.
- `leaf_split_stop_condition`: reached for this pass; direct component baselines now cover the extracted sections, and deeper splitting should wait until duplicated row/filter primitives are isolated by a later style or design-system leaf.
- `leaf_split_decision_result`: close `frontend.runtime_panels.history_sections` and continue to `frontend.runtime_panels.event_feed_and_candles`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/components/RunHistorySection.test.jsx src/components/BacktestHistorySection.test.jsx src/components/EventStreamPanel.backtestHistory.test.jsx src/components/EventStreamPanel.layout.test.jsx src/components/StrategyResearchConsole.test.jsx src/pages/StrategyBacktestsPage.test.jsx`: passed, 6 files / 12 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Step

Continue `frontend.runtime_panels` through `frontend.runtime_panels.event_feed_and_candles`.
