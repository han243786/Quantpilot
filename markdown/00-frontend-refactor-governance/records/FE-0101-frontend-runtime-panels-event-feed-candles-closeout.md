# FE-0101 Frontend Runtime Panels Event Feed And Candles Closeout

Status: closed.

## Child Node

`frontend.runtime_panels.event_feed_and_candles`

## Boundary

This leaf owns the event feed interaction surface and the asset candles projection surface. It covers event node scope chips, event type filtering, search debounce, clear actions, event-row node focus, data-quality metadata rendering, and asset candle data-source priority.

## Changed Files

- `frontend/src/components/EventFeedSection.test.jsx`
- `frontend/src/components/AssetCandlesPanel.test.jsx`
- `markdown/00-frontend-refactor-governance/frontend-module-tree.md`
- `markdown/00-frontend-refactor-governance/frontend-full-feature-tree.md`
- `markdown/00-frontend-refactor-governance/frontend-recursive-state.json`
- `markdown/00-frontend-refactor-governance/records/FE-0101-frontend-runtime-panels-event-feed-candles-closeout.md`

## Public Surface

- `EventFeedSection`
- `AssetCandlesPanel`

## Preserved Behavior

- Event feed node chips still switch between all-node scope and selected-node focus.
- Event type filtering and debounced search still call the provided runtime panel setters.
- Event feed clear still resets type and search filters.
- Clicking an event row with a node id still focuses that node through the parent callbacks.
- Data-quality event metadata still renders freshness, gap count, source health, and quality flags.
- Asset candles still prefer backtest replay snapshots first, live/current run event snapshots second, and graph-matched run history snapshots last.
- Empty event and candle states still render without chart or event rows.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; the event feed is still exported from the event shell and shares helper primitives with it.
- `leaf_split_positive_trigger`: `testability_gain` and `white_box_boundary`.
- `leaf_split_stop_condition`: reached for this pass; moving `EventFeedSection` now would also require helper relocation, so the direct baseline lands first and keeps later helper extraction safe.
- `leaf_split_decision_result`: close `frontend.runtime_panels.event_feed_and_candles` and continue to `frontend.runtime_panels.runtime_diagnostics_surface`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/components/EventFeedSection.test.jsx src/components/AssetCandlesPanel.test.jsx src/components/EventStreamPanel.dataQuality.test.jsx src/components/EventStreamPanel.layout.test.jsx`: passed, 4 files / 10 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Step

Continue `frontend.runtime_panels` through `frontend.runtime_panels.runtime_diagnostics_surface`.
