# FE-0205 - Frontend Backtest Analysis Shell Tokens Surface Chrome Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.page_style_contracts.backtest_analysis_page_styles.shell_tokens_surface_chrome`

## Code Changes

- Added `frontend/src/pages/backtest-analysis/shell-tokens-surface-chrome.css`.
- Moved detail page shell, analysis-page tokens, compare token overrides, header chrome, hero/section/followup surface chrome, and surface glint rules out of `frontend/src/pages/backtest-analysis.css`.
- Kept `frontend/src/pages/backtest-analysis.css` as the ordered page-style import aggregator plus remaining route, summary, section, and responsive rules.

## Preserved Behavior

- Extracted selector bodies and cascade order are unchanged after import expansion.
- Backtest detail and compare pages continue to import only `frontend/src/pages/backtest-analysis.css`.

## Public Inputs

- Design-system token values and analysis page DOM classes.
- Backtest detail/compare layout wrappers.

## Public Outputs

- `frontend/src/pages/backtest-analysis/shell-tokens-surface-chrome.css`
- `frontend/src/pages/backtest-analysis.css`

## Further-Split Decision

No deeper split is useful inside `shell_tokens_surface_chrome` now. The leaf owns the common page shell and tokenized surface chrome that must stay ordered before route, summary, and section rules.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/pages/backtest-analysis.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
