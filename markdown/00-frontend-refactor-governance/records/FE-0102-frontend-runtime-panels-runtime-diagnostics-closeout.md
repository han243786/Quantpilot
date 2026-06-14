# FE-0102 Frontend Runtime Panels Runtime Diagnostics Closeout

Status: closed.

## Child Node

`frontend.runtime_panels.runtime_diagnostics_surface`

## Boundary

This leaf owns the runtime diagnostics panel and its projection helper. It covers raw event projection, backend-projected diagnostics, governance identity rows, evidence/timeline/report embedding, selected-node switching, empty diagnostics guidance, and selected-node fallback through the graph store.

## Changed Files

- `frontend/src/components/RuntimeDiagnosticsPanel.test.jsx`
- `markdown/00-frontend-refactor-governance/frontend-module-tree.md`
- `markdown/00-frontend-refactor-governance/frontend-full-feature-tree.md`
- `markdown/00-frontend-refactor-governance/frontend-recursive-state.json`
- `markdown/00-frontend-refactor-governance/records/FE-0102-frontend-runtime-panels-runtime-diagnostics-closeout.md`

## Public Surface

- `RuntimeDiagnosticsPanel`
- `buildRuntimeDiagnosticsProjection`

## Preserved Behavior

- Runtime diagnostics still render selected-node snapshots, governance rows, V4 evidence, governed timeline, report entry points, data quality rows, explanation rows, risk rows, order rows, and recent node events.
- Backend-projected diagnostics still take precedence over local raw-event projection.
- Empty diagnostics still render guidance instead of the full diagnostics panel.
- Node switching still calls an explicit `onSelectNode` when supplied and falls back to the graph store selected-node action otherwise.

## Leaf Split Decision Gate

- `leaf_split_base_gate`: triggered; diagnostics combine panel orchestration with projection helpers and embedded runtime subpanels.
- `leaf_split_positive_trigger`: `testability_gain` and `blast_radius_reduction`.
- `leaf_split_stop_condition`: reached for this pass; projection and panel baselines now cover the critical boundaries, and deeper extraction should wait until the evidence/timeline/report leaf is handled.
- `leaf_split_decision_result`: close `frontend.runtime_panels.runtime_diagnostics_surface` and continue to `frontend.runtime_panels.evidence_timeline_reports`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/components/RuntimeDiagnosticsPanel.test.jsx src/utils/runtimeDiagnosticsProjection.test.js src/components/GovernedTimelinePanel.test.jsx src/components/RuntimeReportPanel.test.jsx src/components/V4RuntimeEvidencePanel.test.jsx`: passed, 5 files / 13 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Step

Continue `frontend.runtime_panels` through `frontend.runtime_panels.evidence_timeline_reports`.
