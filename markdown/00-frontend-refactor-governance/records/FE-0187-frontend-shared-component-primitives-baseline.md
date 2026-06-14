# FE-0187 - Frontend Shared Component Primitives Baseline

Status: baseline established.

## Child Parent Node

`frontend.design_system_styles.shared_component_primitives`

## Trigger Decision

This node must be treated as a child parent instead of a single leaf.

Rules triggered:

- Scale: `frontend/src/shared.css` has 310 lines and multiple named sections.
- Responsibility split: reusable `ad-*` primitives, legacy `qp-*` compatibility aliases, loading skeleton motion, and monthly heatmap table styles serve different consumers.
- Reuse surface: `ad-btn`, `ad-card`, and related classes are referenced broadly across app shell, components, and pages; `qp-*` aliases remain used by legacy operational pages; heatmap styles are page-analysis specific.
- Risk: mechanical CSS import splitting can preserve exact behavior, but collapsing these responsibilities into one closeout would keep a hidden mixed leaf.

## Current Owned And Split-Target Files

- `frontend/src/shared.css`

## Important Consumers

- `frontend/src/styleEntrypoint.js`
- `frontend/src/app/AppShellFallback.jsx`
- `frontend/src/pages/backtestViews/shared/MonthlyReturnsHeatmap.jsx`
- `frontend/src/components/*`
- `frontend/src/pages/*`

## Subchild Queue

- `frontend.design_system_styles.shared_component_primitives.ad_core_primitives`
- `frontend.design_system_styles.shared_component_primitives.legacy_qp_aliases`
- `frontend.design_system_styles.shared_component_primitives.loading_skeleton_motion`
- `frontend.design_system_styles.shared_component_primitives.monthly_heatmap_styles`

## Extraction Plan

- Keep `frontend/src/shared.css` as the shared-style aggregator.
- Extract contiguous CSS sections into `frontend/src/shared/*.css` partials in the same order.
- For each leaf, require expanded CSS equivalence against `HEAD:frontend/src/shared.css`, frontend build, recursive JSON parse, full feature tree check, matrix governance check, and `git diff --check`.

## Verification

- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
- From repo root, `git diff --check`: passed.
