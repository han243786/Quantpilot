# FE-0177 - Frontend Design System Styles Baseline

Status: open.

## Parent Node

`frontend.design_system_styles`

## Baseline Scope

- Current owned and split-target file count: 19 files.
- Scope includes global CSS entry imports, app/root loading and toast shells, design tokens, native element styling, shared visual primitives, responsive panel overrides, and page-level style contract aggregators that were already split by earlier parent passes.

## Owned And Split-Target Files

- `frontend/src/main.jsx`
- `frontend/src/design-system.css`
- `frontend/src/styles.css`
- `frontend/src/styles-responsive-panels.css`
- `frontend/src/shared.css`
- `frontend/src/pages/backtest-analysis.css`
- `frontend/src/pages/strategy-hub.css`
- `frontend/src/pages/strategy-hub-shell-hero.css`
- `frontend/src/pages/strategy-hub-notes-tasks-status.css`
- `frontend/src/pages/strategy-hub-layout-template.css`
- `frontend/src/pages/strategy-hub-roster.css`
- `frontend/src/pages/strategy-hub-inspector-activity.css`
- `frontend/src/pages/strategy-hub-responsive.css`
- `frontend/src/pages/strategy-workspace.css`
- `frontend/src/pages/strategy-workspace-shell.css`
- `frontend/src/pages/strategy-workspace-overview-diagnostics.css`
- `frontend/src/pages/strategy-workspace-builder-inspector.css`
- `frontend/src/pages/strategy-workspace-cards-runtime.css`
- `frontend/src/pages/strategy-workspace-responsive.css`

## Important Consumers

- `frontend/src/main.jsx`
- `frontend/src/app/AppRoot.jsx`
- `frontend/src/pages/StrategyWorkspacePage.jsx`
- `frontend/src/pages/StrategyHubPage.jsx`
- `frontend/src/pages/BacktestDetailPage.jsx`
- `frontend/src/pages/BacktestComparePage.jsx`
- Frontend components that consume shared class contracts under `frontend/src/components`.

## Candidate Child Queue

- `frontend.design_system_styles.global_style_entry`
- `frontend.design_system_styles.design_tokens_and_native_controls`
- `frontend.design_system_styles.shared_component_primitives`
- `frontend.design_system_styles.responsive_panel_overrides`
- `frontend.design_system_styles.page_style_contracts`

## Whitebox Boundary

- Parent-to-child communication is CSS entry import order plus explicit class/token contracts.
- Child nodes may not introduce direct cross-child shortcuts; shared behavior must remain in the parent contract or an explicitly extracted shared primitive.
- Page-level CSS previously closed under `frontend.strategy_workspace.layout_styles`, `frontend.strategy_hub.layout_styles`, and `frontend.backtest_views` remains behavior-owned by those parents; this parent owns only the global style contract and the integration map for those page style entries.
- Visual or cascade changes require at least build verification, and risky selector changes should be promoted to a separate code step with browser or targeted visual verification.

## Prior Evidence

- `markdown/00-frontend-refactor-governance/records/FE-0048-frontend-strategy-workspace-layout-styles-closeout.md`
- `markdown/00-frontend-refactor-governance/records/FE-0060-frontend-strategy-hub-layout-styles-closeout.md`
- `markdown/00-frontend-refactor-governance/records/FE-0120-frontend-backtest-views-parent-closeout.md`

## Further-Split Decision

The parent is worth splitting now. The current style surface mixes global entry orchestration, tokens/native defaults, shared primitives, responsive overrides, and page-level style contracts. Those are distinct whitebox responsibilities, and isolating them makes later frontend refactor work safer without changing runtime behavior in this baseline step.

## Verification

- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
