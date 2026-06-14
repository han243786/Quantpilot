# FE-0113 Frontend Backtest Views Detail Evidence Report Sections Closeout

Status: closed.

## Parent Node

`frontend.backtest_views.detail_page_analysis`

## Closed Subchild

`frontend.backtest_views.detail_page_analysis.evidence_report_sections`

## Extraction

Backtest detail evidence and report lifecycle rendering now lives behind the detail analysis public surface:

- `frontend/src/pages/backtestViews/detailPageAnalysis/BacktestDetailEvidenceReportSections.jsx`
- `frontend/src/pages/backtestViews/detailPageAnalysis/BacktestDetailEvidenceReportSections.test.jsx`

`frontend/src/pages/BacktestDetailPage.jsx` now delegates governed timeline and report lifecycle sections through route-owned whitebox components while preserving the v4 artifact section between them.

## Whitebox Contract

### Public Inputs

- Translation function `t`.
- Timeline source projection from the detail page model.
- Backtest source id for the report lifecycle section.

### Public Outputs

- Governed timeline analysis section with stable detail route test ids.
- Runtime report lifecycle section bound to `sourceKind="backtest"` and the current backtest id.

## Preserved Behavior

- The governed timeline still uses the same `backtest-detail-governed-timeline` and `backtest-detail-timeline` anchors.
- The report lifecycle still uses the same `backtest-detail-report-lifecycle` and `runtime-report-panel` anchors.
- The v4 artifact section remains between the timeline and report sections.
- Replay preview, output artifact references, execution explanations, sidebar context, and event stream remain untouched.

## Further-Split Decision

No deeper split is useful inside `evidence_report_sections` now. Timeline and report lifecycle are already backed by mature shared components; this leaf only owns route-specific section framing and explicit prop handoff. Continue the active detail child parent through `replay_output_explanation_sections`.

## Verification

- From `frontend/`, detail evidence/report target set: passed, 4 files / 12 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Child

`frontend.backtest_views.detail_page_analysis.replay_output_explanation_sections`
