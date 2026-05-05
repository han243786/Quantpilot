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
| supported（已支持） | 37 |
| trace_only（仅追踪） | 1 |

## 按系列汇总

| 系列 | 条目数 |
| --- | --- |
| compile_boundary（编译边界） | 3 |
| exchange（交易所） | 2 |
| execution_module（执行模块） | 1 |
| frontend_module（前端模块） | 14 |
| runtime_mode（运行模式） | 1 |
| strategy_ir_indicator_kind（策略 IR 指标类型） | 7 |
| symbol（交易对） | 3 |
| ui_action（UI 操作） | 6 |
| user_facing_claim（面向用户声明） | 7 |
| workspace_surface（工作区界面） | 4 |

## runtime_mode（运行模式）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| runtime.mode.paper | paper | supported | 后端运行时负责人 | 后端合约、编译/运行时检查 | backend:/api/capabilities.runtime.supported_modes |  |

## execution_module（执行模块）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| execution.module.builtin.execution.paper | builtin.execution.paper | supported | 后端运行时负责人 | 执行语义、能力响应 | backend:/api/capabilities.runtime.supported_execution_modules |  |

## exchange（交易所）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| market.exchange.binance | binance | supported | 后端市场数据负责人 | 市场边界、fixture、措辞 | backend:/api/capabilities.market_data.supported_exchanges |  |
| market.exchange.okx | okx | supported | 后端市场数据负责人 | 市场边界、fixture、措辞 | backend:/api/capabilities.market_data.supported_exchanges |  |

## symbol（交易对）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| market.symbol.BTCUSDT | BTCUSDT | supported | 后端市场数据负责人 | 市场边界、fixture、措辞 | backend:/api/capabilities.market_data.supported_symbols |  |
| market.symbol.ETHUSDT | ETHUSDT | supported | 后端市场数据负责人 | 市场边界、fixture、措辞 | backend:/api/capabilities.market_data.supported_symbols |  |
| market.symbol.SOLUSDT | SOLUSDT | supported | 后端市场数据负责人 | 市场边界、fixture、措辞 | backend:/api/capabilities.market_data.supported_symbols |  |

## strategy_ir_indicator_kind（策略 IR 指标类型）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| strategy_ir.indicator.ma_cross | ma_cross | supported | 后端编译负责人 | 降级边界、诊断 | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.rsi | rsi | supported | 后端编译负责人 | 降级边界、诊断 | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.macd | macd | supported | 后端编译负责人 | 降级边界、诊断 | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.momentum | momentum | supported | 后端编译负责人 | 降级边界、诊断 | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.spread | spread | restricted | 后端编译负责人 | 降级边界、诊断 | backend:/api/capabilities.strategy_ir.declared_indicator_kinds | Spread 存在于 beta 编译/运行时路径中，但不得被宣传为研究级的价差策略支持。 |
| strategy_ir.indicator.z_score | z_score | supported | 后端编译负责人 | 降级边界、诊断 | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.custom | custom | restricted | 后端编译负责人 | 降级边界、诊断 | backend:/api/capabilities.strategy_ir.declared_indicator_kinds | Custom 仅限于受限制的 Strategy IR 表达式路径，该路径降级为 Core IR。 |

## frontend_module（前端模块）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| frontend.module.builtin.data.kline | builtin.data.kline | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.data.quote | builtin.data.quote | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.double_ma | builtin.intent.double_ma | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.ma_deviation | builtin.intent.ma_deviation | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.rsi | builtin.intent.rsi | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.macd | builtin.intent.macd | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.momentum | builtin.intent.momentum | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.zscore | builtin.intent.zscore | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.spread_observer | builtin.intent.spread_observer | restricted | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys | 价差相关模块暴露仅为 beta 版本且必须携带显式边界说明。 |
| frontend.module.builtin.agent.weighted | builtin.agent.weighted | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.agent.arbitrage | builtin.agent.arbitrage | trace_only | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys | 模块键可能在 beta 代码路径中保持可见，但这不代表真正的套利平台支持。 |
| frontend.module.builtin.risk.global | builtin.risk.global | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.execution.paper | builtin.execution.paper | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.runtime.control | builtin.runtime.control | supported | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 | frontend:support-matrix.frontend.supportedModuleKeys |  |

## ui_action（UI 操作）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| ui.action.export_runtime_config | export_runtime_config | supported | 前端编辑器负责人 | 操作门禁、原因文本、E2E | frontend:CAPABILITY_ACTION_MAP | 只在当前策略图编译通过后导出图生成的 runtime_config。; 当前端正在同步后端能力快照或进入安全回退模式时，该操作会被锁定。 |
| ui.action.export_quantscript | export_quantscript | supported | 前端编辑器负责人 | 操作门禁、原因文本、E2E | frontend:CAPABILITY_ACTION_MAP | 只导出当前 strategy_graph 草稿，不依赖后端能力门禁，也不会替代 formal QuantScript 编译链路。 |
| ui.action.compile | compile | supported | 前端编辑器负责人 | 操作门禁、原因文本、E2E | frontend:CAPABILITY_ACTION_MAP | Strategy IR 只承担语义预检。; 运行时编译仍然是可运行输出的最终真源。 |
| ui.action.start_simulation | start_simulation | supported | 前端编辑器负责人 | 操作门禁、原因文本、E2E | frontend:CAPABILITY_ACTION_MAP | 当前 Beta 边界内仅支持纸面模拟运行时。; 缓存回退模式下仍可见，但依旧受后端校验约束。 |
| ui.action.run_backtest | run_backtest | supported | 前端编辑器负责人 | 操作门禁、原因文本、E2E | frontend:CAPABILITY_ACTION_MAP | 当前仅提供基础回放/回测支持，不宣称研究级回测能力。; 缓存回退模式下仍可见，但依旧受后端校验约束。 |
| ui.action.run_parameter_sweep | run_parameter_sweep | supported | 前端编辑器负责人 | 操作门禁、原因文本、E2E | frontend:CAPABILITY_ACTION_MAP | 参数扫掠建立在现有回测能力边界之上，能力未同步或 safe fallback 时不得继续暴露为可执行入口。; 该入口只表示窄执行假设扫描，不表示通用优化器或第二套实验运行时。 |

## workspace_surface（工作区界面）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| workspace.surface.template_library | template_library | supported | 前端编辑器负责人 | 工作区暴露、后端路由诚实性、收口审计 | frontend:support-matrix.workspace.surfaces.template_library | 模板库是前端本地 starter graph 入口，不依赖 /api/capabilities 显隐。; 加载模板只替换当前内存工作草稿，不创建第二套后端模板传输。 |
| workspace.surface.version_history | version_history | supported | 前端编辑器负责人 | 工作区暴露、后端路由诚实性、收口审计 | frontend:support-matrix.workspace.surfaces.version_history | 版本历史属于图持久化工作流，不由 /api/capabilities 决定显隐。; 可见不代表扩展了新的 runtime capability，只代表当前图版本工件可管理。 |
| workspace.surface.collaboration_audit | collaboration_audit | supported | 前端编辑器负责人 | 工作区暴露、后端路由诚实性、收口审计 | frontend:support-matrix.workspace.surfaces.collaboration_audit | 协作与审计属于当前图元数据和审计记录投影，不由 /api/capabilities 决定显隐。; 当前边界仍是本地 actor 协作切片，不应外推成远程账号系统能力。 |
| workspace.surface.parameter_sweep | parameter_sweep | restricted | 前端编辑器负责人 | 工作区暴露、后端路由诚实性、收口审计 | frontend:support-matrix.workspace.surfaces.parameter_sweep | 参数扫掠是现有 backtest surface 上的窄执行假设扫描，不是第二套实验运行时。; 发起扫掠必须遵守与回测相同的 capability 同步和 safe-fallback 锁定规则。 |

## compile_boundary（编译边界）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| compile.strategy_ir_preflight | strategy_ir | restricted | 后端编译负责人 | 降级边界、诊断 | frontend:support-matrix.compile.preflightArtifact | 仅语义预检。它不决定可运行输出。 |
| compile.formal_quantscript_lowering | quantscript.formal_source | restricted | 后端编译负责人 | 降级边界、诊断 | frontend:support-matrix.compile.boundaryNotes | 存在时拥有运行时降级权，但运行时编译仍决定可运行输出。 |
| compile.runtime_source_of_truth | /api/runtime/compile | supported | 后端编译负责人 | 后端合约、编译/运行时检查 | frontend:support-matrix.compile.runtimeSourceOfTruth | 当工件不一致时，运行时行为遵循此真实数据源。 |

## user_facing_claim（面向用户声明）

| ID | 值 | 类别 | 负责人角色 | 审查责任 | 真实数据源 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| claim.allowed.纸面运行时_Beta | 纸面运行时 Beta | supported | 文档和 QA 负责人 | README、markdown、UI 文案、文本门禁 | frontend:support-matrix.userFacingGuardrails.allowedClaims |  |
| claim.allowed.基础回测支持 | 基础回测支持 | supported | 文档和 QA 负责人 | README、markdown、UI 文案、文本门禁 | frontend:support-matrix.userFacingGuardrails.allowedClaims |  |
| claim.allowed.受限的_Custom_Strategy_IR_表达式路径 | 受限的 Custom Strategy IR 表达式路径 | supported | 文档和 QA 负责人 | README、markdown、UI 文案、文本门禁 | frontend:support-matrix.userFacingGuardrails.allowedClaims |  |
| claim.disallowed.claiming_research-grade_backtest_support | 宣称具备研究级回测能力 | disallowed_claim | 文档和 QA 负责人 | README、markdown、UI 文案、文本门禁 | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
| claim.disallowed.claiming_live_trading_support | 宣称支持实盘交易 | disallowed_claim | 文档和 QA 负责人 | README、markdown、UI 文案、文本门禁 | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
| claim.disallowed.claiming_true_arbitrage_agent_support | 宣称支持真实套利代理 | disallowed_claim | 文档和 QA 负责人 | README、markdown、UI 文案、文本门禁 | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
| claim.disallowed.claiming_third-party_plugin_marketplace_support | 宣称支持第三方插件市场 | disallowed_claim | 文档和 QA 负责人 | README、markdown、UI 文案、文本门禁 | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
