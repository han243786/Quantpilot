# 前端 E2E 当前状态清单

状态：已准备；spec body 清理延后到后端重构收口之后。

这份清单记录当前 E2E 覆盖面，但不重组 spec。它用于未来全局整合阶段区分有效覆盖、历史遗留覆盖和依赖后端稳定后的测试债务。

## 支撑契约

| 支撑文件 | 当前职责 | 后续清理说明 |
| --- | --- | --- |
| `frontend/tests/e2e/support/apiHarness.js` | Playwright API mock harness，并通过受保护的 `**/api/**` fallback 捕获未 mock 请求。 | 除非后端整合需要真实服务路径，否则继续保留为共享 mock 边界。 |
| `frontend/tests/e2e/support/workspaceBootstrapMocks.js` | 共享 editor/workspace graph、history、mutation、report、experiment 启动 mock。 | 后端路由清理完成后，再对齐 endpoint shape。 |
| `frontend/tests/e2e/support/workspaceGraphFixture.js` | 通过前端 graph/compiler helper 构建已验证的 workspace graph fixture。 | 继续作为前端 graph fixture 所有者。 |
| `frontend/tests/e2e/support/analysisReviewFixtures.js` | 共享 visual/performance review graph，并提供 runtime/backtest mock。 | 只有出现新的独立 review 场景时，才考虑拆分。 |

## Spec 清单

| Spec | 当前覆盖 | 支撑 fixture | 后端/API 表面 | 后续动作 |
| --- | --- | --- | --- | --- |
| `frontend/tests/e2e/editor-capabilities-smoke.spec.js` | 能力同步成功、缓存 fallback、安全 fallback、compile/run/backtest 结构化拒绝。 | `apiHarness`、`workspaceBootstrapMocks`、capability/runtime fixtures。 | `/api/capabilities`、`/api/runtime/compile`、`/api/runtime/test-run`、`/api/runtime/backtest`。 | 保留；后端 capability 契约稳定后对齐 rejection payload。 |
| `frontend/tests/e2e/run-simulation.spec.js` | Simulation start、SSE/events 展示、artifact save、history refresh。 | `apiHarness`、`workspaceBootstrapMocks`、run success fixture。 | `/api/capabilities`、`/api/quantscript/formal/compile`、`/api/runtime/compile`、`/api/runtime/test-run`、`/api/runtime/runs`、`/api/runtime/runs/*`、`/api/runtime/runs/*/events`、`/api/runtime/runs/*/save`。 | 作为 runtime-run smoke 保留；后端 runtime route tree 最终确定后更新。 |
| `frontend/tests/e2e/run-backtest.spec.js` | Backtest start、history refresh、artifact save、detail route。 | `apiHarness`、`workspaceBootstrapMocks`、backtest success fixture。 | `/api/capabilities`、`/api/quantscript/formal/compile`、`/api/runtime/compile`、`/api/runtime/backtest`、`/api/runtime/backtests`、`/api/runtime/backtests/*`、`/api/runtime/backtests/*/save`。 | 作为 backtest smoke 保留；后端 backtest/detail artifact 稳定后更新。 |
| `frontend/tests/e2e/v4-runtime-contracts.spec.js` | Auth capability fallback、v4 strategy runtime 浏览器契约、v4 backtest artifact 契约。 | `apiHarness`、`workspaceBootstrapMocks`、capability/backtest fixtures。 | `/api/capabilities`、`/api/runtime/v4/run`、`/api/runtime/backtest`、`/api/runtime/compile`。 | 保留；后端 closeout 后与后端 v4 runtime contract 对齐。 |
| `frontend/tests/e2e/runtime-mutation-walkthrough.spec.js` | Runtime mutation proposal、safe window、activation、rollback 状态展示。 | `apiHarness`、`workspaceBootstrapMocks`、run fixture。 | `/api/capabilities`、`/api/runtime/runs/*`、`/api/runtime/mutations**`。 | 保留；与后端 runtime mutation 模块对齐 mutation record schema。 |
| `frontend/tests/e2e/evidence-contract-walkthrough.spec.js` | Backtest evidence timeline、replay paging、compact mode、runtime report 生命周期。 | `apiHarness`、`workspaceBootstrapMocks`、backtest fixture。 | `/api/capabilities`、`/api/runtime/backtests/*`、`/api/runtime/backtests/*/replay**`、`/api/runtime/reports`、`/api/runtime/reports/*`、`/api/runtime/reports/*/export`。 | 保留；大概率需要后端 report/replay 契约审计。 |
| `frontend/tests/e2e/visual-regression.spec.js` | Strategy hub、alerts、snapshots、runbook 的截图覆盖。 | `apiHarness`、`workspaceBootstrapMocks`、本地 visual fixtures。 | `/api/capabilities`、`/api/v1/alerts`、`/api/v1/snapshots`、`/api/v1/runbook`。 | 作为受控 visual suite 保留；只在后端与布局稳定后刷新截图。 |
| `frontend/tests/e2e/visual-responsive-review.spec.js` | Strategy hub、workspace、backtest detail、backtest compare 的响应式截图。 | `analysisReviewFixtures`。 | 通过 support helper mock graph/runtime/backtest/report 表面。 | 作为手动 visual review 保留；只有明确要求 visual review 时运行。 |
| `frontend/tests/e2e/perf-first-screen-review.spec.js` | Editor、backtest detail、backtest compare 的冷启动耗时。 | `analysisReviewFixtures`。 | 通过 support helper mock graph/runtime/backtest/report 表面。 | 由 `PERF_REVIEW` 门控保留；后端/前端整合后，只有明确要求性能 review 时运行。 |
| `frontend/tests/e2e/perf-react-flow-mount-review.spec.js` | React Flow full-node-card 与 staged-card 挂载耗时对照。 | `analysisReviewFixtures`。 | 通过 support helper mock graph/runtime/backtest/report 表面。 | 由 `PERF_REVIEW` 门控保留；不要并入默认 E2E smoke。 |
| `frontend/tests/e2e/scenario-test-v2.spec.js` | 历史宽场景：canvas 交互、test bridge、i18n/responsive、alerts/runbook/snapshots/chaos/approvals。 | 使用真实页面状态和直接 `page.request`；不使用共享 API harness。 | 硬编码 `http://127.0.0.1:3000/api/v1/alerts/rules` 和 `http://127.0.0.1:3000/api/v1/runbook`；页面路由 `/strategies`、`/alerts`、`/snapshots`、`/chaos`、`/approvals`。 | 后端收口后最高优先级清理：修复编码、按工作流拆分、替换硬编码后端 URL，并决定哪些覆盖继续保留。 |

## 延后清理优先级

1. 先规范化或替换 mojibake 严重的历史 E2E 文本，再把它当成规格事实使用。
2. 只有在后端路由所有权稳定后，才拆分宽场景。
3. 将直接 `127.0.0.1:3000` 调用替换为 `baseURL` 或共享 API harness。
4. 决定哪些 spec 进入默认 `npm.cmd run test:e2e`，哪些继续作为 visual/performance review 门控套件。
5. 只有在后端与布局整合稳定后，才刷新 visual snapshots。

## 当前命令

- 默认 Playwright：`npm.cmd run test:e2e`。
- Visual review：`npm.cmd run test:e2e:visual-review`。
- First-screen performance：`npm.cmd run test:perf:first-screen`。
- React Flow mount performance：`npm.cmd run test:perf:react-flow`。

## 暂不执行

- 后端 endpoint 所有权关闭前，不删除 spec body。
- 后端/前端整合稳定前，不更新 snapshots。
- 后端 closeout 打开全局整合前，不把这份清单合并进全局治理。
