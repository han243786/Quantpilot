# 生成的能力治理注册表

此文件由 `frontend/src/capabilities/capabilityGovernance.js` 生成。
请勿手动编辑。

模式版本：`quantpilot/capability-governance/v1`

使用以下命令重新生成此快照：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1 -WriteSnapshot
```

## 按类别汇总

| 类别 | 条目数 |
| --- | --- |
| disallowed_claim（禁止声明） | 4 |
| restricted（受限） | 6 |
| supported（已支持） | 65 |
| trace_only（仅追踪） | 1 |

## 按系列汇总

| 系列 | 条目数 |
| --- | --- |
| compile_boundary（编译边界） | 3 |
| exchange（交易所） | 2 |
| execution_module（执行模块） | 2 |
| frontend_module（前端模块） | 16 |
| runtime_mode（运行模式） | 1 |
| strategy_ir_indicator_kind（策略 IR 指标类型） | 18 |
| symbol（交易对） | 3 |
| ui_action（UI 操作） | 14 |
| user_facing_claim（面向用户声明） | 7 |
| workspace_surface（工作区界面） | 10 |

## runtime_mode（运行模式）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| runtime.mode.paper | paper | supported | backend runtime owner | backend contract, compile/runtime checks | backend:/api/capabilities.runtime.supported_modes |  |

## execution_module（执行模块）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| execution.module.builtin.execution.paper | builtin.execution.paper | supported | backend runtime owner | execution semantics, capability response | backend:/api/capabilities.runtime.supported_execution_modules |  |
| execution.module.live.okx | live.okx | supported | backend runtime owner | execution semantics, capability response | backend:/api/capabilities.runtime.supported_execution_modules |  |

## exchange（交易所）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| market.exchange.binance | binance | supported | backend market-data owner | market boundary, fixtures, wording | backend:/api/capabilities.market_data.supported_exchanges |  |
| market.exchange.okx | okx | supported | backend market-data owner | market boundary, fixtures, wording | backend:/api/capabilities.market_data.supported_exchanges |  |

## symbol（交易对）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| market.symbol.BTCUSDT | BTCUSDT | supported | backend market-data owner | market boundary, fixtures, wording | backend:/api/capabilities.market_data.supported_symbols |  |
| market.symbol.ETHUSDT | ETHUSDT | supported | backend market-data owner | market boundary, fixtures, wording | backend:/api/capabilities.market_data.supported_symbols |  |
| market.symbol.SOLUSDT | SOLUSDT | supported | backend market-data owner | market boundary, fixtures, wording | backend:/api/capabilities.market_data.supported_symbols |  |

## strategy_ir_indicator_kind（策略 IR 指标类型）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| strategy_ir.indicator.ma_cross | ma_cross | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.rsi | rsi | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.macd | macd | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.momentum | momentum | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.spread | spread | restricted | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds | Spread exists in the beta compile/runtime path but must not be marketed as research-grade spread strategy support. |
| strategy_ir.indicator.z_score | z_score | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.custom | custom | restricted | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds | Custom is limited to the restricted Strategy IR expression path that lowers into Core IR. |
| strategy_ir.indicator.quote_observe | quote_observe | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.atr | atr | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.bollinger_bands | bollinger_bands | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.obv | obv | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.cmf | cmf | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.adx | adx | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.stochastic | stochastic | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.cci | cci | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.parabolic_sar | parabolic_sar | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.keltner_channel | keltner_channel | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.donchian_channel | donchian_channel | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |

## frontend_module（前端模块）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| frontend.module.builtin.data.kline | builtin.data.kline | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.builtin.data.quote | builtin.data.quote | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.builtin.intent.double_ma | builtin.intent.double_ma | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.builtin.intent.ma_deviation | builtin.intent.ma_deviation | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.builtin.intent.rsi | builtin.intent.rsi | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.builtin.intent.macd | builtin.intent.macd | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.builtin.intent.momentum | builtin.intent.momentum | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.builtin.intent.zscore | builtin.intent.zscore | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.builtin.intent.spread_observer | builtin.intent.spread_observer | restricted | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support | Spread-related module exposure is beta-only and must carry explicit boundary notes. |
| frontend.module.builtin.agent.weighted | builtin.agent.weighted | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.builtin.agent.arbitrage | builtin.agent.arbitrage | trace_only | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support | 该模块键可能在 Beta 代码路径中保持可见，但这并不代表真正的套利平台支持。 |
| frontend.module.builtin.risk.global | builtin.risk.global | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.builtin.execution.paper | builtin.execution.paper | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.builtin.runtime.control | builtin.runtime.control | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.v4.machine.param | v4.machine.param | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |
| frontend.module.v4.transition.guard | v4.transition.guard | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | backend:/api/capabilities.frontend.module_support |  |

## ui_action（UI 操作）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| ui.action.open_tutorial | open_tutorial | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 教程入口是本地前端辅助面板，但其可见和可点击状态仍由后端 ui_actions 声明。 |
| ui.action.manage_credentials | manage_credentials | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 凭证面板只管理交易提供方凭证，不代表实盘执行已开放。 |
| ui.action.reset_graph | reset_graph | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 新建策略图只重置本地草稿，入口可用性由后端 ui_actions 声明。 |
| ui.action.load_latest_graph | load_latest_graph | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 加载最新策略图属于图持久化读取路径。 |
| ui.action.save_graph | save_graph | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 保存策略图属于图持久化写入路径，不代表运行时写入。 |
| ui.action.export_runtime_config | export_runtime_config | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 只在当前策略图编译通过后导出图生成的 runtime_config。; 当前端正在同步后端能力快照或进入安全回退模式时，该操作会被锁定。 |
| ui.action.export_quantscript | export_quantscript | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 只导出当前 strategy_graph 草稿，不依赖后端能力门禁，也不会替代 formal QuantScript 编译链路。 |
| ui.action.compile | compile | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 策略中间表示只承担语义预检。; 运行时编译仍然是可运行输出的最终真源。 |
| ui.action.start_v4_simulation | start_v4_simulation | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | v4 模拟运行只接收 v4 QS 静态审计通过后的 machine graph handoff。; 嵌套状态机当前为 beta，深度上限为 2，并必须输出复杂度预算与层级 evidence。; 该入口固定使用 PaperSimulated，本地模拟成交不会连接 provider submission。 |
| ui.action.run_backtest | run_backtest | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | v4 backtest uses /api/runtime/backtest with runtime_kind=v4 and exposes v4_artifact evidence without enabling provider submission.; v4.5.0 adds beta tick_replay artifacts, advanced simulated order evidence, and microstructure metrics under the same PaperSimulated boundary.; 当前仅提供基础回放/回测支持，不宣称研究级回测能力。; 缓存回退模式下仍可见，但依旧受后端校验约束。 |
| ui.action.stop_runtime | stop_runtime | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 停止入口只对当前运行中会话可用。 |
| ui.action.reset_runtime | reset_runtime | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 重置运行时清理前端运行态投影和连接状态。 |
| ui.action.open_backtests | open_backtests | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 打开回测进入回测列表视图，不直接触发回测写入。 |
| ui.action.run_parameter_sweep | run_parameter_sweep | supported | frontend editor owner | action gating, reason text, E2E | backend:/api/capabilities.ui_actions.actions | 参数扫掠建立在现有回测能力边界之上，能力未同步或 safe fallback 时不得继续暴露为可执行入口。; 该入口只表示窄执行假设扫描，不表示通用优化器或第二套实验运行时。 |

## workspace_surface（工作区界面）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| workspace.surface.dashboard | dashboard | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | backend:/api/capabilities.workspace.surfaces | 工作区总览入口由后端 capability 快照决定是否可见和可点击。; 前端只保留排序、标签和布局投影。 |
| workspace.surface.code | code | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | backend:/api/capabilities.workspace.surfaces | 构建工作区承载图编辑、诊断和源码审查入口。; 入口可用性来自后端 workspace surface 声明。 |
| workspace.surface.diagnostics | diagnostics | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | backend:/api/capabilities.workspace.surfaces | 诊断工作区由问题队列和编译诊断触发，不一定作为一级标签展示。; 程序化导航仍必须通过后端 workspace surface 声明。 |
| workspace.surface.research | research | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | backend:/api/capabilities.workspace.surfaces | 研究回测入口仅代表当前基础回放/回测工作区。; 不得外推为研究级回测平台。 |
| workspace.surface.monitor | monitor | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | backend:/api/capabilities.workspace.surfaces | 运行监控入口展示运行时只读投影和事件流摘要。; 入口可用性必须跟随后端 workspace surface。 |
| workspace.surface.source | source | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | backend:/api/capabilities.workspace.surfaces | 源码工作区仅投影当前图谱源码和 Strategy IR 审查材料。; 可见不代表绕过正式编译链路。 |
| workspace.surface.template_library | template_library | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | backend:/api/capabilities.workspace.surfaces | 模板库是前端本地 starter graph 入口，但入口显隐仍必须由 /api/capabilities 声明。; 加载模板只替换当前内存工作草稿，不创建第二套后端模板传输。 |
| workspace.surface.version_history | version_history | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | backend:/api/capabilities.workspace.surfaces | 版本历史属于图持久化工作流，入口显隐由 /api/capabilities 决定。; 可见不代表扩展了新的 runtime capability，只代表当前图版本工件可管理。 |
| workspace.surface.collaboration_audit | collaboration_audit | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | backend:/api/capabilities.workspace.surfaces | 协作与审计属于当前图元数据和审计记录投影，入口显隐由 /api/capabilities 决定。; 当前边界仍是本地 actor 协作切片，不应外推成远程账号系统能力。 |
| workspace.surface.parameter_sweep | parameter_sweep | restricted | frontend editor owner | workspace exposure, backend route honesty, closeout audit | backend:/api/capabilities.workspace.surfaces | 参数扫掠是现有 backtest surface 上的窄执行假设扫描，不是第二套实验运行时。; 发起扫掠必须遵守与回测相同的 capability 同步和 safe-fallback 锁定规则。 |

## compile_boundary（编译边界）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| compile.strategy_ir_preflight | strategy_ir | restricted | backend compile owner | lowering boundary, diagnostics | frontend:support-matrix.compile.preflightArtifact | Semantic preflight only. It does not decide runnable output. |
| compile.formal_quantscript_lowering | quantscript.formal_source | restricted | backend compile owner | lowering boundary, diagnostics | frontend:support-matrix.compile.boundaryNotes | Owns runtime lowering when present, but runtime compile still decides runnable output. |
| compile.runtime_source_of_truth | /api/runtime/compile | supported | backend compile owner | backend contract, compile/runtime checks | frontend:support-matrix.compile.runtimeSourceOfTruth | When artifacts disagree, runtime behavior follows this source of truth. |

## user_facing_claim（面向用户声明）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| claim.allowed.纸面运行时_Beta | 纸面运行时 Beta | supported | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.allowedClaims |  |
| claim.allowed.基础回测支持 | 基础回测支持 | supported | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.allowedClaims |  |
| claim.allowed.受限的_Custom_策略中间表示表达式路径 | 受限的 Custom 策略中间表示表达式路径 | supported | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.allowedClaims |  |
| claim.disallowed.claiming_research-grade_backtest_support | 宣称具备研究级回测能力 | disallowed_claim | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
| claim.disallowed.claiming_live_trading_support | 宣称支持实盘交易 | disallowed_claim | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
| claim.disallowed.claiming_true_arbitrage_agent_support | 宣称支持真实套利代理 | disallowed_claim | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
| claim.disallowed.claiming_third-party_plugin_marketplace_support | 宣称支持第三方插件市场 | disallowed_claim | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
