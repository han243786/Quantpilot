# Frontend E2E Current State Inventory

Status: prepared; spec-body cleanup deferred until backend refactor closeout.

This inventory records the current E2E surface without reorganizing specs. It exists so the future global integration step can distinguish useful coverage from legacy or backend-dependent test debt.

## Support Contracts

| Support file | Current role | Future cleanup note |
| --- | --- | --- |
| `frontend/tests/e2e/support/apiHarness.js` | Playwright API mock harness with guarded `**/api/**` fallback. | Keep as the shared mock boundary unless backend integration requires a real-server path. |
| `frontend/tests/e2e/support/workspaceBootstrapMocks.js` | Shared editor/workspace graph, history, mutation, report, experiment bootstrap mocks. | Reconcile endpoint shapes after backend route cleanup. |
| `frontend/tests/e2e/support/workspaceGraphFixture.js` | Builds validated workspace graph fixture through frontend graph/compiler helpers. | Keep as frontend graph fixture owner. |
| `frontend/tests/e2e/support/analysisReviewFixtures.js` | Shared visual/performance review graph plus runtime/backtest mocks. | Split only if new independent review scenarios appear. |

## Spec Inventory

| Spec | Coverage now | Support fixtures | Backend/API surface | Future action |
| --- | --- | --- | --- | --- |
| `frontend/tests/e2e/editor-capabilities-smoke.spec.js` | Capability success, cached fallback, safe fallback, compile/run/backtest structured rejection. | `apiHarness`, `workspaceBootstrapMocks`, capability/runtime fixtures. | `/api/capabilities`, `/api/runtime/compile`, `/api/runtime/test-run`, `/api/runtime/backtest`. | Keep; reconcile rejection payloads after backend capability contracts settle. |
| `frontend/tests/e2e/run-simulation.spec.js` | Simulation start, SSE/events display, artifact save, history refresh. | `apiHarness`, `workspaceBootstrapMocks`, run success fixture. | `/api/capabilities`, `/api/quantscript/formal/compile`, `/api/runtime/compile`, `/api/runtime/test-run`, `/api/runtime/runs`, `/api/runtime/runs/*`, `/api/runtime/runs/*/events`, `/api/runtime/runs/*/save`. | Keep as runtime-run smoke; update after backend runtime route tree is finalized. |
| `frontend/tests/e2e/run-backtest.spec.js` | Backtest start, history refresh, artifact save, detail route. | `apiHarness`, `workspaceBootstrapMocks`, backtest success fixture. | `/api/capabilities`, `/api/quantscript/formal/compile`, `/api/runtime/compile`, `/api/runtime/backtest`, `/api/runtime/backtests`, `/api/runtime/backtests/*`, `/api/runtime/backtests/*/save`. | Keep as backtest smoke; update after backend backtest/detail artifacts stabilize. |
| `frontend/tests/e2e/v4-runtime-contracts.spec.js` | Auth capability fallback, v4 strategy runtime browser contract, v4 backtest artifact contract. | `apiHarness`, `workspaceBootstrapMocks`, capability/backtest fixtures. | `/api/capabilities`, `/api/runtime/v4/run`, `/api/runtime/backtest`, `/api/runtime/compile`. | Keep; align with backend v4 runtime contract after backend closeout. |
| `frontend/tests/e2e/runtime-mutation-walkthrough.spec.js` | Runtime mutation proposal, safe window, activation, rollback state display. | `apiHarness`, `workspaceBootstrapMocks`, run fixture. | `/api/capabilities`, `/api/runtime/runs/*`, `/api/runtime/mutations**`. | Keep; reconcile mutation record schema with backend runtime mutation module. |
| `frontend/tests/e2e/evidence-contract-walkthrough.spec.js` | Backtest evidence timeline, replay paging, compact mode, runtime report lifecycle. | `apiHarness`, `workspaceBootstrapMocks`, backtest fixture. | `/api/capabilities`, `/api/runtime/backtests/*`, `/api/runtime/backtests/*/replay**`, `/api/runtime/reports`, `/api/runtime/reports/*`, `/api/runtime/reports/*/export`. | Keep; likely needs backend report/replay contract audit. |
| `frontend/tests/e2e/visual-regression.spec.js` | Snapshot coverage for strategy hub, alerts, snapshots, runbook. | `apiHarness`, `workspaceBootstrapMocks`, local visual fixtures. | `/api/capabilities`, `/api/v1/alerts`, `/api/v1/snapshots`, `/api/v1/runbook`. | Keep gated visual suite; refresh screenshots only after backend and layout are stable. |
| `frontend/tests/e2e/visual-responsive-review.spec.js` | Responsive screenshots for strategy hub, workspace, backtest detail, backtest compare. | `analysisReviewFixtures`. | Mocked graph/runtime/backtest/report surfaces through support helper. | Keep as manual visual review; run only when visual review is explicitly requested. |
| `frontend/tests/e2e/perf-first-screen-review.spec.js` | Cold-start timing for editor, backtest detail, and backtest compare. | `analysisReviewFixtures`. | Mocked graph/runtime/backtest/report surfaces through support helper. | Keep gated by `PERF_REVIEW`; run after backend/frontend integration only if performance review is requested. |
| `frontend/tests/e2e/perf-react-flow-mount-review.spec.js` | React Flow full-node-card vs staged-card mount timing. | `analysisReviewFixtures`. | Mocked graph/runtime/backtest/report surfaces through support helper. | Keep gated by `PERF_REVIEW`; do not fold into normal E2E smoke. |
| `frontend/tests/e2e/scenario-test-v2.spec.js` | Legacy broad scenario for canvas interactions, test bridge, i18n/responsive, alerts/runbook/snapshots/chaos/approvals. | Uses real page state and direct `page.request`; no shared API harness. | Hard-coded `http://127.0.0.1:3000/api/v1/alerts/rules` and `http://127.0.0.1:3000/api/v1/runbook`; page routes `/strategies`, `/alerts`, `/snapshots`, `/chaos`, `/approvals`. | Highest cleanup priority after backend closeout: fix encoding, split by workflow, replace hard-coded backend URLs, and decide which coverage survives. |

## Deferred Cleanup Priorities

1. Normalize or replace mojibake-heavy legacy E2E text before using it as specification truth.
2. Split broad legacy scenarios only after backend route ownership is stable.
3. Replace direct `127.0.0.1:3000` calls with `baseURL` or the shared API harness.
4. Decide which specs run in default `npm.cmd run test:e2e` and which remain gated visual/performance review suites.
5. Refresh visual snapshots only after backend and layout integration is stable.

## Current Commands

- Default Playwright: `npm.cmd run test:e2e`.
- Visual review: `npm.cmd run test:e2e:visual-review`.
- First-screen performance: `npm.cmd run test:perf:first-screen`.
- React Flow mount performance: `npm.cmd run test:perf:react-flow`.

## Do Not Do Yet

- Do not delete spec bodies before backend endpoint ownership is closed.
- Do not update snapshots before backend/frontend integration is stable.
- Do not merge this inventory into global governance until backend closeout opens global integration.
