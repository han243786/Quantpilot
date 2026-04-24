# RFC-017 Backtest Artifact Protocol

## Status

Current status: draft

Applies to:

- compile artifact bundle returned by `POST /api/runtime/compile`
- compile artifact bundle embedded in `POST /api/runtime/backtest`
- persisted backtest records under `storage/backtests/*.json`

## Goal

This RFC defines the stable artifact boundary for the current run-centered beta.

The immediate purpose is to stop treating backtest persistence as an ad hoc dump of:

- `protocol_name`
- `config_hash`
- `core_ir`
- runtime/backtest output JSON

Instead, the system should persist and expose explicit artifacts with stable names,
schema versions, and digest rules.

## Artifact Set

The current artifact bundle contains three objects:

1. `StrategyArtifact`
2. `CompileArtifact`
3. `CoreIrArtifact`

These are surfaced together as `CompileArtifactBundle`.

## Versioned Objects

### StrategyArtifact

```json
{
  "schema_version": "quantpilot/strategy-artifact/v1",
  "artifact_id": "strategy_artifact_<digest-prefix>",
  "graph_id": "graph_test",
  "compile_id": "compile_test",
  "strategy_id": "graph_test",
  "name": "Test Graph",
  "source_kind": "frontend_graph",
  "source_ref": "graph_test",
  "metadata": {
    "runtime_mode": "paper"
  },
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  }
}
```

Purpose:

- identify the user-facing strategy source
- distinguish source provenance from compile output
- give storage and later artifact projections a stable parent object

### CoreIrArtifact

```json
{
  "schema_version": "quantpilot/core-ir-artifact/v1",
  "artifact_id": "core_ir_artifact_<digest-prefix>",
  "graph_id": "graph_test",
  "compile_id": "compile_test",
  "ir_version": "quantpilot/core-ir/v1",
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "core_ir": {}
}
```

Purpose:

- freeze the exact lowered Core IR used by runtime
- provide a canonical digest anchor for `RunSpec`
- make future artifact projections referenceable without re-deriving Core IR

### CompileArtifact

```json
{
  "schema_version": "quantpilot/compile-artifact/v1",
  "artifact_id": "compile_artifact_<digest-prefix>",
  "graph_id": "graph_test",
  "compile_id": "compile_test",
  "protocol_name": "quantpilot/minimal-sim/v1",
  "config_hash": "runtime-spec-...",
  "strategy_artifact_id": "strategy_artifact_...",
  "core_ir_artifact_id": "core_ir_artifact_...",
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "runtime_config": {}
}
```

Purpose:

- freeze the compiled runtime protocol config
- connect the source artifact to the lowered Core IR artifact
- provide the stable compile boundary used by run/backtest specs

## Digest Rule

All artifact digests use:

- algorithm: `sha256_canonical_json`
- canonical form: `serde_json::to_vec(...)` on the artifact payload
- output format: lowercase hex string

Notes:

- `artifact_id` is not the full digest; it is a readable identifier derived from the digest prefix
- `config_hash` remains the runtime-protocol hash used by existing compile consumers
- artifact digests and `config_hash` serve different roles and should not be conflated

## Boundary Rules

- `StrategyArtifact` is about source identity and provenance, not runtime semantics
- `CompileArtifact` is about the compiled runtime input boundary
- `CoreIrArtifact` is about the executable lowered representation
- output artifacts defined in `RFC-019` should reference this bundle instead of introducing
  parallel compile identities

## Current Implementation

Current code paths:

- shared schema types: `qrpc_core/src/lib.rs`
- compile endpoint assembly: `src/main.rs`
- backtest endpoint persistence: `src/main.rs`

## Out of Scope

This RFC does not yet define multi-run comparison artifact layout.

Backtest output-side artifacts are defined in `RFC-019`.
