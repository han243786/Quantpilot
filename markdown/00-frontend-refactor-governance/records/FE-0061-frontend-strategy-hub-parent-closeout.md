# FE-0061 Frontend Strategy Hub Parent Closeout

Status: closed.

## Parent Node

`frontend.strategy_hub`

## Closed Leaves

- `frontend.strategy_hub.route_shell`
- `frontend.strategy_hub.directory_model`
- `frontend.strategy_hub.hero_summary`
- `frontend.strategy_hub.roster_projection`
- `frontend.strategy_hub.roster_row_actions`
- `frontend.strategy_hub.inspector_projection`
- `frontend.strategy_hub.recent_activity_compare`
- `frontend.strategy_hub.template_library`
- `frontend.strategy_hub.shared_component_boundary`
- `frontend.strategy_hub.layout_styles`

## Final Parent Boundary

`frontend.strategy_hub` now owns the strategy management route shell, directory model, hero summary, roster projection/actions, inspector projection/actions, recent activity and compare queue surfaces, template-library UI, shared-boundary migration, and split strategy hub style entry.

## Whitebox Contract

### Public Inputs

- Graph store graph metadata, graph index, runtime history, backtest history, compare selections, and refresh/load/delete/reveal actions.
- Router helpers for opening strategy workspaces, strategy backtests, backtest detail, and compare routes.
- Strategy template definitions and template load actions.
- Browser confirmation and tutorial open event.
- Strategy hub CSS import through `StrategyHubPage.jsx`.
- Neutral shared components and formatters for cross-parent consumers.

### Public Outputs

- Route-owned strategy hub page shell and lazy-loaded hub sections.
- Strategy directory model for tracked strategy files, selected strategy state, activity timeline, compare queue, and hub summary.
- Hero/status-strip metrics and workspace entry actions.
- Template library rendering and template load interaction.
- Roster toolbar, roster rows, row action groups, and row action dispatch.
- Inspector overview, next-move guidance, recent backtests, recent runs, and compare queue actions.
- Neutral shared component and formatter boundaries used by hub, workspace, generic components, and backtest-analysis consumers.
- Ordered style entry through `frontend/src/pages/strategy-hub.css` and its split CSS leaves.

## Preserved Behavior

- Strategy hub routing still enters through `StrategyHubPage.jsx`.
- Existing hub layout, activity panels, workspace entry actions, roster row actions, template library loading, inspector projections, compare queue, and cross-parent shared component consumers remain available through their original public components.
- The page-level CSS import remains stable while strategy hub style rules are split behind the same import path.
- Compatibility aliases keep global documentation path checks stable without requiring edits to forbidden hot global files during isolated frontend recursion.

## Further-Split Decision

No further split is useful inside `frontend.strategy_hub` now. All planned child leaves are closed, shared component leakage is neutralized, and the large CSS file is split behind the stable page import. Additional splitting should wait for a concrete change request, visual regression, or a new strategy-hub feature boundary.

## Verification

- From `frontend/`, parent anchor test set: passed, 16 files / 46 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Parent Candidate

`frontend.graph_editor`
