# Runtime / Backtest Explanation Contract

This file is the active wording boundary for `CL-P1-003`.

## Goal

Keep runtime detail, backtest detail, event stream, and replay surfaces on one
explanation protocol family.

## Explanation truth sources

- `runtime_diagnostics.node_details[*]`
- persisted `event_log.events[*]` payload facts
- persisted replay payloads that return the same event facts back by sequence

No page should invent a second explanation DTO family beside those facts.

## Shared frontend projection rules

- `RuntimeDiagnosticsPanel` renders the selected node through
  `buildRuntimeDiagnosticsProjection(...)`.
- `EventStreamPanel` history cards and `BacktestDetailPage` explanation cards
  aggregate from the same `runtime_diagnostics.node_details[*]` rows.
- `EventReplaySection` only reads event-level
  `payload.explanation_summary` or `payload.reason_text`; it does not build a
  second detail protocol.

## Allowed detail families

- `explanation_rows`
- `data_quality_rows`
- `risk_detail_rows`
- `order_detail_rows`

If a detail page needs more explanation, the backend must extend one of these
structured families or add runtime facts that the existing projection can read.
The frontend should not add ad-hoc explanation-only DTOs.

## Closeout rules

- event stream, diagnostics, replay, and backtest detail must agree on the same
  node name, row labels, and explanation summary when they refer to the same
  persisted fact
- persisted history reload must not reconstruct explanation rows from unrelated
  transient UI state
- event-level fallback is allowed only when `runtime_diagnostics` is absent, and
  it must still come from persisted event payload facts

## Current implementation entrypoints

- `frontend/src/utils/runtimeDiagnosticsProjection.js`
- `frontend/src/utils/runtimeExplanation.js`
- `frontend/src/components/RuntimeDiagnosticsPanel.jsx`
- `frontend/src/components/EventStreamPanel.jsx`
- `frontend/src/components/EventReplaySection.jsx`
- `frontend/src/pages/BacktestDetailPage.jsx`
