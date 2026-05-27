# QuantPilot 支持矩阵

## 目的

本文档是当前 QuantPilot beta 边界的 P0 支持矩阵参考。用于保持 README、UI 提示、前端能力门禁、测试 fixture 和验收检查的一致性。

矩阵描述：

- 当前已支持的内容
- 仅在受限边界条件下存在的内容
- 哪些内容不得被宣传为已支持平台能力
- 哪些工具栏操作受能力门禁控制以及它们触及哪些后端路由

## 当前已支持边界

### 产品定位与 unsupported 边界

QuantPilot 当前固定为单机交易工具: 面向单人本地桌面使用, 通过 Tauri / localhost 访问本地后端和执行端。以下能力不进入后续里程碑, 不作为缺陷排期:

| 能力 ID | 状态 | 说明 |
|---|---|---|
| `auth.logout` | `unsupported` | 单机本地会话边界, 不做多设备注销语义 |
| `auth.password_reset` | `unsupported` | 不提供远程账户找回或邮件重置流程 |
| `auth.2fa` | `unsupported` | 不做 2FA / TOTP / WebAuthn |
| `account.profile` | `unsupported` | 不做用户资料页或团队账号资料管理 |
| `auth.rbac` | `unsupported` | 不做管理员角色、权限组或多租户 RBAC |
| `hub.search` | `unsupported` | 单机策略数保持在可滚动范围内 |
| `hub.filter` | `unsupported` | 策略中心不做筛选、分页或排序工作台 |

现有 `register` / `login` / `refresh` 端点只保留为本地会话和凭证隔离机制, 不代表完整账户系统支持。

### 运行时

- 主应用 API runtime mode：`paper`
- 用户侧与执行端统一命名：
  - `PaperSimulated`：本地仿真，本地模拟成交，不连接 provider submission。
  - `PaperActual`：OKX demo / testnet 边界，仅代表演示盘提交，不代表真实资金。
- `live_execution_allowed=false` 是当前硬边界。用户文案、README、总览和里程碑不得再用 `live` 描述 OKX demo / testnet。
- v4 策略进入执行端启动前必须消费后端生成的 `strategy_config_preflight`；执行端不得自行推断 capability。缺失、blocked、真实资金字段打开、模式不匹配或 action 未允许时直接拒绝启动。
- 已支持的执行模块：`builtin.execution.paper`、`live.okx`
  - `live.okx` 仅作为当前内部能力键保留；用户侧必须显示为 `PaperActual` / `OKX demo` 边界，不得显示为实盘或 live trading。
- 当前市场边界：
  - 交易所：`binance`、`okx`
  - 交易对：`BTCUSDT`、`ETHUSDT`、`SOLUSDT`

### AI proposal 与策略配置域

- AI proposal 只能停留在 `proposal_only` 治理边界，不能直接写 QS、不能直接启动运行、不能绕过审批。
- `ai.proposal.config_domain_binding` 已进入 v4 策略配置系统主线：proposal 静态检查要求 `target_domain`、`before_digest`、`after_digest` 和 `evidence_anchor_ids`；缺失时保留为不可审批通过的失败候选。
- v4 proposal 必须绑定 backtest evidence；非 v4 proposal 仍绑定 run evidence。
- 审批通过前必须验证配置域绑定存在、静态检查通过、沙箱报告存在且未判定为 underperform；否则返回 `423 Locked`。
- 沙箱自动重试失败必须进入审批生命周期，不能静默变成可审批状态。

### Strategy IR 和 QuantScript 边界

- 已声明的指标类型：
  - `ma_cross`
  - `rsi`
  - `macd`
  - `momentum`
  - `spread`
  - `z_score`
  - `custom`
  - `quote_observe`
  - `atr`
  - `bollinger_bands`
  - `obv`
  - `cmf`
  - `adx`
  - `stochastic`
  - `cci`
  - `parabolic_sar`
  - `keltner_channel`
  - `donchian_channel`
- 当前已支持的指标类型：
  - `ma_cross`
  - `rsi`
  - `macd`
  - `momentum`
  - `spread`
  - `z_score`
  - `custom`
  - `quote_observe`
  - `atr`
  - `bollinger_bands`
  - `obv`
  - `cmf`
  - `adx`
  - `stochastic`
  - `cci`
  - `parabolic_sar`
  - `keltner_channel`
  - `donchian_channel`

边界说明：

- `custom` 仅通过降级为 Core IR 的受限 Strategy IR 表达式路径支持。
- `custom` 不允许任意宿主代码、直接风险变更或绕过执行。
- `strategy_ir` 仅为语义预检。它不是运行时的真实数据源。
- 当存在时，`quantscript.formal_source` 负责运行时降级。
- 当工件不一致时，运行时行为遵循 `/api/runtime/compile`。
- UI 和文档必须将编译解释呈现为三个独立字段：`Strategy IR 角色`、`运行时来源` 和 `可运行真实结果`。
- 当前精确的正式 QuantScript 语法和解析与降级限制在 `markdown/guides/quantscript/guide-formal-quantscript-syntax.md` 中定义。

### 已支持的前端模块键

- `builtin.data.kline`
- `builtin.data.quote`
- `builtin.intent.double_ma`
- `builtin.intent.ma_deviation`
- `builtin.intent.rsi`
- `builtin.intent.macd`
- `builtin.intent.momentum`
- `builtin.intent.zscore`
- `builtin.intent.spread_observer`
- `builtin.agent.weighted`
- `builtin.agent.arbitrage`
- `builtin.risk.global`
- `builtin.execution.paper`
- `builtin.runtime.control`
- `v4.machine.param`
- `v4.transition.guard`

边界说明：

- 价差相关和套利相关的模块键可能出现在 beta 编译路径中。
- 它们不得在外部被描述为真实套利平台支持的证据。
- `v4.machine.param` 和 `v4.transition.guard` 属于 v4 策略配置系统的配置域能力，不代表新增 Machine 模板，也不代表真实资金执行能力。
- 前端模块暴露必须与 `/api/capabilities` 保持一致。
- `builtin.data.kline` 和 `builtin.data.quote` 现在在前端/运行时编译路径中暴露 `ping_enabled` 和 `request_interval_ms`。
- 这些请求控制字段还不是图生成的正式 QuantScript 面的一部分。

## 能力驱动的 UI 操作

所有用户可见 UI 操作必须先出现在 `/api/capabilities.ui_actions.actions`。前端的 `CAPABILITY_ACTION_MAP` 只保留按钮文案、排序、路由备注和测试定位，不再作为操作可用性的真源。

| 操作 | 能力门禁 | 后端路由 | 备注 |
|---|---|---|---|
| `打开教程` | 由 `/api/capabilities.ui_actions.actions` 控制 | 无 | 本地教程入口；可见不等于产品能力扩展 |
| `管理凭证` | 由 `/api/capabilities.ui_actions.actions` 控制 | `/api/credentials` | 凭证面板不代表实盘执行已开放 |
| `新建策略图` | 由 `/api/capabilities.ui_actions.actions` 控制 | 无 | 重置本地草稿 |
| `加载最新` | 由 `/api/capabilities.ui_actions.actions` 控制 | `/api/graphs/latest` | 图持久化读取路径 |
| `保存策略图` | 由 `/api/capabilities.ui_actions.actions` 控制 | `/api/graphs` | 图持久化写入路径，不代表运行时写入 |
| `编译` | 能力同步加载时或安全回退激活时锁定 | `/api/strategy-ir/compile`、`/api/quantscript/formal/compile`、`/api/runtime/compile` | `strategy_ir` 仅为预检；运行时编译保持权威 |
| `导出配置` | 能力同步加载时或安全回退激活时锁定 | `/api/runtime/compile` | 导出依赖于可编译的运行时配置 |
| `启动模拟` | 能力同步加载时或安全回退激活时锁定 | `/api/runtime/test-run`、`/api/runtime/runs/:run_id/events`、`/api/runtime/runs/:run_id/status` | 当前 beta 边界仅为纸面运行时 |
| `启动 v4 模拟运行` | 能力同步加载时或安全回退激活时锁定 | `/api/runtime/v4/run` | 固定使用 `PaperSimulated`；只接收 v4 QS 静态审计通过后的 machine graph handoff |
| `运行回测` | 能力同步加载时或安全回退激活时锁定 | `/api/runtime/backtest`、`/api/runtime/backtests`、`/api/runtime/backtests/:backtest_id` | 当前回测仅为基础回放/回测支持 |
| `运行参数扫掠` | 能力同步加载时或安全回退激活时锁定 | `/api/runtime/experiments/backtest-sweep`、`/api/runtime/experiments`、`/api/runtime/experiments/:experiment_id` | 在现有回测面上进行窄执行假设扫描；不是第二套实验运行时 |
| `停止` | 由 `/api/capabilities.ui_actions.actions` 控制 | `/api/runtime/runs/:run_id/status` | 只对当前运行中会话可用 |
| `重置运行时` | 由 `/api/capabilities.ui_actions.actions` 控制 | 无 | 清理前端运行态投影和连接状态 |
| `打开回测` | 由 `/api/capabilities.ui_actions.actions` 控制 | `/api/runtime/backtests` | 进入回测列表，不直接触发回测写入 |
| `导出 strategy_graph 源码` | 由 `/api/capabilities.ui_actions.actions` 控制 | 无 | 仅前端图源草稿导出；这不是正式 QuantScript 语言 |

## 能力驱动的可见工作区界面

所有工作区入口必须先出现在 `/api/capabilities.workspace.surfaces`。前端的 `WORKSPACE_SURFACE_MAP` 只保留排序、标签、说明和路由备注，不再作为显隐或可点击状态的真源。

| 界面 | 可见性真实数据源 | 后端路由 | 能力驱动？ | 备注 |
|---|---|---|---|---|
| `总览` | `/api/capabilities.workspace.surfaces` | 无 | 是 | 聚合当前图、编译和运行摘要 |
| `构建` | `/api/capabilities.workspace.surfaces` | `/api/runtime/compile` | 是 | 图编辑、诊断和源码审查主工作区 |
| `检查` | `/api/capabilities.workspace.surfaces` | `/api/runtime/compile` | 是 | 可由问题队列程序化进入，不一定作为一级标签展示 |
| `研究回测` | `/api/capabilities.workspace.surfaces` | `/api/runtime/backtest`、`/api/runtime/backtests` | 是 | 仅代表基础回放/回测支持 |
| `运行监控` | `/api/capabilities.workspace.surfaces` | `/api/runtime/runs/:run_id/events`、`/api/runtime/runs/:run_id/status` | 是 | 运行时只读投影和事件流摘要 |
| `源码` | `/api/capabilities.workspace.surfaces` | 无 | 是 | 图谱源码和 Strategy IR 审查材料 |
| `策略模板库` | `/api/capabilities.workspace.surfaces` | 无 | 是 | 本地模板注册表提供内容，后端 capability 决定入口状态 |
| `版本历史` | `/api/capabilities.workspace.surfaces` | `/api/graphs/:graph_id/versions`、`/api/graphs/:graph_id/versions/:version_id`、`/api/graphs/:graph_id/versions/:version_id/restore`、`/api/graphs/:graph_id/versions/compare` | 是 | 可见不代表扩展新的 runtime capability |
| `协作与审计` | `/api/capabilities.workspace.surfaces` | `/api/graphs/:graph_id/audit` | 是 | 当前切片是本地 actor 协作元数据，而非远程账号系统 |
| `参数扫掠` | `/api/capabilities.workspace.surfaces` | `/api/runtime/experiments/backtest-sweep`、`/api/runtime/experiments`、`/api/runtime/experiments/:experiment_id` | 是 | 作为窄工作区卡片可见，但其提交操作必须遵守与回测相同的能力锁定规则 |

## 能力源行为

### `remote`

- 正常运行状态。
- 前端应信任 `/api/capabilities` 作为活跃能力参考。

### `cache`

- 降级但仍可使用。
- 前端可保持操作可用。
- UI 必须说明最终可用性仍取决于实时后端验证。

### `safe_fallback`

- 风险遏制状态。
- 前端必须隐藏未声明入口，将 `declared_only` 工作区或操作保留为禁用态，并锁定风险操作。
- UI 必须解释能力验证失败，前端已收紧行为以避免暴露虚假能力。

## 允许的声明

- `纸面运行时 beta`
- `基础回测支持`
- `受限的 Custom Strategy IR 表达式路径`

## 不得作为正面支持声明出现的表述

- 宣称具备研究级回测能力
- 宣称支持实盘交易
- 宣称支持真实套利代理
- 宣称支持第三方插件市场

## 参考

- [当前状态与发布状态](../10-overview/overview-current-status-and-roadmap.md)
- [编译链合约](implementation-compile-chain-contract.md)
- [首次发布就绪状态](../../09-archive/planning-retired/implementation-first-release-readiness.md)
