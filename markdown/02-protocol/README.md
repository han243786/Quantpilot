# Quant 运行时协议核心 RFC 索引

本目录包含 20 个活跃 RFC，覆盖 QuantPilot 运行时链的全部数据结构与接口契约。

## 状态标记

| 标记 | 含义 |
|------|------|
| ✅ 已落地 | 代码中有对应 struct/enum，运行时链路可用 |
| 🔄 部分 | 核心类型已定义，部分字段或变体未完成 |
| 📋 设计 | 协议已定稿，代码尚未实现 |

## 活跃协议集

### 数据层

| RFC | 类型 | 状态 | 说明 |
|-----|------|:--:|------|
| [RFC-001](./RFC-001-data-request-protocol.md) | `DataRequest` | 📋 | 数据请求协议 — 声明所需数据源、时间框架、标的 |
| [RFC-002](./RFC-002-normalized-market-data-protocol.md) | `NormalizedMarketData` / `KlineSeriesSnapshot` | ✅ | 规范化市场数据 — K 线快照、报价快照的统一结构 |

### 信号层

| RFC | 类型 | 状态 | 说明 |
|-----|------|:--:|------|
| [RFC-005](./RFC-005-intent-protocol.md) | `Intent` / `IntentSignal` | ✅ | 意图协议 — 指标信号的标准表达 |
| [RFC-006](./RFC-006-intent-generator-protocol.md) | `IntentGenerator` | ✅ | 意图生成器协议 — 从市场数据计算意图信号 |

### 决策层

| RFC | 类型 | 状态 | 说明 |
|-----|------|:--:|------|
| [RFC-004](./RFC-004-agent-protocol.md) | `Agent` / `AgentDecision` | ✅ | 代理协议 — 意图信号→投资组合决策 |
| [RFC-007](./RFC-007-portfolio-protocol.md) | `Portfolio` / `PortfolioTarget` / `PortfolioState` | ✅ | 组合协议 — 目标持仓与当前持仓快照 |
| [RFC-008](./RFC-008-risk-protocol.md) | `GlobalRiskController` | ✅ | 全局风险控制协议 |
| [RFC-009](./RFC-009-risk-decision-protocol.md) | `RiskDecision` | ✅ | 风险决策协议 — 组合目标→风险校验后的决策 |
| [RFC-010](./RFC-010-allocation-protocol.md) | `Allocation` | 📋 | 分配协议 — 组合权重→具体标的分配 |

### 执行层

| RFC | 类型 | 状态 | 说明 |
|-----|------|:--:|------|
| [RFC-011](./RFC-011-execution-plan-protocol.md) | `ExecutionPlan` | ✅ | 执行计划协议 |
| [RFC-012](./RFC-012-order-protocol.md) | `Order` | 📋 | 订单协议 — 下单请求的标准结构 |
| [RFC-013](./RFC-013-execution-feedback-protocol.md) | `ExecutionFeedback` | 📋 | 执行反馈协议 — 成交回报、部分成交、拒绝 |

### 运行时层

| RFC | 类型 | 状态 | 说明 |
|-----|------|:--:|------|
| [RFC-003](./RFC-003-runtime-state-protocol.md) | `RuntimeState` | ✅ | 运行时状态协议 |
| [RFC-014](./RFC-014-runtime-mode-protocol.md) | `RuntimeMode` | ✅ | 运行模式协议 — paper / testnet |
| [RFC-015](./RFC-015-runtime-event-protocol.md) | `RuntimeEvent` | ✅ | 运行时事件协议 — 事件信封、事件类型、事件投影 |

### 能力与工件层

| RFC | 类型 | 状态 | 说明 |
|-----|------|:--:|------|
| [RFC-016](./RFC-016-capability-discovery-protocol.md) | `CapabilityResponse` | ✅ | 能力发现协议 — `/api/capabilities` 的响应结构 |
| [RFC-017](./RFC-017-backtest-artifact-protocol.md) | `BacktestArtifact` | ✅ | 回测工件协议 — 编译工件包与回测工件标识 |
| [RFC-018](./RFC-018-backtest-input-protocol.md) | `RunSpec` / `BacktestSpec` | ✅ | 回测输入协议 — 运行规格、回测规格与回放模式 |
| [RFC-019](./RFC-019-backtest-output-artifact-protocol.md) | `EventLogArtifact` | ✅ | 回测输出工件协议 — 事件日志工件与投影 |
| [RFC-020](./RFC-020-plugin-manifest-protocol.md) | `PluginManifest` | 🔄 | 插件清单协议 — 最小清单、兼容性边界、扩展点白名单 |

## 核心链路

```
DataRequest → NormalizedMarketData → Intent → Agent → Portfolio
                                                    ↓
                                              RiskDecision → ExecutionPlan → Order → ExecutionFeedback
                                                    ↓
                                              RuntimeEvent
```

## 统计

| 状态 | 数量 |
|------|:--:|
| ✅ 已落地 | 15 |
| 🔄 部分 | 1 |
| 📋 设计 | 4 |

未落地的 4 个 RFC (001/010/012/013) 和部分落地的 RFC-020 是 v1.0.0 插件化架构的协议基础。
