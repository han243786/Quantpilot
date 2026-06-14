# 前端全局合并回填对照表

状态：已准备；后端重构收口前不得执行全局 merge-back。

这份对照表冻结了前端本地治理线在未来全局整合阶段需要使用的事实。它只是前置准备文档，不代表已经允许在后端进程准备好之前编辑全局治理文件。

## 延后处理的全局目标文件

- `markdown/00-matrix-governance/module-tree.md`
- `markdown/00-matrix-governance/guidance-matrix.md`
- `markdown/00-matrix-governance/process-matrix.md`
- `markdown/00-matrix-governance/standard-matrix.md`
- `markdown/10-overview/overview-full-feature-tree.md`
- `markdown/10-overview/overview-docs-index.md`
- `markdown/10-overview/overview-current-status-and-roadmap.md`

## 合并触发条件

只有在后端重构进程已经关闭当前父级队列，并且明确要求整合前端治理时，才能启动这次 merge-back。

## 合并顺序

1. 重新读取后端当前状态和最新后端 closeout 记录。
2. 重新读取 `frontend-recursive-state.json`，确认 `current_child_queue` 为空且 `next_parent` 为 `null`。
3. 将 `frontend-module-tree.md` 合并到全局模块树的 `root.frontend` 下。
4. 将 `frontend-full-feature-tree.md` 作为前端覆盖补充，合并到全局全量功能树。
5. 只把具备全局不变量意义的前端提案、规范、引导、流程规则合并进全局矩阵。
6. 将延后的 E2E 清理登记为后端完成后的整合任务，而不是前端重构阻塞项。
7. 运行全局治理校验和完整前端验证门槛。

## 父级映射

| 前端父级 | 本地 closeout | 全局模块树位置 | 全局矩阵影响 | 全局全量树影响 |
| --- | --- | --- | --- | --- |
| `frontend.app_shell` | `records/FE-0011-frontend-app-shell-parent-closeout.md` | `root.frontend -> app_shell` | Shell 启动、桌面/浏览器外壳规则。 | React 根节点、应用外壳、全局浮层、桌面标题栏、路由宿主。 |
| `frontend.routing` | `records/FE-0016-frontend-routing-parent-closeout.md` | `root.frontend -> routing` | 路由契约和导航分发规则。 | Router、路由契约、shell navigation 文件与测试。 |
| `frontend.api_client` | `records/FE-0021-frontend-api-client-parent-closeout.md` | `root.frontend -> api_client` | API base、transport、timeout、错误传播规则。 | API base、fetch helpers、API transport、client 兼容路径。 |
| `frontend.capabilities` | `records/FE-0038-frontend-capabilities-parent-closeout.md` | `root.frontend -> capabilities` | 能力门控、安全 fallback、模块可见性、registry 事实源。 | Capability support matrix、registry、内置快照、模块 registry 契约。 |
| `frontend.strategy_workspace` | `records/FE-0049-frontend-strategy-workspace-parent-closeout.md` | `root.frontend -> strategy_workspace` | 工作区白箱页面契约和标签页所有权。 | Strategy workspace 路由、toolbar bridge、dashboard、cards、monitor/research/source 标签页。 |
| `frontend.strategy_hub` | `records/FE-0061-frontend-strategy-hub-parent-closeout.md` | `root.frontend -> strategy_hub` | 策略目录、roster、activity、inspector、template 契约。 | Strategy hub 页面、roster、inspector、recent activity、template library。 |
| `frontend.graph_editor` | `records/FE-0096-frontend-graph-editor-parent-closeout.md` | `root.frontend -> graph_editor` | Canvas、node、property panel、graph factory、compiler、QuantScript bridge 规则。 | Graph editor 组件、compiler helpers、validation、parser、editor-store action wrappers。 |
| `frontend.runtime_panels` | `records/FE-0106-frontend-runtime-panels-parent-closeout.md` | `root.frontend -> runtime_panels` | Runtime 面板展示契约和 evidence surface。 | Event stream、runtime diagnostics、reports、mutation controls、replay/explanations。 |
| `frontend.backtest_views` | `records/FE-0120-frontend-backtest-views-parent-closeout.md` | `root.frontend -> backtest_views` | Backtest analysis、detail、compare、shared layout 契约。 | Backtest index、detail sections、compare sections、shared analysis layout。 |
| `frontend.store` | `records/FE-0176-frontend-store-parent-closeout.md` | `root.frontend -> store` | Store facade、persistence、compile flow、runtime session/history、transport 规则。 | Graph store root、persistence、editor actions、compile flow、runtime session/history。 |
| `frontend.design_system_styles` | `records/FE-0214-frontend-design-system-styles-parent-closeout.md` | `root.frontend -> design_system_styles` | Style entry、design token、shared primitive、responsive、page style 契约。 | Style entrypoint、design-system CSS、shared CSS、responsive CSS、page style partials。 |
| `frontend.test_support` | `records/FE-0221-frontend-test-support-parent-closeout.md` | `root.frontend -> test_support` | Unit fixture、dev bridge、E2E harness、E2E support fixture 规则。 | Vitest setup、test bridge、shared fixtures、E2E support helpers。 |

## 保护栏

- 不要把前端本地记录合并进后端里程碑日志。
- 不要把 E2E spec body 清理转换成前端 merge 阻塞项。
- 除非开发者明确开启 release-transition 工作，否则不要加入发布过渡快捷连接。
- 除非开发者明确启动发布准备，否则继续保持父子通信规则。

## merge-back 时预期验证项

- 在 `frontend` 下运行 `npm.cmd run build`。
- 在 `frontend` 下运行 `npm.cmd test`。
- 运行后端 closeout 负责人指定的后端检查。
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`。
- `powershell -ExecutionPolicy Bypass -File tools/check-matrix-governance.ps1`。
- 后端路由和 fixture 对齐后，运行选定的 E2E smoke。
