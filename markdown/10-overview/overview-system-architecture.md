# QuantPilot v1.0.0 系统架构与使用手册

> 本文是 QuantPilot 的长期技术参考手册。覆盖完整架构、协议栈、编译链、运行时、插件系统、前端设计、存储安全、开发流程和使用指南。

---

## 一、系统总览

### 1.1 定位

QuantPilot 是一个**单机量化策略研发沙盒**。它不是交易执行平台，不是行情终端，不是策略托管服务，不是社交跟单平台。它的唯一职责是：让用户在本机完成策略的设计、编译验证、Paper 运行、历史回测和结果对比——在策略接触真实资金之前，完成全部研发闭环。

系统支持两个交易所 (`binance`, `okx`)、三个交易对 (`BTCUSDT`, `ETHUSDT`, `SOLUSDT`)、十八种技术指标、两种运行模式 (`paper` 纸面交易和 `testnet` 测试网)、以及从数据获取到订单成交的完整运行时链路。

### 1.2 核心设计原则

| 原则 | 说明 | 违反示例 |
|------|------|---------|
| 诚实的能力边界 | `/api/capabilities` 暴露当前真实支持的交易所、交易对、指标、运行模式。不支持的功能在 API 和 UI 中显式标记为不可用，而不是假装支持然后静默失败 | 将 `Custom` indicator 包装成"支持任意策略" |
| 可复现的运行时行为 | 同一策略图 + 同一编译 ID + 同一回放源 (DeterministicMock)，每次运行产生相同的事件序列、相同的成交、相同的权益曲线 | 在回测中引入真实网络延迟的随机抖动 |
| QS 唯一编译路径 | 所有策略必须经过 `graph JSON → QS 源码 → parse → HIR → lower → Core IR` 管道。禁止任何绕过此管道的编译入口 | 前端直接将 `runtime_config` 发给后端编译 |
| 本地沙盒优先 | 没有服务端依赖。所有数据、凭证、运行记录、回测结果存储在本地文件系统。桌面应用不需要网络连接即可完成策略编辑、编译和确定性回测 | 要求用户登录云端账户才能使用策略编辑器 |
| 插件可插拔 | 核心只保留编译链、沙盒调度器和插件协议。策略逻辑 (指标计算、代理决策、风控规则、执行算法) 全部通过插件注册表加载 | 在核心代码中硬编码一个新的 indicator |

### 1.3 技术栈全貌

```
┌─ 桌面壳 ───────────────────────────────────────────────────────┐
│  Tauri v2 (Rust)                                               │
│  ├── 自绘标题栏 (decorations: false, 32px, z-index: 200)       │
│  ├── WebView2 嵌入式浏览器 (Windows 11 内置)                     │
│  ├── 原生窗口管理 (最小化/最大化/关闭)                            │
│  └── beforeDevCommand: 自动启动 Vite + Cargo                    │
├─ 前端 (React SPA) ─────────────────────────────────────────────┤
│  React 18 + Vite 6 + Zustand 4 + React Flow 12                 │
│  ├── Adobe 暗色面板设计系统 (--ad-* CSS 令牌, 约 50 个变量)       │
│  ├── SVG 图标组件库 (禁止 Unicode Emoji)                        │
│  ├── Testing Library + Vitest (92 文件, 269 测试)               │
│  └── Playwright (E2E, API-mock 隔离合约)                       │
├─ 后端 (Rust Axum) ─────────────────────────────────────────────┤
│  Axum 0.7 (HTTP) + Tokio (async runtime, multi-thread)          │
│  ├── QRPC 运行时协议栈 (20 RFC, 19 已落地)                       │
│  ├── QuantScript 编译器 (parse → HIR → lower → Core IR)          │
│  ├── 5 个子 crate: qrpc_core / qrpc_core_ir / qrpc_compiler     │
│  │                 qrpc_runtime / quantscript                   │
│  ├── CredentialVault (AES-256-GCM + ring + Zeroizing)          │
│  └── Tauri src-tauri (桌面壳入口)                                │
├─ 存储 (文件系统) ────────────────────────────────────────────────┤
│  storage/                                                      │
│  ├── graphs/      (Permanent)  策略图和 QS 源码                  │
│  ├── runs/        (Temporary)  Paper 运行记录                   │
│  ├── backtests/   (Temporary)  回测工件                         │
│  ├── experiments/ (Temporary)  实验记录                         │
│  ├── snapshots/   (Transient)  快照                            │
│  ├── alerts/      (Transient)  告警                            │
│  ├── chaos/       (Transient)  混沌实验报告                      │
│  ├── audit/       (Permanent)  审计日志                         │
│  └── .credentials (Permanent)  AES-256-GCM 加密凭证             │
└────────────────────────────────────────────────────────────────┘
```

### 1.4 部署拓扑

QuantPilot 是单进程桌面应用。后端 (Axum HTTP Server) 和前端 (Vite Dev Server / 生产构建静态文件) 在 Tauri 壳内共享同一个操作系统进程。前端通过 `http://127.0.0.1:3000` 与后端通信，不暴露任何外部网络端口。

在浏览器模式下运行时，前端可通过环境变量 `VITE_API_BASE_URL` 指向任意后端地址，但生产构建始终内嵌在 Tauri 壳中。

### 1.5 数据流生命周期

一次完整的策略运行经过以下数据流阶段：

```
用户操作 (前端)
  │  拖拽节点、连线、配置参数
  ▼
策略图 (graph JSON)
  │  nodes, edges, metadata, validation_state
  ▼
QS 编译 (compile_api.rs)
  │  generate_quantscript_from_graph_value → parse → lower → compile
  ▼
Core IR (CoreStrategyIr)
  │  data_bindings, indicators, signal_rules, agent_policies, risk_policies, execution
  ▼
运行时调度 (RuntimeCoordinator)
  │  DataCollection → IntentComputation → AgentDecision → RiskCheck → ExecutionPlan
  ▼
事件流 (Vec<RuntimeEvent>)
  │  event_id, event_type, payload, ts_ms, trace_id
  ▼
前端渲染 (EventStreamPanel)
  │  实时 SSE 推送或历史事件回放
  ▼
持久化 (storage/runs/*.json)
  │  RunRecord / BacktestRecord
  ▼
回测详情 / 对比分析
```

---

## 二、架构分层详解

### 2.1 前端层

前端是一个 React 单页应用，通过 Zustand 管理全局状态。应用启动时：

1. 从 `localStorage` 加载上次保存的 `capabilities` 缓存
2. 调用 `GET /api/capabilities` 获取后端最新能力快照
3. 比较 `schema_hash`，如不一致则更新模块注册表
4. 从 `localStorage` 加载上次编辑的策略图草稿
5. 渲染当前路由对应的页面

**路由表**:

| 路由 | 页面 | 职责 |
|------|------|------|
| `/strategies` | StrategyHubPage | 策略目录，全局管理入口 |
| `/strategies/:id` | StrategyWorkspacePage | 单策略工作台 |
| `/strategies/:id/backtests` | StrategyBacktestsPage | 单策略回测历史 |
| `/backtests/:id` | BacktestDetailPage | 单次回测详情 |
| `/backtests/compare?ids=` | BacktestComparePage | 多回测对比分析 |
| `/approvals` | ApprovalPage | 审批面板 |
| `/alerts` | AlertsPage | 告警面板 |
| `/snapshots` | SnapshotsPage | 快照管理 |
| `/runbook` | RunbookPage | 故障手册 |
| `/chaos` | ChaosPage | 混沌实验 |
| `/quantscript` | QuantScriptEditor | QS 源码编辑器 |

### 2.2 API 网关层

Axum HTTP Server 在 `127.0.0.1:3000` 监听。请求经过以下中间件栈：

```
Request
  → tower-http CORS (permissive, localhost only)
  → auth_middleware (Bearer token, DEV 模式跳过)
  → rate_limiter (令牌桶, 默认 60 req/min)
  → json_rejection_middleware (JSON 解析错误→中文消息)
  → Router
      ├── /api/runtime/*   → runtime_api.rs
      ├── /api/graphs/*    → graph_api.rs
      ├── /api/credentials → credential_api.rs
      ├── /api/capabilities → capability_api.rs
      ├── /api/compile     → compile_api.rs
      └── /api/health      → 健康检查
```

### 2.3 编译链层

详细说明见第四章。

### 2.4 运行时层

详细说明见第五章。

### 2.5 插件注册表层

详细说明见第六章。

### 2.6 存储层

详细说明见第八章。

---

## 三、协议栈详解 (QRPC RFC 001-020)

QuantPilot 的运行时协议命名为 QRPC (Quant Runtime Protocol Core)，共有 20 个 RFC。每个 RFC 定义一个或多个数据结构，所有结构体在 Rust 中实现为 `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]` 的结构体和枚举，位于 `qrpc_core/src/lib.rs`。

协议的序列化格式统一为 JSON (serde_json)。前端的 capability fixture (`backend-capabilities-v1.json`) 是后端 `build_capability_response()` 的精确快照。

### 3.1 RFC-001: 数据请求协议 (DataRequest)

`DataRequest` 是 QRPC 主链的起点。它表达"系统需要什么数据语义"，而不是"去哪个交易所调哪个接口"。

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| `request_id` | String | 请求唯一标识 |
| `instrument` | Symbol | 标的，如 `BTCUSDT` |
| `market_scope` | MarketScope | 市场域: Spot/Margin/Perpetual/Futures/Options/Index/Composite |
| `primary_data_type` | PrimaryDataType |一级数据类型: `FactPrice`(事实价格快照) 或 `KlineRange`(K 线序列) |
| `source_type` | SourceType | 语义子维度: SpotTrade/SpotTicker/PerpetualTrade/PerpetualMark/PerpetualIndex/FuturesTrade/FuturesMark/FuturesIndex/IndexPrice/Aggregated |
| `timeframe` | Option\<Timeframe\> | K 线周期: Tick/Ms100/Sec1/Sec5/Min1/Min3/Min5/Min15/Min30/Hour1/Hour4/Day1/Week1 |
| `lookback_count` | Option\<u32\> | 回看窗口大小 (历史数据条数) |
| `time_range` | Option\<TimeRange\> | 显式时间范围: `{ start_ms, end_ms }` |
| `precision_policy` | PrecisionPolicy | 精度策略: `{ price_scale, quantity_scale, rounding_mode: Floor/Ceil/Round/Truncate }` |
| `usage_tag` | UsageTag | 用途标签: LiveExecution/IntentComputation/FactSimulation/HistoricalBacktest/Diagnostics |
| `priority` | u8 | 调度优先级 (单机资源受限时使用) |
| `is_realtime` | bool | 是否需要实时推送 |
| `requested_at_ms` | u64 | 请求发起时间戳 (毫秒) |

**与适配层的关系**: `DataRequest` 是语义层协议，`Source Adapter` 是实现层协议。适配层负责把 `DataRequest` 翻译成具体交易所的 HTTP 调用，但不能修改 `DataRequest` 的核心语义。

### 3.2 RFC-002: 规范化市场数据 (NormalizedMarketData)

将不同交易所的原始响应统一为两种标准格式：

**KlineSeriesSnapshot** — K 线序列快照:

| 字段 | 类型 | 说明 |
|------|------|------|
| `data_id` | String | 数据源标识 |
| `exchange` | Exchange | 交易所 |
| `symbol` | Symbol | 标的 |
| `market_type` | MarketType | 市场类型 |
| `interval` | String | K 线周期 (如 "1h") |
| `bars` | Vec\<NormalizedKline\> | K 线列表 (按 open_time 升序) |
| `window_len` | usize | 窗口大小 |
| `ts_ms` | u64 | 快照时间戳 |
| `source_latency_ms` | u64 | 数据源延迟 |
| `source_status` | SourceStatus | 数据源状态 |
| `data_quality` | DataQualitySnapshot | 数据质量快照 |

每条 `NormalizedKline` 包含: `open_time_ms`, `close_time_ms`, `open`, `high`, `low`, `close`, `volume`。

**QuoteSnapshot** — 报价快照:

| 字段 | 类型 | 说明 |
|------|------|------|
| `data_id` | String | 数据源标识 |
| `exchange` | Exchange | 交易所 |
| `symbol` | Symbol | 标的 |
| `best_bid` | f64 | 最优买价 |
| `best_ask` | f64 | 最优卖价 |
| `bid_size` | f64 | 买一量 |
| `ask_size` | f64 | 卖一量 |
| `mid_price` | f64 | 中间价 |
| `ts_ms` | u64 | 时间戳 |
| `source_latency_ms` | u64 | 数据源延迟 |

### 3.3 RFC-005/006: 意图协议

**IntentSignal** — 指标计算的输出信号:

| 字段 | 类型 | 说明 |
|------|------|------|
| `signal_id` | String | 信号唯一标识 |
| `intent_id` | String | 关联的 intent 标识 |
| `kind` | IntentKind | 信号类型 (MA_CROSS, RSI, MACD, MOMENTUM, ZSCORE, SPREAD, CUSTOM) |
| `exchange_scope` | Vec\<Exchange\> | 适用交易所范围 |
| `symbol_scope` | Vec\<Symbol\> | 适用标的范围 |
| `side` | SignalSide | 信号方向: Buy/Sell/Neutral |
| `strength` | f64 | 信号强度 (用于排序和加权) |
| `confidence` | f64 | 信号置信度 [0, 1] |
| `reference_price` | Option\<f64\> | 参考价格 |
| `derived_metrics` | BTreeMap\<String, f64\> | 衍生指标 (如 RSI 值、MACD 柱值、布林带宽度) |
| `reason` | String | 信号生成原因 (中文描述) |
| `triggered_at_ms` | u64 | 触发时间 |
| `ttl_ms` | u64 | 信号有效期 (超时后失效) |
| `trace_id` | String | 追踪 ID (串联同一次运行的所有事件) |

### 3.4 RFC-004/007/008/009: 决策协议

**AgentDecision** — 代理决策输出:

| 字段 | 类型 | 说明 |
|------|------|------|
| `decision_id` | String | 决策唯一标识 |
| `agent_id` | String | 代理标识 |
| `symbol` | Symbol | 标的 |
| `exchange_targets` | Vec\<Exchange\> | 目标交易所 |
| `net_side` | SignalSide | 净方向 (Long/Short/Neutral) |
| `net_strength` | f64 | 净强度 |
| `portfolio_target_decision` | Option\<PortfolioTargetDecision\> | 组合目标决策 (可选) |
| `proposed_actions` | Vec\<ProposedAction\> | 提议的操作 |
| `reason` | String | 决策原因 |
| `produced_at_ms` | u64 | 产出时间 |
| `trace_id` | String | 追踪 ID |

**PortfolioTarget** — 目标持仓:

| 字段 | 类型 | 说明 |
|------|------|------|
| `allocation_kind` | String | 分配类型 |
| `target_weights` | Vec\<TargetWeight\> | 目标权重列表 |

**TargetWeight**:

| 字段 | 类型 | 说明 |
|------|------|------|
| `exchange` | Exchange | 交易所 |
| `symbol` | Symbol | 标的 |
| `target_weight` | f64 | 目标权重 (0-1) |
| `current_weight` | f64 | 当前权重 |
| `reference_price` | f64 | 参考价格 |
| `signal_score` | Option\<f64\> | 信号评分 |

**PortfolioState** — 当前持仓快照:

| 字段 | 类型 | 说明 |
|------|------|------|
| `cash_balance` | f64 | 现金余额 |
| `available_cash_balance` | f64 | 可用现金 |
| `frozen_cash_balance` | f64 | 冻结现金 |
| `open_orders` | Vec\<OpenOrder\> | 未结订单 |
| `positions` | Vec\<Position\> | 持仓列表 |
| `exchange_exposures` | Vec\<ExchangeExposure\> | 交易所敞口 |
| `total_gross_notional` | f64 | 总名义价值 |
| `total_net_notional` | f64 | 净名义价值 |
| `total_leverage` | f64 | 总杠杆 |
| `updated_at_ms` | u64 | 更新时间 |

**SignalSide** — 信号方向: `Long` / `Short` / `Neutral`。

**RiskDecision** — 风控决策:

| 字段 | 类型 | 说明 |
|------|------|------|
| `risk_decision_id` | String | 决策标识 |
| `decision_id` | String | 关联的代理决策 ID |
| `symbol` | Symbol | 标的 |
| `adjusted_actions` | Vec\<ProposedAction\> | 调整后的操作 |
| `adjusted_portfolio_target_decision` | Option\<PortfolioTargetDecision\> | 调整后的组合目标 |
| `status` | DecisionStatus | Accept/Reduce/Reject |
| `risk_explanations` | Vec\<String\> | 风控解释 (中文) |

### 3.5 RFC-011/012/013: 执行协议

**ExecutionPlan** — 执行计划:

| 字段 | 类型 | 说明 |
|------|------|------|
| `plan_id` | String | 计划标识 |
| `source_risk_decision_id` | String | 关联的风控决策 ID |
| `orders` | Vec\<SimOrder\> | 订单列表 |
| `created_at_ms` | u64 | 创建时间 |
| `trace_id` | String | 追踪 ID |

**Order** — 订单 (完整生命周期):

| 字段 | 类型 | 说明 |
|------|------|------|
| `order_id` | String | 系统内部订单 ID |
| `client_order_id` | Option\<String\> | 客户端订单 ID |
| `exchange` | Exchange | 交易所 |
| `instrument` | Symbol | 标的 |
| `side` | OrderSide | 方向: Buy/Sell |
| `order_type` | OrderType | 类型: Market/Limit/StopLoss/StopLossLimit/TakeProfit/TakeProfitLimit |
| `price` | Option\<f64\> | 限价 (市价单为空) |
| `quantity` | f64 | 委托数量 |
| `executed_qty` | f64 | 已成交数量 |
| `time_in_force` | TimeInForce | 时效: Gtc(有效至取消)/Ioc(立即或取消)/Fok(全部或取消) |
| `status` | OrderStatus | 状态机: Created→Submitted→Accepted→PartiallyFilled→Filled/Cancelled/Rejected/Expired |
| `source_intent_id` | Option\<String\> | 来源意图 ID |
| `source_agent_id` | Option\<String\> | 来源代理 ID |
| `venue_order_id` | Option\<String\> | 交易所返回的订单 ID |
| `created_at_ms` | u64 | 创建时间 |
| `updated_at_ms` | u64 | 最后更新时间 |

**订单状态流转**:
```
Created → Submitted → Accepted → PartiallyFilled → Filled
                       ↘ Rejected
                       ↘ Cancelled
                       ↘ Expired
```

**ExecutionFeedback** — 执行反馈:

| 字段 | 类型 | 说明 |
|------|------|------|
| `feedback_id` | String | 反馈标识 |
| `order_id` | String | 关联的订单 ID |
| `kind` | FeedbackKind | 反馈类型: OrderSubmitted/OrderRejected/OrderPartiallyFilled/OrderFilled/OrderCancelled/OrderExpired/VenueError |
| `fill_qty` | Option\<f64\> | 本次成交数量 |
| `fill_price` | Option\<f64\> | 本次成交价格 |
| `remaining_qty` | Option\<f64\> | 剩余未成交数量 |
| `reject_reason` | Option\<String\> | 拒绝原因 |
| `venue_message` | Option\<String\> | 交易所原始消息 |
| `occurred_at_ms` | u64 | 发生时间 |
| `ingested_at_ms` | u64 | 系统接收时间 |

### 3.6 RFC-014/015: 运行时协议

**RuntimeEvent** — 运行时事件信封。每次数据更新、意图计算、代理决策、风控检查、订单成交、订单状态变化都产生一个 RuntimeEvent。事件 ID 格式为 `evt-{stage}-{source_id}-{ts_ms}`。

**事件类型枚举**: DataUpdated, IntentComputed, AgentDecided, RiskChecked, ExecutionPlanned, OrderSubmitted, OrderCancelled, OrderFilled, PortfolioUpdated, CapabilitySnapshotTaken, RuntimeError, RuntimeWarning。

每个事件携带 `governance` 快照 (capability_hash, deployment_revision, strategy_version, parameter_version) 以支持回放验证。

### 3.7 RFC-016: 能力发现协议

`GET /api/capabilities` 返回当前系统的完整能力声明:

```json
{
  "api_version": "quantpilot-capabilities/v1",
  "schema_version": "quantpilot/capabilities-schema/v1",
  "schema_hash": "sha256:6c294c89...",
  "chain_stages": ["data", "intent", "agent", "risk", "execution", "fill"],
  "strategy_ir": {
    "declared_indicator_kinds": ["ma_cross", "rsi", ...],
    "supported_indicator_kinds": ["ma_cross", "rsi", ...],
    "indicator_support": [{"kind": "ma_cross", "status": "supported", "reason": null}]
  },
  "runtime": {
    "supported_modes": ["paper"],
    "supported_execution_modules": ["builtin.execution.paper"],
    "mode_support": [{"key": "paper", "status": "supported"}]
  },
  "market_data": {
    "supported_exchanges": ["binance", "okx"],
    "supported_symbols": ["BTCUSDT", "ETHUSDT", "SOLUSDT"]
  },
  "frontend": {
    "declared_module_keys": [...],
    "supported_module_keys": [...],
    "unsupported_module_reasons": {}
  },
  "versioning": {
    "model_version": "quantpilot/versioning-model/v1",
    "strategy_version_source": "frontend_runtime_config.metadata.version",
    "parameter_version_policy": "immutable_generation_pointer",
    "deployment_revision_policy": "strategy_version_plus_compile_id_plus_capability_hash"
  },
  "permission_boundary": {
    "model_version": "quantpilot/permission-boundary/v1",
    "execution_owner_module": "builtin.execution.paper",
    "live_execution_allowed": false,
    "ai_write_policy": "proposal_only",
    "plugin_network_default": "deny",
    "non_execution_order_access": "deny"
  }
}
```

### 3.8 RFC-017/018/019: 回测协议

**RunSpec** — 运行规格: 声明 `run_mode`(LiveReplay/DeterministicMock)、`datasets`、`execution_assumptions`(slippage_bps, fee_bps, latency, time_in_force)。

**BacktestSpec** — 回测规格: 包含 `RunSpec` + `market_data_snapshot`(历史 K 线快照或 Mock 快照)。

**BacktestOutput** — 回测输出: `equity_curve`(每个 cycle 的权益值)、`BacktestSummary`(total_return_ratio, max_drawdown_ratio, sharpe_ratio, win_rate, trade_count)、`sessions`(每段回放会话的 SessionOutput)。

### 3.9 RFC-020: 插件清单协议

**PluginManifest** 完整字段:

| 字段 | 类型 | 必填 | 说明 |
|------|------|:--:|------|
| `api_version` | String | ✅ | 固定为 `"quantpilot/plugin-manifest/v1"` |
| `id` | String | ✅ | 全局唯一插件标识 |
| `version` | String | ✅ | 语义化版本 |
| `plugin_type` | Option\<PluginType\> | | Atom(原子) 或 Suite(套件) |
| `kind` | PluginKind | ✅ | Data/Intent/Agent/Risk/Execution |
| `display` | PluginDisplay | ✅ | `{name, summary}` |
| `capability_declarations` | Vec\<CapabilityDeclaration\> | ✅ | 声明的能力合约列表 |
| `extension_points` | Vec\<ExtensionPoint\> | ✅ | 挂载的扩展点 |
| `execution` | PluginExecution | ✅ | 执行引擎 (Native/Builtin) + 入口点 |
| `compatibility` | PluginCompatibility | ✅ | Core IR 版本 + Capability API 版本 |
| `security` | PluginSecurity | ✅ | `{max_compute_ms, max_memory_mb, allow_network}` |
| `dependencies` | Vec\<PluginDependency\> | | 依赖的其他插件 |
| `params_schema` | Option\<Value\> | | JSON Schema 格式的参数定义 |
| `atoms` | Vec\<AtomRef\> | | 套件引用的原子列表 |
| `hot_handoff` | bool | | 是否支持热接管 |
| `asset_management` | bool | | 是否有完整资产管理能力 |

---

## 四、编译链详解

### 4.1 管道全貌

```
POST /api/runtime/compile
  Body: { graph_json: Value, runtime_config: FrontendRuntimeConfig }
    │
    ├── 1. 空 intent 保护
    │      if runtime_config.intent_generators.is_empty() → 400 "策略必须包含至少一个意图"
    │
    ├── 2. 能力校验
    │      validate_runtime_config_capabilities()
    │      检查每个 intent/agent/risk/execution 模块 key 是否在 capability 白名单中
    │
    ├── 3. 合约诊断
    │      collect_runtime_compile_contract_diagnostics()
    │      检查 graph_id 有效性、节点完整性、连线一致性
    │
    ├── 4. QS 管道编译 (§1.1, §1.3 唯一路径)
    │      compile_runtime_protocol_via_qs(&request.graph_json)
    │        │
    │        ├── generate_quantscript_from_graph_value()
    │        │     图结构 (nodes/edges/config) → QuantScript 源码字符串
    │        │     每个节点类型生成对应的 QS 语句:
    │        │       data 节点 → fetch("binance", "BTCUSDT", "1h")
    │        │       intent 节点 → indicator("ma_cross", fast=7, slow=25)
    │        │       agent 节点 → agent("weighted")
    │        │       risk 节点 → risk.profile("global", max_position=0.5)
    │        │       execution 节点 → execution.profile("paper", fee_bps=10)
    │        │
    │        ├── parse_graph_quantscript_source()
    │        │     QS 源码字符串 → Tokenize → Parse → AST (ScriptModule)
    │        │     语法错误在此阶段抛出，带 QS 诊断码 (如 QS0001, QS0403, QS0504)
    │        │
    │        ├── convert_graph_json_to_script_module()
    │        │     AST → 标准化的 ScriptModule (补全 imports, 规范化参数)
    │        │
    │        └── quantscript::lower_script_to_runtime_config()
    │              ScriptModule → HIR → Core IR → RuntimeProtocolCoreConfig
    │              HIR 阶段: 语义分析 (函数存在性、参数类型、数据源引用)
    │              lower 阶段: HIR → Core IR (移除语法糖, 展开宏, 解析引用)
    │
    ├── 5. 协议编译
    │      compile_runtime_protocol_config(&qs_protocol)
    │      RuntimeProtocolCoreConfig → CompiledRuntimeProtocol
    │      { protocol_name, config_hash, core_ir }
    │
    ├── 6. 工件构建
    │      build_compile_artifact_bundle()
    │      生成 strategy/compile/core_ir 三个工件 + 摘要
    │
    └── 7. 响应组装
           CompileRuntimeResponse {
             compilable: true,
             protocol_name, config_hash, core_ir,
             artifacts, counts, diagnostics,
             runtime_config, runtime_targets
           }
```

### 4.2 编译产物中的工件

每个编译请求生成三个工件 (Artifact):

| 工件 | 内容 | 用途 |
|------|------|------|
| Strategy Artifact | 策略图 JSON + QS 源码 | 策略版本管理、图恢复 |
| Compile Artifact | RuntimeProtocolCoreConfig | 编译输入快照、回放验证 |
| Core IR Artifact | CoreStrategyIr | 执行引擎输入、跨版本兼容性 |

### 4.3 诊断系统

编译过程中的全部问题通过 `CompileDiagnostic` 结构返回:

| 字段 | 说明 |
|------|------|
| `code` | 诊断码: QSPIPELINE(管道通过), QS0001(语法错误), QS0403(除零), QS0504(周期不足), QPCONVOK(转换成功), QPQSLOW001~004(lowering 错误) |
| `severity` | Error/Warning/Info |
| `message` | 中文诊断消息 |
| `target` | 可选: 定位到具体的 node/edge/field |
| `span_label` | 可选: 源码位置标注 |
| `hint` | 可选: 修复建议 |

### 4.4 架构铁律 (§1)

三条不可违背的规则：

1. **§1.1 QS 唯一路径**: 禁止 graph 编辑器直接产出 `RuntimeProtocolCoreConfig` 用于编译。`compile_runtime_protocol_via_qs()` 是唯一编译入口函数。

2. **§1.3 编译不可绕过**: 禁止在任何 API 路由中直接调用 `map_frontend_runtime_config` 进行编译。该函数在 v0.5.2 已被标记 `#[deprecated]` 且仅保留用于测试。

3. **§5.4 前端必须携带 graph_json**: 编译请求必须同时包含 `runtime_config` 和 `graph_json`。只发 `runtime_config` 的请求返回 400 错误。

### 4.5 QuantScript 语法 (主干)

QuantScript 是策略的形式化定义语言。语法参考: `markdown/04-guides/guide-formal-quantscript-syntax.md`。

核心关键词:

```
# 模块导入
import fetch, indicator, agent, risk, execution from "quantpilot"

# 数据获取
data = fetch("binance", "BTCUSDT", "1h")

# 指标计算
ma_fast = indicator("ma", source=data.close, period=7, method="ema")
ma_slow = indicator("ma", source=data.close, period=25, method="ema")
rsi = indicator("rsi", source=data.close, period=14)
macd = indicator("macd", source=data.close, fast=12, slow=26, signal=9)

# 辅助函数
def crossover(a, b, index):
    return a[index] > b[index] and a[index-1] <= b[index-1]

# 策略函数
def strategy():
    risk.profile("global", max_position=0.5, max_total_leverage=2.0)
    execution.profile("paper", fee_bps=10, slippage_bps=5)
    emit Intent("buy") when crossover(ma_fast, ma_slow)
```

---

## 五、运行时沙盒详解

### 5.1 运行时主链 (逐阶段)

**阶段 1: DataCollection**

Sandbox 调用 `DataModuleProvider::collect()`。数据提供者根据 `CoreStrategyIr.data_bindings` 中声明的数据源配置, 从 OKX/Binance API 获取实时 K 线/报价, 或从本地缓存加载历史数据。输出: `Vec<NormalizedMarketData>` + 数据质量事件。

**阶段 2: IntentComputation**

Sandbox 遍历 `CoreStrategyIr.indicators`, 对每个指标调用 `evaluate_indicator_signal()`。evaluator 根据 `CoreIndicatorKind` 分派到具体计算函数 (evaluate_ma_family, evaluate_rsi, evaluate_macd 等)。输出: `Vec<IntentSignal>`。每个信号携带 strength (强度值)、confidence (置信度)、derived_metrics (RSI 值、MACD 柱等衍生指标)。

**阶段 3: AgentDecision**

Sandbox 调用 `AgentModuleProvider::evaluate()`。代理模块读取所有活跃的 IntentSignal, 根据代理策略 (weighted 加权 / arbitrage 套利) 生成 PortfolioTarget。输出: `Vec<AgentDecision>`, 每个决策包含 `portfolio_targets`。

**阶段 4: RiskCheck**

Sandbox 调用 `RiskCheckerProvider::check()`。风控检查器对每个 AgentDecision 施加约束: max_position_ratio (最大持仓比例), max_single_weight (单一标的最大权重), max_total_leverage (最大总杠杆), max_exchange_leverage (单交易所最大杠杆), min_action_interval_ms (最小操作间隔)。输出: `Vec<RiskDecision>`, 每个决策标记为 Accept (通过)、Reduce (减仓后通过) 或 Reject (拒绝)。

**阶段 5: ExecutionPlan**

Sandbox 调用 `ExecutionModuleProvider::plan_execution()`。执行模块将 RiskDecision 转化为具体的订单列表。对于 paper 执行: 根据 quantity_ratio 和 reference_price 计算订单数量和限价, 生成 SimOrder。输出: `Vec<ExecutionPlan>`, 每个计划包含 `orders: Vec<SimOrder>`。

**阶段 6: FillEngine**

Sandbox 调用 `FillEngine::submit_plan()`。成交引擎模拟订单撮合: Market 订单按当前市价立即成交, Limit 订单在价格满足条件时成交。StopLoss/StopLossLimit 订单在触发价格被穿透时激活。输出: `FillResult`, 包含成交报告和更新后的 PortfolioState。

### 5.2 RealTimeSandbox (实时沙盒)

```
RealTimeSandbox::new(RuntimeCoordinator::new(compiled))
    │
    ├── start()  → running = true
    │
    ├── run_session(slow_now_ms, fast_now_ms)
    │     │
    │     ├── slow_cycle: 数据获取 + 意图计算 + 代理决策
    │     │     (低频: 与 K 线周期对齐, 如 1h)
    │     │
    │     └── fast_cycle: 风控检查 + 订单监控 + 状态更新
    │           (高频: 如每分钟)
    │
    ├── on_market_data(data, now_ms) → Vec<RuntimeEvent>
    │     外部推送市场数据时调用
    │
    ├── submit_execution_plan(plan, data, now_ms) → FillResult
    │     提交新的执行计划
    │
    ├── handoff(snapshot) → Result<()>
    │     v1.0.0 热接管: 验证快照完整性
    │
    └── snapshot(now_ms) → SandboxSnapshot
          当前沙盒状态快照
```

### 5.3 FastBacktestSandbox (快速回测沙盒)

```
FastBacktestSandbox::with_replay_from_core_ir(core_ir, end_ms)
    │
    ├── HistoricalReplay (历史回放)
    │     └── 从 storage/data/ 加载历史 K 线缓存
    │         如果缓存缺失 → 从 OKX/Binance API 拉取 → 缓存到本地
    │         按时间戳顺序回放每条 K 线
    │
    └── DeterministicMock (确定性 Mock)
          └── 使用固定种子 (1710000000000) 生成伪随机 K 线序列
              确保同一策略每次回测结果完全一致
              Mock 数据包含: open/high/low/close/volume, 带趋势和波动性模拟
```

**回测输出**:

```
BacktestOutput
├── backtest_id: String
├── graph_id: String
├── compile_id: String
├── equity_curve: Vec<{ cycle: u64, equity: f64, cash: f64, positions_value: f64 }>
├── summary:
│   ├── total_return_ratio: f64    // 总收益率 (+12.50% 表示 0.125)
│   ├── max_drawdown_ratio: f64   // 最大回撤
│   ├── sharpe_ratio: f64         // 夏普比率
│   ├── win_rate: f64             // 胜率
│   ├── trade_count: u64          // 成交次数
│   └── final_equity: f64         // 最终权益
├── sessions: Vec<SessionOutput>  // 每段回放会话
└── final_portfolio: PortfolioState
```

### 5.4 事件信封 (RuntimeEvent Envelope)

每个运行时事件被包装为标准信封后再持久化和传输:

```json
{
  "envelope": {
    "event_id": "evt-intent-BTCUSDT-1710000000000",
    "event_type": "IntentComputed",
    "run_id": "run_1710000000000",
    "sequence_no": 42,
    "occurred_at_ms": 1710000000000,
    "capability_hash": "sha256:...",
    "deployment_revision": "strategy_v3_compile_abc123_cap_sha256:..."
  },
  "payload": {
    "signal_id": "...",
    "kind": "ma_cross",
    "side": "Buy",
    "strength": 0.85,
    ...
  }
}
```

信封保证:
- `run_id` 串联同一次运行的全部事件
- `sequence_no` 严格递增, 支持断点续传和去重
- `capability_hash` + `deployment_revision` 绑定事件到特定的能力和部署版本, 回放时可校验一致性

---

## 六、插件系统详解 (v1.0.0)

### 6.1 双层模型

**原子 (Atom)**: 最小可组合单元。一个 `PluginType::Atom` 插件实现一个 `PluginKind` 对应的 trait:

| PluginKind | 实现 Trait | 内置实现 |
|------------|-----------|---------|
| Data | `DataModuleProvider` | BuiltinDataModule (OKX + Binance HTTP) |
| Intent | `IntentModuleProvider` | 18 种 indicator evaluator |
| Agent | `AgentModuleProvider` | BuiltinAgentModule (加权 + 套利) |
| Risk | `RiskCheckerProvider` | BuiltinRiskChecker (全局风控) |
| Execution | `ExecutionModuleProvider` | BuiltinExecutionModule (Paper) |

**套件 (Suite)**: 纯打包层。`PluginType::Suite` 不实现任何逻辑 trait, 只声明 `atoms: Vec<AtomRef>` 列表。套件在运行时被展开为其引用的所有原子。

### 6.2 插件生命周期

```
Registered → Active → Stopped / Faulted
     │           │
     └── 已注册但未激活, 不可被策略引用
                 │
                 └── 激活后 Sandbox 可调用其 trait 方法
                           │
                           ├── 正常停用: Stopped
                           └── 异常: Faulted (panic/超时/内存溢出)
```

### 6.3 安全边界

`PluginSecurityAction` 定义了插件可执行的三类操作。Sandbox 在每次插件调用前执行安全检查:

```rust
impl RuntimePluginRegistry {
    pub fn check_security(&self, plugin_id: &str, action: PluginSecurityAction) -> Result<(), String> {
        let manifest = /* 查找 manifest */;
        match action {
            PluginSecurityAction::AccessCredentials => {
                // 任何插件都禁止访问凭证
                return Err("插件不允许访问凭证管理".to_string());
            }
            PluginSecurityAction::NetworkCall => {
                // 只有 manifest.security.allow_network == true 的插件才能联网
                if !manifest.security.allow_network {
                    return Err("插件未声明 allow_network".to_string());
                }
            }
            PluginSecurityAction::WriteState => {
                // 默认允许写入沙盒状态
            }
        }
        Ok(())
    }
}
```

### 6.4 原子注册流程

```
启动时
  │
  ├── 扫描 plugins/atoms/*.json
  │     对每个 .json 文件:
  │       1. 读取文件内容
  │       2. serde_json::from_str → PluginManifest
  │       3. manifest.validate() → 字段校验
  │       4. RuntimePluginRegistry::scan_atoms() → 注册
  │
  └── 内置模块自动注册
        BuiltinDataModule / 18 个 indicator evaluator /
        BuiltinAgentModule / BuiltinRiskChecker / BuiltinExecutionModule
        全部注册为 plugin_type=None (内置, 不参与套件)
```

### 6.5 套件校验规则

```rust
pub fn validate_suite(&self, suite: &PluginManifest) -> Result<(), Vec<String>> {
    // 1. 必须是 Suite 类型
    if suite.plugin_type != Some(PluginType::Suite) { ... }

    // 2. 必须声明至少一个原子
    if suite.atoms.is_empty() { ... }

    // 3. 所有引用的原子必须已在注册表中
    for atom in &suite.atoms {
        if !self.manifests.contains(atom.atom_id) {
            errors.push(format!("原子 {} 未注册", atom.atom_id));
        }
    }
}
```

### 6.6 插件市场客户端

```rust
pub struct PluginMarketClient { repo_url: String }

impl PluginMarketClient {
    // 获取市场索引
    pub async fn fetch_index(&self) -> Result<MarketMetadata, String> {
        let resp = reqwest::get("{repo_url}/index.json").await?;
        serde_json::from_str(&resp.text().await?)
    }

    // 下载并校验单个插件
    pub async fn fetch_manifest(&self, summary: &PluginSummary) -> Result<PluginManifest, String> {
        let resp = reqwest::get(&summary.download_url).await?;
        let manifest: PluginManifest = serde_json::from_str(&resp.text().await?)?;
        manifest.validate()?;  // 本地协议校验
        Ok(manifest)
    }
}
```

---

## 七、前端架构详解

### 7.1 Adobe 暗色面板设计系统

完整设计令牌定义在 `design-system.css` 的 `:root` 块中。系统包含约 50 个 CSS 变量。

**配色体系**:

| 令牌 | 值 | 用途 |
|------|------|------|
| `--ad-bg` | `#0d0d0d` | 全局背景 |
| `--ad-panel` | `#1a1a1a` | 面板背景 |
| `--ad-card` | `#242424` | 卡片背景 |
| `--ad-border` | `#4a4a4a` | 边框 |
| `--ad-text` | `#e6e6e6` | 主文本 |
| `--ad-text-secondary` | `#aaaaaa` | 次要文本 |
| `--ad-text-muted` | `#9e9e9e` | 弱化文本 |
| `--ad-accent` | `#1473e6` | Adobe 蓝 (唯一允许的强调色) |
| `--ad-success` | `#6b9e7a` | 鼠尾草绿 |
| `--ad-error` | `#c48888` | 玫瑰灰 |
| `--ad-warning` | `#c4a55a` | 琥珀金 |

**禁止颜色**: 高饱和绿 (`#22c55e`)、高饱和红 (`#ef4444`)、高饱和橙 (`#f59e0b`)、高饱和蓝 (`#3b82f6`)、纯色 (`#00ff00`)。

**圆角约束**: 组件最大 6px, 面板/卡片 2-4px, 输入框/按钮 2px。禁止 `> 8px` 或 `999px` 完全圆形。

**背景约束**: 全局纯色 `#0d0d0d`, 禁止渐变、禁止毛玻璃 (`backdrop-filter: blur`)。

### 7.2 App Shell 布局

```
┌──────────────────────────────────────────────────────────────┐
│  ad-titlebar (fixed, top:0, h:32px, z:200, bg:#0a0a0a)      │
│  [QuantPilot]                                 [—] [□] [✕]    │
├────┬─────────────────────────────────────────────────────────┤
│ QP │  ad-main-content (margin-left:48px, overflow-y:auto)    │
│    │                                                         │
│ 策 │  ┌─ TopToolbar ──────────────────────────────────────┐  │
│ 略 │  │ [教程] [凭证] [新建] [保存] [导出] │ 编译 模拟 回测 │  │
│    │  └──────────────────────────────────────────────────┘  │
│ QS │  ┌─ ad-tabbar ───────────────────────────────────────┐  │
│    │  │ [仪表盘] [画布] [研究] [源码]           [打开回测] │  │
│ 审 │  └──────────────────────────────────────────────────┘  │
│ 批 │                                                         │
│    │  ┌─ Page Content ────────────────────────────────────┐  │
│ ...│  │                                                     │  │
│    │  └───────────────────────────────────────────────────┘  │
├────┴─────────────────────────────────────────────────────────┤
│  ad-sidebar (fixed, left:0, w:48px→160px hover, z:100)       │
│  [QP] [策] [QS] [审] [告] [快] [故] [混]                     │
└──────────────────────────────────────────────────────────────┘
```

### 7.3 策略中心页 (StrategyHubPage)

默认首页 (`/strategies`)。职责: 策略目录、全局管理入口。

**布局**:

```
StrategyHubPage
├── TopToolbar (凭证入口 + 编译/运行按钮)
├── StrategyHubHeroSection
│   ├── 策略中心说明 (inline note 悬浮说明)
│   ├── 管理任务组: [刷新活动] [同步最新策略图]
│   ├── 构建任务组: [打开当前工作区] [打开空白工作区]
│   ├── 状态条: 策略文件数 / 可运行数 / 可研究数 / 最近活动
│   └── 操作卡片: 待修复 / 运行就绪 / 对比队列 / 已选策略
├── StrategyHubBodySection
│   ├── StrategyHubTemplateLibrarySection (策略模板库: 折叠态/展开态)
│   ├── StrategyHubRosterSection (策略清单表: 全量可滚动列表, 不做搜索/筛选/排序)
│   └── StrategyHubInspectorSection (策略驾驶舱: 选中策略的操作面板)
```

### 7.4 策略工作台页 (StrategyWorkspacePage)

单策略编辑和分析的主界面 (`/strategies/:id`)。

**标签页**:

| 标签 | 组件 | 功能 |
|------|------|------|
| 仪表盘 | StrategyWorkspaceDashboard | 编译状态、运行状态、快速操作、版本历史、近期运行/回测摘要 |
| 画布 | StrategyWorkspaceCodeTab | 策略图编辑 (React Flow)、属性面板 (配置/检查/源码)、诊断定位 |
| 研究 | StrategyWorkspaceResearchTab | 事件流、运行/回测控制、研究控制台 |
| 源码 | StrategyWorkspaceSourceTab | 原始 QuantScript 源码查看和测试运行 |

**画布标签 (CodeTab) 的结构**:

```
StrategyWorkspaceCodeTab
├── workspace-inspector-nav (配置 | 检查 | 源码)
├── workspace-task-lanes-section
│   ├── 主通道: 当前激活的 inspector panel
│   └── 辅助通道 (可展开): 其他两个 panel
├── StrategyCanvas (React Flow 画布)
│   ├── 数据源节点
│   ├── 意图节点
│   ├── 代理节点
│   ├── 风控节点
│   ├── 执行节点
│   ├── 运行时控制节点
│   └── 连线 (edges)
└── 画布推荐 + 修复路径面板
```

### 7.5 状态管理 (Zustand)

`useGraphStore` 是全局状态中心。核心状态切片:

```javascript
{
  // 策略图
  graph: {
    metadata: { graph_id, name, version, source_mode, ... },
    nodes: [{ id, type, module_key, config, position, ... }],
    edges: [{ id, source_node_id, target_node_id, source_port, target_port }],
    validation_state: { is_valid, is_runnable, issue_counts, graph_issues, node_issues, edge_issues },
    compile_summary: { compilable, protocol_name, config_hash, counts, diagnostics },
    artifacts: { quantscript: { graph_source, formal_source } }
  },

  // 运行时
  runtime: {
    status: "idle" | "running" | "stopped" | "error",
    runKind: "simulation" | "backtest",
    history: [RunRecord],
    backtestHistory: [BacktestRecord],
    events: [RuntimeEvent],
    account: { cash_balance, available_cash_balance, frozen_cash_balance, open_orders },
    diagnostics: { risk: [...], order: [...], dataQuality: [...] },
    backendError: String | null
  },

  // 能力
  capabilities: { api_version, schema_hash, strategy_ir, runtime, market_data, frontend, ... },
  capabilityStatus: "loading" | "ready" | "degraded" | "error",
  capabilitySource: "remote" | "cache" | "safe_fallback",

  // UI
  selectedNodeId: String | null,
  selectedEdgeId: String | null,
  formalQuantScriptOverride: String | null,
  graphIndex: [{ graph_id, name, updated_at, path }],
  graphAuditHistory: [AuditEntry]
}
```

### 7.6 凭证管理组件

`CredentialInput.jsx` — v1.0.0 重构为单 textarea 输入:

1. 用户在 `<textarea>` 中粘贴完整的凭证 JSON 或 key=value 文本
2. `parseCredentialBlock()` 函数解析输入: 优先尝 JSON.parse，失败则按 `key=value` 逐行解析
3. 解析成功后展示字段预览 (字段名 + 前 4 个字符 + `****`)
4. 点击保存 → `POST /api/credentials` → `{ service: label, fields: parsed_map }`
5. 后端 `CredentialVault::set_service()` 将整个 field_map 序列化为 JSON → AES-256-GCM → 原子写入
6. 组件卸载时 `useEffect` cleanup 清零 React state 中的凭证明文

---

## 八、存储系统详解

### 8.1 三级生命周期

`storage/` 下的每个子目录在 `StorageLifecycle` 中被分类:

```rust
pub enum StorageLifecycle {
    Permanent,   // 无 TTL, 仅用户显式删除
    Temporary,   // 7 天 TTL (DEV 模式 1 天)
    Transient,   // 1 小时 TTL (DEV 模式 10 分钟)
}
```

**分类表**:

| 目录 | 生命周期 | 写入保护 | 启动清理 |
|------|:--:|------|------|
| `graphs/` | Permanent | 全局配额豁免 | 不清理 |
| `audit/` | Permanent | 全局配额豁免 | 不清理 |
| `.credentials` | Permanent | 全局配额豁免 | 不清理 |
| `.machine_key` | Permanent | 全局配额豁免 | 不清理 |
| `runs/` | Temporary | ≤200MB / 受全局配额 | TTL 过期后清理 |
| `backtests/` | Temporary | ≤200MB / 受全局配额 | TTL 过期后清理 |
| `experiments/` | Temporary | ≤200MB / 受全局配额 | TTL 过期后清理 |
| `reports/` | Temporary | ≤200MB / 受全局配额 | TTL 过期后清理 |
| `mutations/` | Temporary | ≤200MB / 受全局配额 | TTL 过期后清理 |
| `approvals/` | Temporary | ≤200MB / 受全局配额 | TTL 过期后清理 |
| `snapshots/` | Transient | ≤50MB / 受全局配额 | TTL 过期后强制清理; DEV 模式启动时全部清理 |
| `alerts/` | Transient | ≤50MB / 受全局配额 | 同上 |
| `chaos/` | Transient | ≤50MB / 受全局配额 | 同上 |
| `sandbox-reports/` | Transient | ≤50MB / 受全局配额 | 同上 |
| `ai-proposals/` | Transient | ≤50MB / 受全局配额 | 同上 |

### 8.2 配额执行算法

每次写入前调用 `ensure_storage_quota()`:

```
ensure_storage_quota(storage_root, dir_name, lifecycle)
  │
  ├── 1. 计算 storage/ 总大小 (递归遍历所有文件和子目录)
  │
  ├── 2. 全局配额检查
  │      if total > REJECT_AT_BYTES (475 MB)
  │        → 返回错误: "存储空间已满: 当前 {} MB, 上限 500 MB"
  │      if total > WARN_AT_BYTES (400 MB)
  │        → safe_eprintln 告警
  │
  ├── 3. 每目录配额检查
  │      Permanent → 跳过 (无上限)
  │      Temporary → 上限 200 MB
  │      Transient → 上限 50 MB
  │      超过上限 → 返回错误: "目录 {} 已满: 当前 {} MB, 上限 {} MB"
  │
  └── 4. 通过 → 允许写入
```

### 8.3 启动清理算法

每次服务器启动时执行 `startup_storage_cleanup()`:

```
startup_storage_cleanup(storage_root)
  │
  ├── 遍历 storage/ 下每个子目录
  │
  ├── 对每个目录:
  │     │
  │     ├── DEV 模式 + Transient → 删除目录下全部文件 (不限 TTL)
  │     │
  │     ├── 有 TTL 的目录 (Temporary/Transient):
  │     │     遍历目录下每个文件:
  │     │       if file_age > ttl + safety_margin (10 分钟):
  │     │         → 删除
  │     │       如果总存储 > 450 MB → safety_margin = 0 (激进清理)
  │     │
  │     └── Permanent → 跳过
  │
  ├── 如果 total_size > 400 MB → 日志告警
  ├── 如果 total_size > 450 MB → 日志严重告警 + 激进清理已执行
  └── 如果清理了任何文件 → 日志记录: "启动清理: 删除 {} 项, 释放 {} KB"
```

### 8.4 DEV 模式差异

当环境变量 `QUANTPILOT_DEV=true` 时:

| 参数 | 正常模式 | DEV 模式 |
|------|---------|---------|
| Temporary TTL | 7 天 | 1 天 |
| Transient TTL | 1 小时 | 10 分钟 |
| 启动时 Transient 清理 | 按 TTL | 全部删除 (不限 TTL) |
| API 认证 | 需要 Bearer token | 跳过 |
| 安全日志脱敏 | 全面脱敏 | 同上 |

### 8.5 凭证保险库 (CredentialVault)

加密算法: AES-256-GCM (ring crate)。

密钥派生: `SHA-256(hostname + machine_key)`。

存储文件: `storage/.credentials` (整体加密, 原子写入)。

写入流程:
```
set_service("okx_testnet", {"api_key": "...", "secret": "...", "passphrase": "..."})
  │
  ├── 1. 构造 VaultData { entries: BTreeMap<Label, BTreeMap<Key, SecretString>> }
  │
  ├── 2. serde_json::to_vec → JSON 字节
  │
  ├── 3. AES-256-GCM 加密
  │      密钥 = SHA-256(hostname + machine_key)
  │      nonce = 随机 12 字节
  │      AAD = "storage/.credentials" (绑定密文到文件路径, 防止密文被移动到其他位置)
  │
  ├── 4. 原子写入
  │      write(.tmp) → fsync → rename(.tmp → .credentials) → fsync
  │      旧文件重命名为 .credentials.bak (回滚用)
  │
  └── 5. 机器密钥保护
         storage/.machine_key:
           OnceLock 保护 (首次生成后不再变化)
           权限: 仅当前用户可读 (Windows ACL)
           加入 .gitignore
```

读取流程:
```
get_service("okx_testnet")
  │
  ├── 1. 读取 storage/.credentials (如损坏则尝试 .bak)
  │
  ├── 2. AES-256-GCM 解密
  │
  ├── 3. serde_json::from_slice → VaultData
  │
  ├── 4. 查找 "okx_testnet" 条目
  │
  └── 5. 将字段值包装为 Zeroizing<String>
         使用完后主动 drop (触发 zeroize 覆写内存)
```

---

## 九、超级规范化详解

完整文档: `markdown/01-principles/principles-super-standardization.md`

### 9.1 五条流水线

**设计流水线**: 触发条件: 新增功能 / API 路由变更 / 插件协议变更 / ≥3 文件变更。输出: 设计文档 (目标/非目标/方案/验收/风险/决策纪录)。

**开发流水线**: Pre-commit hook 自动执行 staged-file 智能分流。docs-only 默认跑治理文档门禁，rust-only 默认跑 `cargo fmt --check` 与 `cargo check`，frontend-only 默认跑 `vite build` 与 `vitest run`，任何失败拒绝提交；可用 `QUANTPILOT_PRECOMMIT_FULL=1` 强制全量 legacy gate。

**检查流水线**: 10 项自动化门禁。通过 `.github/workflows/ci.yml` 在 push/PR 时触发, 或本地 `tools/run-closeout-gates.bat` 一键执行。

**审计流水线**: 每里程碑 closeout 时执行五维度评分 (功能开发进度/仓库稳定程度/发布就绪度/用户友好程度/系统整体稳定性) + General_Policy §1-§8 合规矩阵。

**优化流水线**: 审计发现按 S0/P1/P2 优先级转化为下个里程碑的优化项。

### 9.2 元流水线

元流水线是优化流水线的自审计层。持续追踪:

- 门禁脚本完整性 (`check-gates-smoke.ps1` 每月)
- 测试覆盖率趋势 (`#[test]` 计数变化)
- 门禁耗时 (每次 CI 总耗时, 超 10 分钟需分析)
- 误报跟踪 (被手动 override 的门禁失败)
- 策略检查工具的误报率和漏报率

数据记录到 `storage/audit/gate-metrics.json`。

### 9.3 CI 流水线 (13 步)

```yaml
steps:
  - checkout
  - setup-node (20)
  - setup-rust (stable)
  - check-utf8              # tools/check-utf8.ps1
  - check-user-facing-text  # tools/check-user-facing-text.ps1
  - check-capability-gov    # tools/check-capability-governance.ps1
  - check-i18n              # tools/check-i18n.ps1
  - cargo check --workspace
  - cargo clippy -- -D warnings
  - cargo test --workspace
  - npm ci
  - npm run build
  - npm run test
  - npm audit --audit-level=moderate
```

---

## 十、API 参考 (完整)

### 10.1 编译

```
POST /api/runtime/compile
Content-Type: application/json

Request:
{
  "graph_json": { ... },        // 策略图 JSON (必填, §5.4)
  "runtime_config": {           // 前端运行时配置 (必填)
    "metadata": {
      "graph_id": "graph_xxx",
      "compile_id": "compile_xxx",
      "name": "策略名称",
      "version": "1",
      "mode": "paper"
    },
    "data_sources": [{ "id": "...", "module_key": "builtin.data.kline", "config": {...} }],
    "intent_generators": [{ "id": "...", "module_key": "builtin.intent.double_ma", "config": {...} }],
    "agents": [{ "id": "...", "module_key": "builtin.agent.weighted", "config": {...} }],
    "risks": [{ "id": "...", "module_key": "builtin.risk.global", "config": {...} }],
    "executions": [{ "id": "...", "module_key": "builtin.execution.paper", "config": {...} }],
    "runtime_control": { "id": "runtime", "module_key": "builtin.runtime.control" }
  }
}

Response 200:
{
  "graph_id": "graph_xxx",
  "compile_id": "compile_xxx",
  "compilable": true,
  "protocol_name": "quantpilot/runtime-config/v1",
  "config_hash": "sha256:...",
  "core_ir": { ... },
  "artifacts": {
    "strategy": { "artifact_id": "...", "artifact_kind": "strategy", ... },
    "compile": { "artifact_id": "...", "artifact_kind": "compile", ... },
    "core_ir": { "artifact_id": "...", "artifact_kind": "core_ir", ... }
  },
  "counts": {
    "data_sources": 1,
    "intent_generators": 1,
    "agents": 1,
    "risk_controls": 1,
    "executions": 1
  },
  "diagnostics": [
    {
      "code": "QSPIPELINE",
      "severity": "Warning",
      "message": "QS 管道编译通过: 1 个数据源, 1 个意图, 1 个代理, 1 个风控",
      "target": null,
      "span_label": null,
      "hint": null
    }
  ],
  "runtime_config": { ... },
  "runtime_targets": {
    "source_to_node": { "binance_btc_1h": "data_binance_btc" },
    "runtime_node_id": "runtime",
    "execution_node_id": "exec_paper"
  }
}

Error 400: { "error_code": "capability_gated", "message": "运行时配置使用了当前 Beta 版本未启用的能力", "details": [...] }
Error 400: { "error_code": "runtime_compile_failed", "message": "运行时图编译合约校验失败", "details": [...] }
Error 400: { "error_code": "qs_generation_failed", "message": "从图生成 QS 源码失败: ..." }
Error 400: { "error_code": "qs_lowering_failed", "message": "QS 下层转换失败: ..." }
```

### 10.2 Paper 运行

```
POST /api/runtime/test-run
Content-Type: application/json

Request:
{
  "graph_json": { ... },
  "runtime_config": { ... },
  "runtime_targets": { ... },
  "capability_context": {
    "schema_hash": "sha256:...",
    "permission_boundary": { ... }
  },
  "actor": "user",
  "backtest_options": null
}

Response 200:
{
  "run_id": "run_1710000000000",
  "graph_id": "graph_xxx",
  "compile_id": "compile_xxx",
  "protocol_name": "quantpilot/runtime-config/v1",
  "config_hash": "sha256:...",
  "event_count": 42,
  "account": {
    "cash_balance": 9500.0,
    "available_cash_balance": 9000.0,
    "frozen_cash_balance": 500.0,
    "open_orders": [...],
    "open_order_count": 1,
    "equity": 10000.0,
    "positions": { "BTCUSDT": 0.01 }
  },
  "governance": {
    "capability_hash": "sha256:...",
    "deployment_revision": "strategy_v1_compile_xxx_cap_sha256:..."
  },
  "events": [ ... ]
}

SSE Stream (Server-Sent Events):
  event: runtime_event
  data: { "event_id": "...", "event_type": "DataUpdated", ... }

  event: runtime_event
  data: { "event_id": "...", "event_type": "IntentComputed", ... }

  event: runtime_complete
  data: { "run_id": "run_1710000000000", "event_count": 42 }
```

### 10.3 回测

```
POST /api/runtime/backtest
Content-Type: application/json

Request:
{
  "graph_json": { ... },
  "runtime_config": { ... },
  "runtime_targets": { ... },
  "capability_context": { ... },
  "actor": "user",
  "backtest_options": {
    "replay_source": "historical",         // "historical" | "deterministic_mock"
    "execution_assumptions": {
      "fee_bps": 10,
      "slippage_bps": 5,
      "latency_assumption_ms": 100,
      "time_in_force": "gtc"
    }
  }
}

Response 200:
{
  "backtest_id": "backtest_1710000000000",
  "graph_id": "graph_xxx",
  "compile_id": "compile_xxx",
  "protocol_name": "quantpilot/runtime-config/v1",
  "config_hash": "sha256:...",
  "event_count": 500,
  "account": { ... },
  "backtest_artifacts": {
    "manifest": { "strategy_artifact_id": "...", "compile_artifact_id": "...", ... },
    "equity_curve": [{ "cycle": 0, "equity": 10000.0 }, { "cycle": 1, "equity": 10015.0 }, ...],
    "summary": {
      "total_return_ratio": 0.125,
      "max_drawdown_ratio": 0.02,
      "sharpe_ratio": 1.8,
      "win_rate": 0.65,
      "trade_count": 15,
      "final_equity": 11250.0
    }
  }
}
```

### 10.4 能力发现

```
GET /api/capabilities

Response 200: (见 §3.7 完整示例)
```

### 10.5 凭证管理

```
GET /api/credentials
Response 200: { "services": ["okx", "binance"] }

POST /api/credentials
Content-Type: application/json
Request: {
  "service": "okx",
  "fields": {
    "api_key": "8a3f2b1c-...",
    "secret": "b2c1d0e9-...",
    "passphrase": "MyPass123"
  }
}
Response 200: { "service": "okx", "stored": true }

DELETE /api/credentials/okx
Response 200: { "service": "okx", "deleted": true }
```

### 10.6 图管理

```
POST /api/graphs
Request: { "graph_json": { ... } }
Response 200: { "graph_id": "graph_xxx", "path": "storage/graphs/graph_xxx.json" }

GET /api/graphs/:id
Response 200: { "graph_json": { ... }, "graph_id": "graph_xxx", "path": "..." }

GET /api/graphs/:id/versions
Response 200: { "versions": [{ "version_id": "v1", "created_at_ms": ... }, ...] }

POST /api/graphs/:id/versions/:vid/restore
Response 200: { "restored": true, "version_id": "v1" }
```

### 10.7 通用错误码

| 错误码 | HTTP | 说明 |
|--------|:---:|------|
| `bad_request` | 400 | 请求格式错误或缺少必填字段 |
| `capability_gated` | 400 | 使用了当前 Beta 版本未启用的能力 |
| `capability_boundary_violation` | 400 | 能力边界违规 |
| `runtime_compile_failed` | 400 | 运行时图编译合约校验失败 |
| `qs_generation_failed` | 400 | 从图生成 QS 源码失败 |
| `qs_parse_failed` | 400 | QS 解析失败 |
| `qs_lowering_failed` | 400 | QS 下层转换失败 |
| `internal_error` | 500 | 服务器内部错误 |

---

## 十一、使用指南

### 11.1 快速启动

**方式一: 桌面应用 (推荐)**

```bat
.\start.bat
```

Tauri 自动编译并启动后端 (127.0.0.1:3000) + 前端 Dev Server (127.0.0.1:5173)，在桌面窗口中加载完整应用。

依赖: Rust 工具链 (rustup) + Node.js 18+ + WebView2 (Windows 11 已内置，Windows 10 需单独安装)。

首次启动可能需要 5-10 分钟编译 Rust 依赖。后续启动只需增量编译，通常在 30 秒内完成。

**方式二: 分开启动 (开发/调试)**

```powershell
# 终端 1: 后端
cargo run
# 看到 "listening on 127.0.0.1:3000" 表示就绪

# 终端 2: 前端
cd frontend
npm install        # 仅首次
npm run dev
```

浏览器打开 `http://127.0.0.1:5173`。

### 11.2 创建第一个策略 (完整步骤)

**步骤 1: 进入工作台**

启动应用后默认进入策略中心页。点击右上角「打开空白工作区」按钮，进入策略工作台。

**步骤 2: 添加数据源**

左侧面板「数据」分组中，拖拽「K 线数据」到画布。点击该节点，右侧属性面板显示配置项:
- 交易所: binance 或 okx
- 交易对: BTCUSDT / ETHUSDT / SOLUSDT
- K 线周期: 1m / 5m / 15m / 30m / 1h / 4h / 1d
- 窗口大小: 回看多少根 K 线 (默认 200)
- Ping 探测: 是否在数据请求前探测延迟

**步骤 3: 添加意图**

左侧面板「意图」分组中，选择一种意图 (如「双均线」) 拖入画布。点击节点配置:
- 快线周期: 7 (默认)
- 慢线周期: 25 (默认)
- 均线方法: EMA / SMA

**步骤 4: 添加代理**

拖入「加权代理」节点。此节点将意图信号转化为组合目标。配置:
- 再平衡分配类型: fixed_weights / equal_weight / score_weight
- 再平衡标的列表: 从数据源自动继承

**步骤 5: 添加风控**

拖入「全局风控」节点。配置:
- 最大持仓比例: 0.5 (总权益的 50%)
- 单一标的最大权重: 0.3
- 最大总杠杆: 2.0
- 最小操作间隔: 60000 (ms, 即 1 分钟)

**步骤 6: 添加执行**

拖入「Paper 执行」节点。配置:
- 手续费: 10 (bps)
- 滑点: 5 (bps)
- 时效: GTC

**步骤 7: 连线**

从数据源节点的输出端口 (蓝色圆点) 拖线到意图节点的输入端口。依次连接: data → intent → agent → risk → execution → runtime_control。

**步骤 8: 保存**

点击顶部「保存策略图」。策略图被保存到 `storage/graphs/{graph_id}.json`。同时生成 QS 源码。

### 11.3 编译与运行

**编译检查**:

点击顶部「编译」按钮。系统执行:
1. 能力校验: 检查所有模块是否在 `/api/capabilities` 白名单中
2. 合约诊断: 检查 graph_id、节点完整性、连线一致性
3. QS 管道编译: 图→源码→AST→HIR→Core IR

成功时顶部状态栏显示绿色 `QSPIPELINE` 标签。检查标签页可查看完整诊断列表。

编译失败时的常见原因:
- 节点未连线: 检查每个节点的输入端口都有来源
- 模块不存在: 检查模块 key 是否正确 (如 `builtin.intent.double_ma` 不是 `builtin.intent.ma_cross`)
- 参数不合法: 如快线周期 > 慢线周期

**Paper 运行**:

编译通过后，点击「启动模拟」。后端创建一个 `RealTimeSandbox` 实例，开始实时数据获取和策略执行。

运行期间:
- 左侧「研究」标签页实时显示事件流
- 每个事件可展开查看详细 payload
- 账户卡片显示当前现金、持仓、未结订单
- 回测历史卡片显示本次运行的摘要数据

点击「停止」可随时终止运行。运行记录自动保存到 `storage/runs/run_{timestamp}.json`。

**回测**:

点击「运行回测」。系统使用 `FastBacktestSandbox` 快速回放历史数据:
- 默认使用 `HistoricalReplay` 模式: 从本地 `storage/data/` 加载历史 K 线缓存。如果缓存缺失，自动从交易所 API 拉取并缓存
- 可选 `DeterministicMock` 模式: 使用固定种子生成伪随机 K 线，确保每次回测结果完全一致 (离线可用)

回测完成后自动跳转至回测详情页。查看:
- 权益曲线图表
- 总收益率、最大回撤、夏普比率、胜率、交易次数
- 治理身份 (capability_hash, deployment_revision)
- 每笔交易的时间线 (风险检查→订单创建→成交)

### 11.4 管理凭证

QuantPilot 使用 AES-256-GCM 加密存储交易所 API 凭证。

1. 点击顶部工具栏「凭证」按钮 (策略中心和工作台都可见)
2. 在弹出的凭证管理面板中:
   - 如果是首次使用，列表为空。点击「新增凭证」
   - 如果已有凭证，可点击「编辑」修改或「删除」移除
3. 在文本框中粘贴交易所提供的完整凭证内容。支持两种格式:

```json
{
  "api_key": "8a3f2b1c-4d5e-6f7a-8b9c-0d1e2f3a4b5c",
  "secret": "b2c1d0e9f8a7b6c5d4e3f2a1b0c9d8e7",
  "passphrase": "MyTradingPass123"
}
```

或逐行 key=value 格式:

```
api_key=8a3f2b1c-4d5e-6f7a-8b9c-0d1e2f3a4b5c
secret=b2c1d0e9f8a7b6c5d4e3f2a1b0c9d8e7
passphrase=MyTradingPass123
```

4. 点击「已解析 N 个字段，点击预览」可查看识别到的字段 (前 4 个字符 + **** 遮蔽)
5. 点击「保存」。凭证被整体加密写入 `storage/.credentials`
6. 点击「关闭」退出面板。面板关闭时 React state 中的凭证明文被主动清零

凭证使用示例 (在 test_runner 中):
```rust
let vault = CredentialVault::load()?;
let fields = vault.get_service("okx")?;
// fields = {"api_key": "...", "secret": "...", "passphrase": "..."}

// OKX API 请求需要三个 HTTP 头:
// OK-ACCESS-KEY: api_key
// OK-ACCESS-SIGN: base64(HMAC-SHA256(timestamp+method+path+body, secret))
// OK-ACCESS-PASSPHRASE: passphrase
```

### 11.5 策略对比

1. 对同一策略使用不同参数各运行一次回测 (如均线周期 7/25 vs 14/50)
2. 在策略工作台的「研究」标签中，找到回测历史卡片
3. 勾选两条回测记录的复选框
4. 点击「打开对比」按钮
5. 对比页并排展示:

| 指标 | 策略 A (7/25) | 策略 B (14/50) |
|------|:----------:|:----------:|
| 总收益率 | +12.50% | +8.30% |
| 最大回撤 | -2.00% | -3.50% |
| 夏普比率 | 1.80 | 1.20 |
| 胜率 | 65% | 58% |
| 交易次数 | 15 | 8 |

对比页同时显示:
- 配置差异 (哪些参数不同)
- 数据集差异 (回测数据源是否一致)
- 权益曲线叠加图

### 11.6 抽象语法树 (QS) 编写 QuantScript

除图形编辑器外，系统还提供 QuantScript 源码编辑器 (路由 `/quantscript`)。

编写 QS 源码后，可执行:
- **形式化分析**: `analyze_formal_quant_script()` — 语法检查 + 语义分析
- **编译**: `lower_script_to_runtime_config()` — QS → Core IR
- **测试运行**: QS 源码中可嵌入 `@test` / `@compile` / `@backtest` / `@assert` 指令

示例 QS 源码:

```quantascript
import fetch, indicator, agent, risk, execution from "quantpilot"

data = fetch("binance", "BTCUSDT", "1h")

ma7 = indicator("ma", source=data.close, period=7, method="ema")
ma25 = indicator("ma", source=data.close, period=25, method="ema")

def crossover(a, b, idx):
    return a[idx] > b[idx] and a[idx-1] <= b[idx-1]

def strategy():
    risk.profile("global",
        max_position=0.5,
        max_single_weight=0.3,
        max_total_leverage=2.0,
        min_action_interval_ms=60000
    )
    execution.profile("paper",
        fee_bps=10,
        slippage_bps=5
    )
    emit Intent("buy") when crossover(ma7, ma25)
    emit Intent("sell") when crossover(ma25, ma7)
```

### 11.7 CLI 命令行工具

```powershell
# 编译策略 IR
cargo run -- strategy-ir compile --input strategy.json

# 管理凭证 (交互式输入, 不从命令行传明文)
cargo run -- credential set okx
# 请输入 api_key: [隐匿输入]
# 请输入 secret: [隐匿输入]
# 请输入 passphrase (可选): [隐匿输入]

cargo run -- credential list
# 输出: okx, binance

cargo run -- credential get okx
# 输出: {"api_key": "8a3f***", "secret": "b2c1***", "passphrase": "MyPa***"}

cargo run -- credential delete okx
```

### 11.8 QS 场景测试

在 `tests/scenarios/` 目录创建 `.qs` 场景文件:

```quantascript
@test "双均线策略回归测试"
@compile {
    graph_id: "btc_dual_ma_stability"
}
@backtest {
    save: true,
    replay_source: "deterministic_mock"
}
@assert {
    event_count > 0,
    has_fills: true,
    total_return > -0.10
}
```

场景文件通过 `cargo test` 自动执行。每个 `@assert` 失败时输出详细诊断。

### 11.9 管理插件

**安装本地原子插件**:

1. 编写 manifest JSON 文件
2. 放入 `plugins/atoms/` 目录
3. 重启应用 → 启动时自动扫描注册
4. 验证: `GET /api/capabilities` → 检查 `strategy_ir.supported_indicator_kinds` 是否包含新插件

**创建套件**:

```json
{
  "api_version": "quantpilot/plugin-manifest/v1",
  "id": "suite.okx_btc_ma_trend",
  "version": "1.0.0",
  "plugin_type": "suite",
  "kind": "intent",
  "display": { "name": "OKX BTC 均线趋势", "summary": "OKX BTC 双均线 + 加权代理 + 全局风控" },
  "atoms": [
    { "atom_id": "quantpilot.data.kline_okx", "version": "0.1.0", "kind": "data" },
    { "atom_id": "quantpilot.intent.double_ma", "version": "0.1.0", "kind": "intent" },
    { "atom_id": "quantpilot.agent.weighted", "version": "0.1.0", "kind": "agent" },
    { "atom_id": "quantpilot.risk.global", "version": "0.1.0", "kind": "risk" },
    { "atom_id": "quantpilot.execution.paper", "version": "0.1.0", "kind": "execution" }
  ],
  "extension_points": ["intent_module_provider"],
  ...
}
```

套件打包时校验所有 atom 已注册且 exchange/symbol 一致。套件自身不含策略逻辑，仅声明组合。

### 11.10 存储管理

```powershell
# 查看存储使用情况
du -sh storage/
# 典型输出: 45M    storage/

# 查看各级别占用
du -sh storage/runs/ storage/backtests/ storage/snapshots/

# 干运行清理 (仅预览, 不删除)
powershell -File tools/cleanup-artifacts.ps1

# 实际执行清理
powershell -File tools/cleanup-artifacts.ps1 -Mode execute

# 清理运行工件 + 日志
powershell -File tools/cleanup-artifacts.ps1 -Mode execute -IncludeRuntimeArtifacts -IncludeLogs

# 清理 30 天前的数据
powershell -File tools/cleanup-artifacts.ps1 -OlderThanDays 30 -Mode execute
```

### 11.11 环境变量完整参考

| 变量 | 用途 | 默认值 | 生效位置 |
|------|------|--------|---------|
| `QUANTPILOT_DEV` | 开发模式: 缩短 TTL, 强制清理瞬态, 跳过 API 认证 | 空 | 后端 |
| `QUANTPILOT_API_KEY` | API 认证密钥 | 随机生成 16 字节 hex | 后端 |
| `QUANTPILOT_STORAGE_WATERMARK_MB` | 存储告警阈值 | 400 | 后端 |
| `VITE_BACKEND_ORIGIN` | Vite dev proxy 后端地址 | `http://127.0.0.1:3000` | 前端 |
| `VITE_API_BASE_URL` | 浏览器直连 API 地址 | 从当前 origin 派生 `/api` | 前端 |
| `HTTPS_PROXY` | (已废弃, v0.4.1 移除 set_var 全局副作用) | — | — |

### 11.12 故障排查完整指南

| 症状 | 可能原因 | 排查步骤 |
|------|---------|---------|
| 编译失败: "策略必须包含至少一个意图" | 策略图中没有 intent 节点 | 在画布中添加至少一个意图节点 |
| 编译失败: "capability_gated" | 使用了不在白名单中的模块 key | 检查 `/api/capabilities` 响应，确认模块 key 在 `frontend.supported_module_keys` 中 |
| 编译失败: "运行时图编译合约校验失败" | 节点连线不完整、graph_id 无效 | 检查每个节点的输入端口都连了线；检查 graph_id 不含非法字符 |
| Paper 运行无事件 | 数据源配置错误 (交易所/交易对/K 线周期) | 检查数据源节点的配置；查看 DevTools Network 标签看是否有 API 请求发出 |
| Paper 运行事件很少 | K 线周期太长 (如 1d)，数据更新频率低 | 切换到更短的周期 (如 1m 或 5m) |
| 回测数据为空 | 历史数据缓存缺失 | 先执行一次 Paper 运行 (会拉取并缓存数据)，再执行回测 |
| 回测使用 HistoricalReplay 失败 | 本地缓存中没有对应时间范围的历史数据 | 使用 `backtest_options.replay_source = "deterministic_mock"` |
| 凭证面板无法加载 | 后端未运行；API 路由未就绪 | `curl http://127.0.0.1:3000/api/health` 检查后端是否在线 |
| 凭证保存后仍提示未设置 | `storage/.machine_key` 损坏或丢失 | 删除 `storage/.machine_key` 和 `storage/.credentials`，重新设置凭证 |
| 侧边栏 QP logo 被标题栏遮挡 | v0.5.2 之前的 CSS bug | 升级到 v0.5.2+ |
| 前端白屏 | 前端构建产物损坏或 Vite dev server 未启动 | `cd frontend && npm run build` 重新构建；或检查 `npm run dev` 输出 |
| `cargo test` 编译失败 | 新增字段未在所有构造点补齐 | 搜索 `CoreStrategyIr {` 确保所有构造点都有 `edges: vec![]` 字段 |
| 存储空间告警 (>400MB) | 运行/回测记录积累 | 运行 `cleanup-artifacts.ps1 -Mode execute -IncludeRuntimeArtifacts` |
| 存储配额拒绝写入 (>475MB) | 同上 | 同上；或手动删除 `storage/runs/` 中的旧 JSON 文件 |
| npm audit 失败 | 前端依赖有已知漏洞 | `cd frontend && npm audit fix` |
| Pre-commit hook 拒绝提交 | cargo check / test / build / vitest 任一失败 | 修复后重新提交；紧急情况用 `git commit --no-verify` 但需在下个里程碑补全检查 |

### 11.13 开发环境搭建

```powershell
# 前置条件
# - Rust toolchain: https://rustup.rs
# - Node.js 18+: https://nodejs.org
# - Git: https://git-scm.com
# - Windows 11 (含 WebView2) 或 Windows 10 + WebView2 Runtime

# 验证前置条件
rustc --version   # ≥ 1.75
node --version    # ≥ 18.0
cargo --version   # ≥ 1.75

# 克隆仓库
git clone https://github.com/han243786/Quantpilot.git
cd quantpilot

# 安装前端依赖
cd frontend
npm install

# 编译后端
cd ..
cargo build

# 运行全部测试
cargo test --workspace
cd frontend && npm run test

# 安装 pre-commit hook
cp scripts/pre-commit .git/hooks/pre-commit

# 启动开发环境
.\start.bat
# 或分开启动:
# 终端1: cargo run
# 终端2: cd frontend && npm run dev
```

---

## 十二、附录: 关键数字

| 指标 | 值 |
|------|----:|
| 源码行数 (Rust, 不含 target) | 76,438 |
| 源码行数 (JS/JSX) | 44,134 |
| 源码行数 (CSS) | 8,344 |
| 测试行数 (Rust tests/) | 7,700 |
| 测试行数 (前端 *.test.*) | 12,251 |
| 文档行数 (Markdown) | 29,347 |
| 代码 + 文档总计 | ~178,000 |
| Rust 子 crate | 5 (qrpc_core, qrpc_core_ir, qrpc_compiler, qrpc_runtime, quantscript) |
| 活跃 RFC | 20 (19 已落地 ✅, 1 部分 🔄) |
| 指标种类 | 18 (全部有 evaluator 实现) |
| 支持交易所 | 2 (binance, okx) |
| 支持交易对 | 3 (BTCUSDT, ETHUSDT, SOLUSDT) |
| 前端测试文件 | 92 |
| 前端测试用例 | 269 |
| 后端测试用例 | ~115 (含集成测试) |
| 存储配额 | 500 MB |
| CI 门禁步数 | 13 |
| API 路由 | 20+ |
| PluginKind 变体 | 5 (Data, Intent, Agent, Risk, Execution) |
