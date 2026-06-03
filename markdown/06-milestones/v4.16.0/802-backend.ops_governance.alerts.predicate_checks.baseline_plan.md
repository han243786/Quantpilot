# v4.16.0 backend.ops_governance.alerts.predicate_checks equivalence baseline and extraction plan

> Batch: BE-001MZ-01
> Node: `backend.ops_governance.alerts.predicate_checks`
> Parent: `backend.ops_governance.alerts`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts.predicate_checks` is frozen as the alert predicate evaluation child.

BE-001MZ-01 does not move code. It defines the exact baseline and allowed movement for BE-001MZ-02.

## Current Owner

Current implementation is still in `src/backend/ops_governance/alerts/handlers.rs`.

The child boundary is:

- alert rule dispatch by `rule.rule_name`;
- `check_data_freshness`;
- `check_event_orphan`;
- `check_risk_reject_rate`;
- `check_replay_divergence`;
- `check_sandbox_timeout`;
- `check_hotswap_rollback`;
- `check_capability_hash_mismatch`;
- `check_storage_watermark`;
- `check_approval_expiry`;
- `check_ai_reject_rate`.

The parent bridge is:

- `should_fire_alert`

The parent bridge must remain callable by `trigger_engine`.

## Frozen Semantics

The next extraction must preserve:

| Surface | Frozen behavior |
| --- | --- |
| Dispatch | Rule names map to the same predicate checks; unknown names return false. |
| Data freshness | Evidence compact retained/source ratio check stays unchanged. |
| Event orphan | Compact detail required count threshold stays unchanged. |
| Risk reject rate | Mutation proposal rejected/created ratio threshold stays unchanged. |
| Replay divergence | Report generation failure count check stays unchanged. |
| Sandbox timeout | User-scoped sandbox report age check and env default remain unchanged. |
| Hotswap rollback | User-scoped rollback reason check remains unchanged. |
| Capability hash mismatch | User-scoped backtest governance hash comparison remains unchanged. |
| Storage watermark | Env parsing, storage root, cleanup call, and size comparison remain unchanged. |
| Approval expiry | User-scoped pending approval expiry window remains unchanged. |
| AI reject rate | User-scoped 24h proposal rejection ratio remains unchanged. |

## Allowed BE-001MZ-02 Movement

BE-001MZ-02 may:

- create a private child module for predicate checks under the alerts handler owner boundary;
- move predicate dispatch and metric-specific helpers into that child;
- keep a parent bridge named `should_fire_alert` that delegates to the child;
- keep `is_condition_resolved` in the parent so trigger engine still calls parent-owned recovery mediation.

## Forbidden BE-001MZ-02 Movement

BE-001MZ-02 must not move or rewrite:

- trigger route orchestration;
- alert list or alert rule list routes;
- rule catalog;
- acknowledge flow;
- alert firing persistence helper implementation;
- startup initialization bridge;
- AppState fields or lock ordering beyond preserving existing read locks;
- frontend API schema types;
- snapshots, runbook, chaos, hotswap, or sandbox modules;
- release transition logic.

## Proof

No direct predicate unit tests were found in the current alerts handler test filter. BE-001MZ-02 must therefore keep the movement mechanical and prove equivalence with compile, existing alerts tests, and governance gates.

## Next Step

BE-001MZ-02 backend.ops_governance.alerts.predicate_checks extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
