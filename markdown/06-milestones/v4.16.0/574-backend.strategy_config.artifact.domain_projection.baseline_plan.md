# v4.16.0 backend.strategy_config.artifact.domain_projection equivalence baseline and extraction plan

> Batch: BE-001HU-01
> Node: `backend.strategy_config.artifact.domain_projection`
> Parent: `backend.strategy_config.artifact`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.strategy_config.artifact.domain_projection equivalence baseline and extraction plan.

## Equivalence Baseline

The move must preserve all config domain projection behavior:

- Domain ordering: market, observation, state_machine, risk, execution,
  evidence, AI governance, snapshot.
- Domain readiness/lifecycle semantics and primary actions.
- Existing findings codes/messages and source refs.
- v4 machine graph static-contract validation behavior for state_machine/risk.
- PaperActual demo boundary and stale capability warnings.
- AI proposal binding digest warnings.
- Snapshot/evidence anchor readiness.

## Extraction Plan

BE-001HU-02 may create:

- `src/backend/strategy_config/artifact/domain_projection.rs`

Move only these functions into that child:

- `build_config_domains`
- `market_domain`
- `observation_domain`
- `state_machine_domain`
- `risk_domain`
- `execution_domain`
- `evidence_domain`
- `ai_governance_domain`
- `snapshot_domain`
- `refs_from_pairs`
- `finding`

Allowed compatibility:

- Re-export `build_config_domains` and `finding` from
  `backend.strategy_config.artifact` while residual preflight/evidence diff code
  still calls them from `src/strategy_config_api.rs`.
- Use `pub(crate)` visibility only where residual root code still needs access.

Forbidden changes:

- Do not move builder orchestration, source/capability/runtime boundary
  builders, digest helpers, preflight report logic, diff/evidence diff logic, or
  route registration in this batch.
- Do not change domain order, readiness, findings, serde shape, graph compare,
  migration sender, frontend type, or runtime mutation behavior.

## Boundary

**Real files**:
- `src/backend/strategy_config/artifact.rs`
- `src/backend/strategy_config/artifact/schema_model.rs`
- `src/backend/strategy_config/artifact/domain_projection.rs`
- `src/strategy_config_api.rs`

**Markers**:
- `domain_projection baseline_frozen`
- `domain_projection plan_frozen`

**Next step**:
BE-001HU-02 backend.strategy_config.artifact.domain_projection extract_closeout

---

## Gates

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot strategy_config --lib`
- `cargo test -p quantpilot graph_version_endpoints_list_load_and_restore_versions`
- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
