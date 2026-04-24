# RFC-018 Backtest Input Protocol

## Status

Current status: draft

Applies to:

- `RunSpec`
- `BacktestSpec`
- `DatasetSpec`
- `MarketDataSnapshotSpec`
- `ExecutionAssumptionSpec`

## Goal

This RFC defines the stable input-side schema for run/backtest execution.

The immediate goal is to make the following boundary explicit:

- what was compiled
- what market data shape was requested
- what execution assumptions were used
- what replay mode produced the result

## Core Objects

### DatasetSpec

```json
{
  "dataset_id": "data_data_1",
  "data_id": "data_data_1",
  "exchange": "Binance",
  "symbol": "BtcUsdt",
  "market_type": "Spot",
  "kind": "KlineSeries",
  "interval": "1d",
  "lookback_days": 200,
  "enabled": true
}
```

Purpose:

- freeze the requested dataset boundary in a run-friendly shape
- decouple future run/backtest schemas from frontend node JSON

### ExecutionAssumptionSpec

```json
{
  "initial_cash_balance": 100000.0,
  "taker_fee_bps": 10.0,
  "default_slippage_bps": 5.0,
  "total_cost_buffer_bps": 20.0,
  "time_in_force": "Gtc",
  "allow_partial_fills": true,
  "latency_assumption_ms": null
}
```

Purpose:

- freeze the execution and cost assumptions that materially affect reproducibility

### RunSpec

```json
{
  "schema_version": "quantpilot/run-spec/v1",
  "run_mode": "backtest",
  "graph_id": "graph_test",
  "compile_id": "compile_test",
  "runtime_mode": "paper",
  "protocol_name": "quantpilot/minimal-sim/v1",
  "config_hash": "runtime-spec-...",
  "datasets": [],
  "execution_assumptions": {},
  "core_ir_digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  }
}
```

Purpose:

- provide one run-centered identity object shared by paper run and backtest flows
- anchor later output artifacts to a single compile-and-assumption boundary

### MarketDataSnapshotSpec

```json
{
  "snapshot_id": "market_snapshot_backtest_123",
  "replay_source": "deterministic_mock",
  "captured_at_ms": 1700000000000,
  "datasets": []
}
```

Purpose:

- describe the replay dataset boundary used by a specific backtest request
- keep historical replay and deterministic mock replay on one typed axis

### BacktestSpec

```json
{
  "schema_version": "quantpilot/backtest-spec/v1",
  "backtest_id": "backtest_1700000000000",
  "replay_source": "deterministic_mock",
  "requested_at_ms": 1700000000000,
  "run_spec": {},
  "market_data_snapshot": {}
}
```

Purpose:

- freeze the exact input-side backtest boundary
- provide a stable parent object for later event log and metrics projections

## Replay Source Semantics

Current supported values:

- `historical_replay`
- `deterministic_mock`

Rules:

- both replay modes must still use the same compile artifact boundary
- replay mode changes data source realization, not compile semantics
- replay mode must be explicit in `BacktestSpec` and `MarketDataSnapshotSpec`

## Boundary Rules

- `RunSpec` is the shared input boundary
- `BacktestSpec` is `RunSpec + replay-specific market data context`
- `DatasetSpec` is derived from runtime protocol data sources, not frontend module JSON
- `ExecutionAssumptionSpec` must contain only semantics that can change execution outcome

## Current Implementation

Current code paths:

- shared schema types: `qrpc_core/src/lib.rs`
- backtest spec assembly: `src/main.rs`
- compile/backtest API exposure: `src/main.rs`

## Relationship to Future Work

This RFC is intentionally limited to input-side contracts.

The next stage should add:

- event log artifact projection
- trade ledger projection
- equity curve projection
- metrics projection
- reproducibility manifest

Those output-side objects should reference `BacktestSpec` instead of inventing new input summaries.
