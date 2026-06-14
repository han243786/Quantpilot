# FE-0106 Frontend Runtime Panels Parent Closeout

Status: closed.

## Parent Node

`frontend.runtime_panels`

## Closed Children

- `frontend.runtime_panels.event_stream_shell`
- `frontend.runtime_panels.strategy_research_model`
- `frontend.runtime_panels.history_sections`
- `frontend.runtime_panels.event_feed_and_candles`
- `frontend.runtime_panels.runtime_diagnostics_surface`
- `frontend.runtime_panels.evidence_timeline_reports`
- `frontend.runtime_panels.mutation_controls`
- `frontend.runtime_panels.replay_and_explanations`

## Final Parent Boundary

`frontend.runtime_panels` now owns live and persisted event stream surfaces, strategy research runtime lanes, run/backtest history sections, event feed and candle panels, runtime diagnostics, governed evidence timeline, runtime reports, mutation controls, replay pagination, and shared runtime explanation helpers.

Backtest detail/compare pages, graph editor surfaces, global store migration, API transport, route contracts, and global design-system/style ownership remain outside this parent.

## Whitebox Contract

### Public Inputs

- Runtime store state for current events, selected run/backtest ids, persisted run/backtest history, artifacts, candles, diagnostics, and replay windows.
- Graph store graph metadata, selected-node state, selected strategy identity, and runtime action callbacks.
- Runtime history API functions for replay, reports, artifacts, and persisted run/backtest records.
- Capability context used by diagnostics, mutation controls, reports, and v4 runtime evidence surfaces.
- Runtime event payloads, governed envelopes, compact evidence projections, and v4 runtime evidence snapshots.

### Public Outputs

- `EventStreamPanel` and its runtime subpanels for live events, filters, artifacts, backtest/run history, replay, diagnostics, reports, mutations, and research console surfaces.
- History sections for runs and backtests with explanation cards, artifacts, report entry points, replay controls, and mutation/evidence embedding.
- Runtime contract readers and projections for diagnostics, timeline evidence, compact evidence summaries, mutation proposals, AI proposal records, v4 evidence, status labels, governance identity, and explanation rows.
- Stable UI callbacks for refreshing history, focusing nodes, opening/revealing artifacts and reports, activating/rolling back mutation proposals, and paging replay windows.

## Preserved Behavior

- Event stream shell still renders live and persisted runtime views through the same public component entry.
- Strategy research state/actions remain split behind stable hooks and selector helpers.
- Run/backtest history panels still render persisted records, artifacts, diagnostics, explanations, reports, and replay/mutation surfaces.
- Event feed and candle panels still respect node filters, event filters, search, data-quality metadata, and replay/live/history data-source priority.
- Runtime diagnostics still prefer backend projections and preserve selected-node switching and empty guidance.
- Evidence timeline/report surfaces still preserve governed identity, source filtering, export links, and failure surfacing.
- Mutation controls remain capability-gated and preserve activation/rollback boundaries.
- Replay/explanation helpers still preserve source readiness, sequence/legacy cursor paging, load failure surfacing, and explanation fallback behavior.
- No child-to-child shortcut was introduced during this parent; communication stays through props, store facades, API helpers, and runtime contract readers.

## Further-Split Decision

No further split is useful inside `frontend.runtime_panels` now. All planned child leaves are closed, and the remaining public surfaces are either stable compatibility shells or shared runtime contract readers that are now covered by focused equivalence baselines. Additional splitting should wait for a concrete runtime feature change, a UI extraction request, or a new backtest/store parent boundary.

## Verification

- From `frontend/`, runtime panels parent anchor test set: passed, 31 files / 94 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Parent Candidate

`frontend.backtest_views`
