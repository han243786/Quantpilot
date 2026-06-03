# Frontend Global Merge-Back Map

Status: prepared; do not execute merge-back until backend refactor closeout.

This map freezes the frontend-local governance truth for the future global integration step. It is a preparation artifact only. It must not be treated as permission to edit global governance files before the backend process is ready.

## Deferred Global Targets

- `markdown/00-matrix-governance/module-tree.md`
- `markdown/00-matrix-governance/guidance-matrix.md`
- `markdown/00-matrix-governance/process-matrix.md`
- `markdown/00-matrix-governance/standard-matrix.md`
- `markdown/10-overview/overview-full-feature-tree.md`
- `markdown/10-overview/overview-docs-index.md`
- `markdown/10-overview/overview-current-status-and-roadmap.md`

## Merge Trigger

Start this merge-back only after the backend refactor process has closed its current parent queue and explicitly asks to integrate frontend governance.

## Merge Order

1. Re-read backend current state and latest backend closeout records.
2. Re-read `frontend-recursive-state.json` and verify `current_child_queue` is empty and `next_parent` is `null`.
3. Merge `frontend-module-tree.md` into the global module tree under `root.frontend`.
4. Merge `frontend-full-feature-tree.md` into the global full feature tree as the frontend coverage supplement.
5. Add frontend proposal, standard, guidance, and process rules only where they define globally useful invariants.
6. Register deferred E2E cleanup as a post-backend integration task, not as a frontend refactor blocker.
7. Run global governance checks and the full frontend verification gate.

## Parent Mapping

| Frontend parent | Local closeout | Global module-tree placement | Global matrix impact | Global full-tree impact |
| --- | --- | --- | --- | --- |
| `frontend.app_shell` | `records/FE-0011-frontend-app-shell-parent-closeout.md` | `root.frontend -> app_shell` | Shell bootstrap and desktop/browser shell rules. | React root, app shell, global overlays, desktop title bar, route host. |
| `frontend.routing` | `records/FE-0016-frontend-routing-parent-closeout.md` | `root.frontend -> routing` | Route contract and navigation dispatch rules. | Router, route contract, shell navigation files and tests. |
| `frontend.api_client` | `records/FE-0021-frontend-api-client-parent-closeout.md` | `root.frontend -> api_client` | API base, transport, timeout, and error propagation rules. | API base, fetch helpers, API transport, client compatibility paths. |
| `frontend.capabilities` | `records/FE-0038-frontend-capabilities-parent-closeout.md` | `root.frontend -> capabilities` | Capability gating, safe fallback, module visibility, registry truth. | Capability support matrix, registry, built-in snapshots, module registry contracts. |
| `frontend.strategy_workspace` | `records/FE-0049-frontend-strategy-workspace-parent-closeout.md` | `root.frontend -> strategy_workspace` | Workspace whitebox page contracts and tab ownership. | Strategy workspace route, toolbar bridge, dashboards, cards, monitor/research/source tabs. |
| `frontend.strategy_hub` | `records/FE-0061-frontend-strategy-hub-parent-closeout.md` | `root.frontend -> strategy_hub` | Strategy directory, roster, activity, inspector, and template contracts. | Strategy hub page, roster, inspector, recent activity, template library. |
| `frontend.graph_editor` | `records/FE-0096-frontend-graph-editor-parent-closeout.md` | `root.frontend -> graph_editor` | Canvas, node, property panel, graph factory, compiler, QuantScript bridge rules. | Graph editor components, compiler helpers, validation, parser, editor-store action wrappers. |
| `frontend.runtime_panels` | `records/FE-0106-frontend-runtime-panels-parent-closeout.md` | `root.frontend -> runtime_panels` | Runtime panel display contracts and evidence surfaces. | Event stream, runtime diagnostics, reports, mutation controls, replay/explanations. |
| `frontend.backtest_views` | `records/FE-0120-frontend-backtest-views-parent-closeout.md` | `root.frontend -> backtest_views` | Backtest analysis, detail, compare, and shared layout contracts. | Backtest index, detail sections, compare sections, shared analysis layout. |
| `frontend.store` | `records/FE-0176-frontend-store-parent-closeout.md` | `root.frontend -> store` | Store facade, persistence, compile flow, runtime session/history, transport rules. | Graph store root, persistence, editor actions, compile flow, runtime session/history. |
| `frontend.design_system_styles` | `records/FE-0214-frontend-design-system-styles-parent-closeout.md` | `root.frontend -> design_system_styles` | Style entry, design token, shared primitive, responsive, and page style contracts. | Style entrypoint, design-system CSS, shared CSS, responsive CSS, page style partials. |
| `frontend.test_support` | `records/FE-0221-frontend-test-support-parent-closeout.md` | `root.frontend -> test_support` | Unit fixture, dev bridge, E2E harness, and E2E support fixture rules. | Vitest setup, test bridge, shared fixtures, E2E support helpers. |

## Guardrails

- Do not merge frontend-local records into backend milestone logs.
- Do not convert E2E spec body cleanup into a frontend merge blocker.
- Do not add release-transition shortcuts unless a developer explicitly opens release-transition work.
- Preserve parent-to-child communication rules unless the developer explicitly starts release preparation.

## Verification Expected During Merge-Back

- `npm.cmd run build` from `frontend`.
- `npm.cmd test` from `frontend`.
- Backend checks selected by the backend closeout owner.
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`.
- `powershell -ExecutionPolicy Bypass -File tools/check-matrix-governance.ps1`.
- Selected E2E smoke after backend routes and fixtures have been reconciled.
