# FE-0204 - Frontend Backtest Analysis Page Styles Baseline

Status: closed.

## Child Parent Node

`frontend.design_system_styles.page_style_contracts.backtest_analysis_page_styles`

## Scope

- Primary file: `frontend/src/pages/backtest-analysis.css`
- Current file size at baseline: 593 lines.
- Purpose: backtest detail/compare analysis shell, page-level tokens, route bar, typography, summary cards, page grid, section/card overrides, responsive compare-page overrides, and reduced-motion behavior.

## Why This Becomes A Child Parent

- The file is far above the direct leaf threshold.
- It mixes shared backtest analysis shell styles with compare-page-specific responsive overrides.
- It contains distinct visual contracts that can be extracted without changing page imports or runtime behavior.

## Initial Subchild Queue

- `frontend.design_system_styles.page_style_contracts.backtest_analysis_page_styles.shell_tokens_surface_chrome`
- `frontend.design_system_styles.page_style_contracts.backtest_analysis_page_styles.route_bar_and_typography`
- `frontend.design_system_styles.page_style_contracts.backtest_analysis_page_styles.summary_cards_and_page_grid`
- `frontend.design_system_styles.page_style_contracts.backtest_analysis_page_styles.analysis_sections_and_card_contracts`
- `frontend.design_system_styles.page_style_contracts.backtest_analysis_page_styles.responsive_compare_motion_overrides`

## Parent Return

- After this child parent closes, return to `frontend.design_system_styles.page_style_contracts`.
- Remaining parent queue after closeout:
  - `frontend.design_system_styles.page_style_contracts.strategy_hub_page_style_contracts`
  - `frontend.design_system_styles.page_style_contracts.strategy_workspace_page_style_contracts`

## Verification

- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
