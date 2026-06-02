# FE-0110 Frontend Backtest Views Detail Page Model Closeout

Status: closed.

## Active Child Parent

`frontend.backtest_views.detail_page_analysis`

## Closed Subchild

`frontend.backtest_views.detail_page_analysis.artifact_model`

## Extraction

The backtest detail route now has a dedicated model surface for runtime artifact selection and route-context projection:

- `frontend/src/pages/backtestViews/detailPageAnalysis/index.js`
- `frontend/src/pages/backtestViews/detailPageAnalysis/backtestDetailPageModel.js`
- `frontend/src/pages/backtestViews/detailPageAnalysis/backtestDetailPageModel.test.js`

`frontend/src/pages/BacktestDetailPage.jsx` keeps the route shell, store hooks, side-effect load trigger, large section rendering, and event/report panel embedding.

## Whitebox Contract

### Public Inputs

- Route `backtestId` and optional `strategyId`.
- Runtime selected backtest id, backtest history, backtest artifacts, event timeline, compact evidence, and retained key-event index.

### Public Outputs

- Selected backtest id and selected history summary.
- Metrics, manifest, summary, start/end timestamps, output artifacts, v4 artifact, and v4 microstructure metrics.
- Resolved strategy id with route > history > artifact precedence.
- Equity and trade previews for the detail page replay preview.
- Timeline source object for governed timeline and report panels.

## Preserved Behavior

- Backtest detail page still loads detail through `loadBacktestDetail(backtestId)`.
- Empty/loading/backend-error behavior stays in the route component.
- The governed timeline, report panel, v4 evidence panel, replay preview, output artifacts, explanations, and event stream embedding still receive the same projected data.
- Route navigation remains unchanged.

## Further-Split Decision

Further split is required. `frontend.backtest_views.detail_page_analysis` is still a large route surface with multiple independent section clusters, so it is promoted to an active child parent. The next subchild queue is:

- `frontend.backtest_views.detail_page_analysis.summary_and_context`
- `frontend.backtest_views.detail_page_analysis.core_artifact_sections`
- `frontend.backtest_views.detail_page_analysis.evidence_report_sections`
- `frontend.backtest_views.detail_page_analysis.replay_output_explanation_sections`

The sibling `frontend.backtest_views.compare_page_analysis` remains queued as an open residual under the original backtest views parent.

## Verification

- From `frontend/`, detail page model target set: passed, 2 files / 4 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Child

`frontend.backtest_views.detail_page_analysis.summary_and_context`
