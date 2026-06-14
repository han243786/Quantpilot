# FE-0114 Frontend Backtest Views Detail Replay Output Explanation Sections Closeout

Status: closed.

## Parent Node

`frontend.backtest_views.detail_page_analysis`

## Closed Subchild

`frontend.backtest_views.detail_page_analysis.replay_output_explanation_sections`

## Extraction

Backtest detail replay preview, output artifact references, and execution explanation rendering now live behind the detail analysis public surface:

- `frontend/src/pages/backtestViews/detailPageAnalysis/BacktestDetailReplayOutputExplanationSections.jsx`
- `frontend/src/pages/backtestViews/detailPageAnalysis/BacktestDetailReplayOutputExplanationSections.test.jsx`

`frontend/src/pages/BacktestDetailPage.jsx` now delegates equity/trade previews, output references, and risk/order explanation cards to the route-owned whitebox component.

## Whitebox Contract

### Public Inputs

- Translation function `t`.
- Equity curve preview points and trade preview rows.
- Equity curve artifact id, trade ledger artifact id, output artifact references.
- Risk and order explanation entries derived by the parent route from runtime diagnostics.

### Public Outputs

- Replay preview section with stable equity and trade card anchors.
- Output artifact section with stable output card anchor.
- Explanation section with stable risk and order card anchors.

## Preserved Behavior

- All existing `data-testid` anchors for replay, output, risk explanation, and order explanation remain stable.
- The parent route still owns runtime/graph state selection, diagnostics entry projection, sidebar context, and event stream embedding.
- The sidebar context and bottom `EventStreamPanel` remain outside this leaf and untouched.

## Further-Split Decision

No deeper split is useful inside `replay_output_explanation_sections` now. The leaf is already a compact presentation boundary with explicit props and focused tests. With this leaf closed, the active `detail_page_analysis` child parent has no remaining subchild queue and can proceed to parent closeout.

## Verification

- From `frontend/`, detail replay/output/explanation target set: passed, 4 files / 10 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Step

Close out `frontend.backtest_views.detail_page_analysis` as a child parent, then resume the `frontend.backtest_views.compare_page_analysis` residual.
