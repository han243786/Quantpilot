# FE-0098 Frontend Runtime Panels Event Stream Shell Closeout

Status: closed.

## Child Node

`frontend.runtime_panels.event_stream_shell`

## Boundary

This leaf owns the default `EventStreamPanel` shell and its public helper exports for event panel layout, feed rendering, backtest summary, account summary, and shared history UI primitives.

## Changed Files

- `frontend/src/components/EventStreamPanel.jsx`

## Public Surface

- `EventStreamPanel`
- `EventPanelIntro`
- `EventFeedSection`
- `BacktestSummarySection`
- `AccountSection`
- History UI primitives and runtime formatting helpers still exported from `EventStreamPanel.jsx` for existing section consumers.

## Preserved Behavior

- The default event stream panel still renders the intro, asset candles, event feed, backtest summary, account summary, and replay section through the same model hook.
- Existing section files still import public helper exports from `EventStreamPanel.jsx`.
- The previously migrated `BacktestHistorySection` and `RunHistorySection` local remnants were removed from the shell file; the live standalone section files remain unchanged.
- Removed imports were no longer used after the dead local section remnants were deleted.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; `EventStreamPanel.jsx` still carries exported helper primitives and default shell orchestration.
- `leaf_split_positive_trigger`: `blast_radius_reduction`, `dead_code_removal`, and `testability_gain`.
- `leaf_split_stop_condition`: reached for the first shell cleanup pass; deeper runtime panel splitting should continue through the next queued leaves instead of hiding history/research/diagnostics ownership inside this shell.
- `leaf_split_decision_result`: continue `frontend.runtime_panels` through `frontend.runtime_panels.strategy_research_model`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/components/EventStreamPanel.layout.test.jsx src/components/EventStreamPanel.backtestArtifacts.test.jsx src/components/EventStreamPanel.backtestHistory.test.jsx src/components/EventStreamPanel.dataQuality.test.jsx src/components/EventStreamPanel.executionExplanation.test.jsx src/components/EventStreamPanel.historyExplanation.test.jsx src/components/EventStreamPanel.nodeFocus.test.jsx src/components/EventStreamPanel.refreshFeedback.test.jsx src/components/EventStreamPanel.runtimeArtifactActions.test.jsx src/components/EventReplaySection.test.jsx src/components/AssetCandlesPanel.test.jsx`: passed, 11 files / 18 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Step

Continue `frontend.runtime_panels` through `frontend.runtime_panels.strategy_research_model`.
