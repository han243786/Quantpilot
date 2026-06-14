# Frontend Guidance Matrix

Status: frontend-only navigation map, not yet merged into global guidance.

## Root

`root.frontend` is the isolated parent for frontend refactor governance.

## Initial Parent Candidates

These candidates are starting coordinates, not final authoritative modules.

| Candidate Node | Likely Scope | First Boundary Question |
| --- | --- | --- |
| `frontend.app_shell` | `App.jsx`, bootstrapping, providers, global layout. | What owns runtime shell initialization? |
| `frontend.routing` | Router definitions and route guards. | Which pages are route leaves and which are shared shells? |
| `frontend.api_client` | API calls, request/response adapters, error handling. | What is the public backend contract surface? |
| `frontend.capabilities` | Feature capability adapters and orchestration. | Which capabilities are reusable outside current pages? |
| `frontend.strategy_workspace` | Strategy editing, canvas/workspace workflows. | Which user workflows belong to the workspace parent? |
| `frontend.strategy_hub` | Strategy list, selection, templates, metadata. | Which list/detail actions are independent leaves? |
| `frontend.graph_editor` | Node graph, graph state, graph rendering. | Where is graph behavior separated from UI shell? |
| `frontend.runtime_panels` | Runtime/session/status panels. | Which panels depend on live runtime contracts? |
| `frontend.backtest_views` | Backtest views, charts, result inspection. | Which result projections can be isolated? |
| `frontend.governance_ops_pages` | Governance/ops/admin pages if present. | Which pages are frontend-only vs backend-contract driven? |
| `frontend.store` | State containers, selectors, reducers/actions. | Which domains deserve their own store boundary? |
| `frontend.design_system_styles` | Shared components, styles, tokens. | Which visual primitives are stable enough to share? |
| `frontend.test_support` | Test fixtures, mocks, render helpers. | Which frontend equivalence checks should be reusable? |

## Guidance Rule

Before a frontend proposal or extraction begins, name the candidate node in this file or add a new frontend-local node here. Do not edit the global guidance matrix until final merge-back.
