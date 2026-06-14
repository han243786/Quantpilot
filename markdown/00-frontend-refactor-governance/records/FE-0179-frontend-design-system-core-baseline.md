# FE-0179 - Frontend Design System Core Baseline

Status: baseline established.

## Scope

- Parent node: `frontend.design_system_styles`
- Active child parent: `frontend.design_system_styles.design_tokens_and_native_controls`
- This is a docs-only recursive baseline for the core design-system stylesheet.

## Owned Files

- `frontend/src/design-system.css`

## Whitebox Boundary

- Inputs:
  - Global CSS entry order from `frontend/src/styleEntrypoint.js`.
  - CSS custom properties consumed by app shell, routes, page styles, shared primitives, React Flow-adjacent surfaces, and legacy `qp-*`/unprefixed aliases.
  - Native form controls and button elements styled by global defaults.
- Processing:
  - Establish base reset and body/root sizing.
  - Declare dark/light theme tokens and legacy alias variables.
  - Apply native controls, focus/selection, scrollbars, app chrome, sidebar, workspace chrome, command surfaces, overlay, panel divider, and reduced-motion rules.
- Outputs:
  - Stable `--ad-*`, legacy unprefixed, and `--qp-*` token contracts.
  - Global native element defaults.
  - Design-system-level app shell and chrome class contracts prefixed with `ad-`.
- Parent communication:
  - `frontend/src/design-system.css` is imported only by `frontend/src/styleEntrypoint.js`.
  - Subchildren must preserve import order through the `design-system.css` parent aggregator.

## Recursive Child Queue

- `frontend.design_system_styles.design_tokens_and_native_controls.reset_and_native_controls`
- `frontend.design_system_styles.design_tokens_and_native_controls.theme_tokens_and_aliases`
- `frontend.design_system_styles.design_tokens_and_native_controls.focus_selection_scrollbars`
- `frontend.design_system_styles.design_tokens_and_native_controls.shell_chrome_styles`
- `frontend.design_system_styles.design_tokens_and_native_controls.workspace_navigation_primitives`
- `frontend.design_system_styles.design_tokens_and_native_controls.overlays_resizers_motion`

## Split Decision

- This child is worth recursive split.
- Hard-rule assessment:
  - The stylesheet mixes reset, native element policy, token contracts, app chrome, navigation primitives, overlays, and motion behavior.
  - The file has multiple public contracts that change for unrelated reasons: CSS variables, native controls, shell layout classes, workspace/tab classes, and overlay/resizer classes.
  - A future token edit should not require reading command palette or sidebar rules.
  - Exact CSS equivalence can be checked by concatenating split partials in original source order and by frontend build.

## Verification

- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
