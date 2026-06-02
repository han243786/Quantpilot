# FE-0112 Frontend Backtest Views Detail Core Artifact Sections Closeout

Status: closed.

## Parent Node

`frontend.backtest_views.detail_page_analysis`

## Closed Subchild

`frontend.backtest_views.detail_page_analysis.core_artifact_sections`

## Extraction

Backtest detail core artifact rendering now lives behind the detail analysis public surface:

- `frontend/src/pages/backtestViews/detailPageAnalysis/BacktestDetailCoreArtifactSections.jsx`
- `frontend/src/pages/backtestViews/detailPageAnalysis/BacktestDetailCoreArtifactSections.test.jsx`

`frontend/src/pages/BacktestDetailPage.jsx` now delegates manifest, metrics, drawdown, monthly returns, and v4 artifact rendering to the detail analysis whitebox components while preserving the original placement of governed timeline, reports, replay, output references, explanations, sidebar context, and event stream.

## Whitebox Contract

### Public Inputs

- Translation function `t`.
- Selected summary, manifest, metrics, summary, started/ended timestamps, governance rows, equity curve, period returns, metrics artifact id, and event count.
- Optional v4 artifact and v4 microstructure metrics.

### Public Outputs

- Core artifact section with manifest, metrics, governance identity, drawdown chart, and monthly returns section.
- Optional v4 evidence section with v4 artifact and microstructure cards.

## Preserved Behavior

- All existing `data-testid` anchors for the core artifact, chart, v4, and route smoke tests remain stable.
- The v4 section stays after the governed timeline, matching the previous DOM order.
- The route continues to own loading/error/empty states, summary expansion state, navigation actions, timeline/report/replay/output/explanation sections, and sidebar context.

## Further-Split Decision

No deeper split is useful inside `core_artifact_sections` now. The extracted component is a route-owned presentation boundary with explicit props and focused tests. Continue the active detail child parent through `evidence_report_sections`.

## Verification

- From `frontend/`, detail core artifact target set: passed, 3 files / 6 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Child

`frontend.backtest_views.detail_page_analysis.evidence_report_sections`
