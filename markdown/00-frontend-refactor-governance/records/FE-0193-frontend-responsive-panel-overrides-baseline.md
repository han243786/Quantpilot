# FE-0193 - Frontend Responsive Panel Overrides Baseline

Status: closed.

## Child Parent Node

`frontend.design_system_styles.responsive_panel_overrides`

## Scope

- Primary file: `frontend/src/styles-responsive-panels.css`
- Current file size at baseline: 1336 lines.
- Purpose: responsive panel breakpoints, runtime panel layout styles, reduced-motion overrides, tutorial overlay styles, dashboard/config styles, QuantScript editor/source/debug styles, print styles, and legacy inline page style migrations.

## Why This Becomes A Child Parent

- The file is far above the direct leaf threshold.
- It owns multiple independent UI contracts with distinct consumers.
- It includes both breakpoint-only rules and page/component style contracts, so direct one-shot extraction would blur ownership.

## Initial Subchild Queue

- `frontend.design_system_styles.responsive_panel_overrides.workspace_editor_breakpoints`
- `frontend.design_system_styles.responsive_panel_overrides.runtime_event_research_panels`
- `frontend.design_system_styles.responsive_panel_overrides.motion_and_runtime_helpers`
- `frontend.design_system_styles.responsive_panel_overrides.tutorial_overlay_styles`
- `frontend.design_system_styles.responsive_panel_overrides.dashboard_and_strategy_config`
- `frontend.design_system_styles.responsive_panel_overrides.quantscript_editor_and_source_tabs`
- `frontend.design_system_styles.responsive_panel_overrides.workspace_debug_approval_print`
- `frontend.design_system_styles.responsive_panel_overrides.legacy_page_inline_migrations`

## Parent Return

- After this child parent closes, return to `frontend.design_system_styles`.
- Remaining parent queue after closeout:
  - `frontend.design_system_styles.page_style_contracts`

## Verification

- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
