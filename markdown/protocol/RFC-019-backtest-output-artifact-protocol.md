# RFC-019 Backtest Output Artifact Protocol

## Status

Current status: draft

Applies to:

- `EventLogArtifact`
- `TradeLedgerArtifact`
- `EquityCurveArtifact`
- `MetricsArtifact`
- `ReproducibilityManifest`
- `BacktestArtifactViews`

## Goal

This RFC defines the stable output-side artifact schema for backtest execution.

The immediate goal is to replace ad hoc output JSON with explicit, versioned artifacts
that can serve three roles at once:

- detail-page rendering
- storage and reload
- reproducibility and later comparison

## Artifact Set

Backtest output is exposed as one artifact view bundle:

1. `EventLogArtifact`
2. `TradeLedgerArtifact`
3. `EquityCurveArtifact`
4. `MetricsArtifact`
5. `ReproducibilityManifest`

These are surfaced together as `BacktestArtifactViews` under `backtest_artifacts`.

## Versioned Objects

### EventLogArtifact

```json
{
  "schema_version": "quantpilot/event-log-artifact/v1",
  "artifact_id": "event_log_artifact_<digest-prefix>",
  "backtest_id": "backtest_test",
  "event_count": 3,
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "events": []
}
```

Purpose:

- provide the fact-source event sequence for a completed backtest
- give UI and audit tools one stable event payload instead of implicit top-level arrays
- anchor later deterministic projections
- carry enough stable projection context in each backtest event payload to derive trade, equity, and metric views without re-reading `BacktestOutput`

### TradeLedgerArtifact

```json
{
  "schema_version": "quantpilot/trade-ledger-artifact/v1",
  "artifact_id": "trade_ledger_artifact_<digest-prefix>",
  "backtest_id": "backtest_test",
  "trade_count": 1,
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "trades": [
    {
      "fill_id": "fill_1",
      "plan_id": "plan_1",
      "exchange": "Binance",
      "symbol": "BtcUsdt",
      "side": "buy",
      "filled_qty": 0.2,
      "filled_price": 50250.0,
      "fee_paid": 1.5,
      "filled_at_ms": 1700000060000,
      "status": "filled",
      "trace_id": "trace_1",
      "session_index": 0,
      "cycle_name": "slow"
    }
  ]
}
```

Purpose:

- expose fills in an analysis-friendly projection
- support detail-page trade tables and future run comparison
- preserve per-fill audit identity without forcing the UI to parse raw runtime events

### EquityCurveArtifact

```json
{
  "schema_version": "quantpilot/equity-curve-artifact/v1",
  "artifact_id": "equity_curve_artifact_<digest-prefix>",
  "backtest_id": "backtest_test",
  "point_count": 2,
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "points": []
}
```

Purpose:

- expose the persisted equity-curve projection used by detail pages and later comparisons
- decouple chart rendering from the raw `BacktestOutput` object

### MetricsArtifact

```json
{
  "schema_version": "quantpilot/metrics-artifact/v1",
  "artifact_id": "metrics_artifact_<digest-prefix>",
  "backtest_id": "backtest_test",
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "summary": {},
  "event_count": 3,
  "session_count": 1,
  "started_at_ms": 1700000000000,
  "ended_at_ms": 1700000060000,
  "final_account": {}
}
```

Purpose:

- expose the summary block used by list pages and detail headers
- capture the minimal result metrics needed for comparison and repro checks
- provide a stable artifact id for result-level caching and UI references

### ReproducibilityManifest

```json
{
  "schema_version": "quantpilot/reproducibility-manifest/v1",
  "manifest_id": "manifest_backtest_test",
  "backtest_id": "backtest_test",
  "graph_id": "graph_test",
  "compile_id": "compile_test",
  "created_at_ms": 1700000060000,
  "protocol_name": "quantpilot/runtime-config/v1",
  "config_hash": "runtime-spec-...",
  "account": {},
  "summary": {},
  "backtest_spec": {},
  "compile_artifacts": {},
  "output_artifacts": [],
  "backtest_output_digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  }
}
```

Purpose:

- connect input-side and output-side artifacts into one reproducibility boundary
- list the persisted output files for storage reload
- keep compile identity, backtest identity, summary, and digest anchors on one object

## Projection Rules

- `EventLogArtifact` is the stable event boundary for backtest detail consumers.
- `TradeLedgerArtifact`, `EquityCurveArtifact`, and `MetricsArtifact` are deterministic projections derived from `EventLogArtifact`.
- `ReproducibilityManifest` references `BacktestSpec`, `CompileArtifactBundle`, and every persisted output artifact file.
- Backtest API detail consumers should read `backtest_artifacts` directly; top-level `events` and `backtest` fields are not part of the output-artifact contract.

Implementation note:

- The current beta implementation persists and reloads these artifacts as an event-log-first bundle.
- Projection evolution must preserve this contract: `TradeLedgerArtifact`, `EquityCurveArtifact`, and `MetricsArtifact` are rebuilt from the event log boundary, not from ad hoc side channels.

## API Boundary

Current response shape:

- `POST /api/runtime/backtest`
- `GET /api/runtime/backtests/{backtest_id}`

Both responses may include:

```json
{
  "backtest_artifacts": {
    "event_log": {},
    "trade_ledger": {},
    "equity_curve": {},
    "metrics": {},
    "manifest": {}
  }
}
```

Detail-page readers should use:

- `backtest_artifacts.event_log.events` for event replay and node highlighting
- `backtest_artifacts.metrics` for summary and event/session counts
- `backtest_artifacts.trade_ledger` for fill tables
- `backtest_artifacts.equity_curve` for chart inputs
- `backtest_artifacts.manifest` for reproducibility metadata

## Storage Layout

Persisted directory layout:

```text
storage/backtests/<backtest_id>/
  event_log.json
  trade_ledger.json
  equity_curve.json
  metrics.json
  backtest_output.json
  manifest.json
```

Rules:

- `manifest.json` is the storage entry point
- reload may reconstruct the full `BacktestRecord` from the directory without relying on legacy single-file backtest JSON
- output file references listed in `manifest.output_artifacts` must match the persisted artifact files

## Digest Rule

All output artifact digests use:

- algorithm: `sha256_canonical_json`
- canonical form: `serde_json::to_vec(...)` on the artifact payload
- output format: lowercase hex string

Notes:

- `artifact_id` is a readable identifier derived from the digest prefix
- `backtest_output_digest` is distinct from the artifact digests; it hashes the full `BacktestOutput` payload
- output artifacts must not invent a second compile identity; they inherit compile identity through `compile_artifacts` and `backtest_spec`

## Current Implementation

Current code paths:

- output artifact structs and persistence: `src/backtest_artifacts.rs`
- backtest API assembly and reload: `src/main.rs`
- frontend detail rendering: `frontend/src/pages/BacktestDetailPage.jsx`
- frontend event stream and summary panels: `frontend/src/components/EventStreamPanel.jsx`

## Out of Scope

This RFC does not yet define:

- multi-run comparison artifact schemas
- paginated artifact query endpoints
- incremental event-log streaming for persisted backtests
- experiment-set manifests across multiple backtests
