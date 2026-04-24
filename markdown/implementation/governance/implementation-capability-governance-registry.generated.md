# Generated Capability Governance Registry

This file is generated from `frontend/src/capabilities/capabilityGovernance.js`.
Do not edit it by hand.

Schema version: `quantpilot/capability-governance/v1`

Regenerate this snapshot with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1 -WriteSnapshot
```

## Summary By Class

| Class | Entry Count |
| --- | --- |
| disallowed_claim | 4 |
| restricted | 6 |
| supported | 37 |
| trace_only | 1 |

## Summary By Family

| Family | Entry Count |
| --- | --- |
| compile_boundary | 3 |
| exchange | 2 |
| execution_module | 1 |
| frontend_module | 14 |
| runtime_mode | 1 |
| strategy_ir_indicator_kind | 7 |
| symbol | 3 |
| ui_action | 6 |
| user_facing_claim | 7 |
| workspace_surface | 4 |

## runtime_mode

| ID | Value | Class | Owner Role | Review Responsibility | Source Of Truth | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| runtime.mode.paper | paper | supported | backend runtime owner | backend contract, compile/runtime checks | backend:/api/capabilities.runtime.supported_modes |  |

## execution_module

| ID | Value | Class | Owner Role | Review Responsibility | Source Of Truth | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| execution.module.builtin.execution.paper | builtin.execution.paper | supported | backend runtime owner | execution semantics, capability response | backend:/api/capabilities.runtime.supported_execution_modules |  |

## exchange

| ID | Value | Class | Owner Role | Review Responsibility | Source Of Truth | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| market.exchange.binance | binance | supported | backend market-data owner | market boundary, fixtures, wording | backend:/api/capabilities.market_data.supported_exchanges |  |
| market.exchange.okx | okx | supported | backend market-data owner | market boundary, fixtures, wording | backend:/api/capabilities.market_data.supported_exchanges |  |

## symbol

| ID | Value | Class | Owner Role | Review Responsibility | Source Of Truth | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| market.symbol.BTCUSDT | BTCUSDT | supported | backend market-data owner | market boundary, fixtures, wording | backend:/api/capabilities.market_data.supported_symbols |  |
| market.symbol.ETHUSDT | ETHUSDT | supported | backend market-data owner | market boundary, fixtures, wording | backend:/api/capabilities.market_data.supported_symbols |  |
| market.symbol.SOLUSDT | SOLUSDT | supported | backend market-data owner | market boundary, fixtures, wording | backend:/api/capabilities.market_data.supported_symbols |  |

## strategy_ir_indicator_kind

| ID | Value | Class | Owner Role | Review Responsibility | Source Of Truth | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| strategy_ir.indicator.ma_cross | ma_cross | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.rsi | rsi | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.macd | macd | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.momentum | momentum | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.spread | spread | restricted | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds | Spread exists in the beta compile/runtime path but must not be marketed as research-grade spread strategy support. |
| strategy_ir.indicator.z_score | z_score | supported | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds |  |
| strategy_ir.indicator.custom | custom | restricted | backend compile owner | lowering boundary, diagnostics | backend:/api/capabilities.strategy_ir.declared_indicator_kinds | Custom is limited to the restricted Strategy IR expression path that lowers into Core IR. |

## frontend_module

| ID | Value | Class | Owner Role | Review Responsibility | Source Of Truth | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| frontend.module.builtin.data.kline | builtin.data.kline | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.data.quote | builtin.data.quote | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.double_ma | builtin.intent.double_ma | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.ma_deviation | builtin.intent.ma_deviation | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.rsi | builtin.intent.rsi | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.macd | builtin.intent.macd | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.momentum | builtin.intent.momentum | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.zscore | builtin.intent.zscore | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.intent.spread_observer | builtin.intent.spread_observer | restricted | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys | Spread-related module exposure is beta-only and must carry explicit boundary notes. |
| frontend.module.builtin.agent.weighted | builtin.agent.weighted | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.agent.arbitrage | builtin.agent.arbitrage | trace_only | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys | The module key may stay visible in beta code paths, but it is not evidence of true arbitrage platform support. |
| frontend.module.builtin.risk.global | builtin.risk.global | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.execution.paper | builtin.execution.paper | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |
| frontend.module.builtin.runtime.control | builtin.runtime.control | supported | frontend editor owner | sidebar exposure, disabled reasons, UX | frontend:support-matrix.frontend.supportedModuleKeys |  |

## ui_action

| ID | Value | Class | Owner Role | Review Responsibility | Source Of Truth | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ui.action.export_runtime_config | export_runtime_config | supported | frontend editor owner | action gating, reason text, E2E | frontend:CAPABILITY_ACTION_MAP | 只在当前策略图编译通过后导出图生成的 runtime_config。; 当前端正在同步后端能力快照或进入安全回退模式时，该操作会被锁定。 |
| ui.action.export_quantscript | export_quantscript | supported | frontend editor owner | action gating, reason text, E2E | frontend:CAPABILITY_ACTION_MAP | 只导出当前 strategy_graph 草稿，不依赖后端能力门禁，也不会替代 formal QuantScript 编译链路。 |
| ui.action.compile | compile | supported | frontend editor owner | action gating, reason text, E2E | frontend:CAPABILITY_ACTION_MAP | Strategy IR 只承担语义预检。; 运行时编译仍然是可运行输出的最终真源。 |
| ui.action.start_simulation | start_simulation | supported | frontend editor owner | action gating, reason text, E2E | frontend:CAPABILITY_ACTION_MAP | 当前 Beta 边界内仅支持纸面模拟运行时。; 缓存回退模式下仍可见，但依旧受后端校验约束。 |
| ui.action.run_backtest | run_backtest | supported | frontend editor owner | action gating, reason text, E2E | frontend:CAPABILITY_ACTION_MAP | 当前仅提供基础回放/回测支持，不宣称研究级回测能力。; 缓存回退模式下仍可见，但依旧受后端校验约束。 |
| ui.action.run_parameter_sweep | run_parameter_sweep | supported | frontend editor owner | action gating, reason text, E2E | frontend:CAPABILITY_ACTION_MAP | 参数扫掠建立在现有回测能力边界之上，能力未同步或 safe fallback 时不得继续暴露为可执行入口。; 该入口只表示窄执行假设扫描，不表示通用优化器或第二套实验运行时。 |

## workspace_surface

| ID | Value | Class | Owner Role | Review Responsibility | Source Of Truth | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| workspace.surface.template_library | template_library | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | frontend:support-matrix.workspace.surfaces.template_library | 模板库是前端本地 starter graph 入口，不依赖 /api/capabilities 显隐。; 加载模板只替换当前内存工作草稿，不创建第二套后端模板传输。 |
| workspace.surface.version_history | version_history | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | frontend:support-matrix.workspace.surfaces.version_history | 版本历史属于图持久化工作流，不由 /api/capabilities 决定显隐。; 可见不代表扩展了新的 runtime capability，只代表当前图版本工件可管理。 |
| workspace.surface.collaboration_audit | collaboration_audit | supported | frontend editor owner | workspace exposure, backend route honesty, closeout audit | frontend:support-matrix.workspace.surfaces.collaboration_audit | 协作与审计属于当前图元数据和审计记录投影，不由 /api/capabilities 决定显隐。; 当前边界仍是本地 actor 协作切片，不应外推成远程账号系统能力。 |
| workspace.surface.parameter_sweep | parameter_sweep | restricted | frontend editor owner | workspace exposure, backend route honesty, closeout audit | frontend:support-matrix.workspace.surfaces.parameter_sweep | 参数扫掠是现有 backtest surface 上的窄执行假设扫描，不是第二套实验运行时。; 发起扫掠必须遵守与回测相同的 capability 同步和 safe-fallback 锁定规则。 |

## compile_boundary

| ID | Value | Class | Owner Role | Review Responsibility | Source Of Truth | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| compile.strategy_ir_preflight | strategy_ir | restricted | backend compile owner | lowering boundary, diagnostics | frontend:support-matrix.compile.preflightArtifact | Semantic preflight only. It does not decide runnable output. |
| compile.formal_quantscript_lowering | quantscript.formal_source | restricted | backend compile owner | lowering boundary, diagnostics | frontend:support-matrix.compile.boundaryNotes | Owns runtime lowering when present, but runtime compile still decides runnable output. |
| compile.runtime_source_of_truth | /api/runtime/compile | supported | backend compile owner | backend contract, compile/runtime checks | frontend:support-matrix.compile.runtimeSourceOfTruth | When artifacts disagree, runtime behavior follows this source of truth. |

## user_facing_claim

| ID | Value | Class | Owner Role | Review Responsibility | Source Of Truth | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| claim.allowed.纸面运行时_Beta | 纸面运行时 Beta | supported | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.allowedClaims |  |
| claim.allowed.基础回测支持 | 基础回测支持 | supported | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.allowedClaims |  |
| claim.allowed.受限的_Custom_Strategy_IR_表达式路径 | 受限的 Custom Strategy IR 表达式路径 | supported | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.allowedClaims |  |
| claim.disallowed.claiming_research-grade_backtest_support | 宣称具备研究级回测能力 | disallowed_claim | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
| claim.disallowed.claiming_live_trading_support | 宣称支持实盘交易 | disallowed_claim | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
| claim.disallowed.claiming_true_arbitrage_agent_support | 宣称支持真实套利代理 | disallowed_claim | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
| claim.disallowed.claiming_third-party_plugin_marketplace_support | 宣称支持第三方插件市场 | disallowed_claim | docs and QA owner | README, markdown, UI copy, text gates | frontend:support-matrix.userFacingGuardrails.disallowedClaims |  |
