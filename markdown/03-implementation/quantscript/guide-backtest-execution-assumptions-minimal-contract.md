# 回测执行假设最小合约

## 目的

本文档定义了研究级回测执行假设的第一个诚实合约。

目标不是引入一个广泛的执行 DSL。目标是让当前回测结果显式声明三个最直接影响模拟成交的假设：

- `fee_bps`
- `slippage_bps`
- `latency_ms`

其他所有内容在第一步中应保持范围外。

## 为何优先处理此事项

隐藏的执行假设比缺失的丰富摘要指标更快地使回测输出产生误导。

因此，第一个 `P4` 切片应：

1. 使执行假设显式化
2. 使其覆盖顺序显式化
3. 使其编译/运行时路径在各入口点间保持一致

只有在此之后，路线图才应扩展到更丰富的指标、交易台账、工件管理或比较工作流。

## 最小字段集

第一个合约仅允许以下字段：

- `fee_bps`
  - 浮点数
  - 必须 `>= 0`
  - 表示每次成交的费用假设，单位为基点
- `slippage_bps`
  - 浮点数
  - 必须 `>= 0`
  - 表示模拟价格滑点，单位为基点
- `latency_ms`
  - 整数
  - 必须 `>= 0`
  - 表示回测模拟中假设的决策到成交延迟

不要添加：

- 价差模型 DSL
- 队列位置模型
- 市场冲击模型
- 部分成交策略 DSL
- 场地路由策略
- 主干 QuantScript 中的逐订单覆盖

## 所有权划分

第一个合约应在两层之间划分。

### 第 1 层：共享 profile 默认值

`execution.profile(...)` 应拥有属于策略形态本身的可复用默认值。

对于第一个 `P4` 切片，这意味着：

- `slippage_bps`
- `fee_bps`

`latency_ms` 被有意排除在第一个 `execution.profile(...)` 扩展之外，因为它更自然地与特定模拟运行相关，而非与策略标识相关。

### 第 2 层：回测请求覆盖

回测请求应拥有运行特定的执行假设。

对于第一个 `P4` 切片，这意味着：

- 可选的 `fee_bps`
- 可选的 `slippage_bps`
- 可选的 `latency_ms`

这些字段应仅针对该特定回测请求覆盖任何 profile 默认值。

## 覆盖规则

第一个回测执行假设合约应按以下顺序解析值：

1. 回测请求显式覆盖
2. execution profile 默认值
3. 后端回退默认值

此顺序必须有文档记录和测试覆盖。

## 后端回退默认值

在存在更丰富的执行假设层之前，第一个默认值应保持简单和显式：

- `fee_bps = 10.0`
- `slippage_bps = 5.0`
- `latency_ms = 0`

如果这些默认值以后发生变化，文档、编译路径、运行时路径和测试必须一起更新。

## 跨入口对齐规则

第一个切片只有在相同语义形态可以在以下各处表达时才有价值：

- 图/运行时编译
- Strategy IR
- 正式 QuantScript

这并不意味着每个入口点必须携带相同的语法。它们必须降级到相同的回测执行假设形态。

## 建议的入口点形态

### 图/运行时

图/运行时应继续使用现有的执行节点加上显式的回测请求覆盖。

第一个落地的图/运行时工作不应发明第二种执行节点类型。

### Strategy IR

Strategy IR 应扩展现有的窄 `execution_profile` 路径，而非引入独立的回测专属执行 DSL。

第一个增加应是：

- 在相同的窄 execution profile 形态上增加 `fee_bps`

而 `latency_ms` 保持请求范围。

### 正式 QuantScript

正式 QuantScript 应继续使用窄的顶层语法：

```qs
execution.profile("paper", slippage_bps=5.0, fee_bps=0.0)
```

在第一个切片中，不应在主干语言中引入 `latency_ms`。`latency_ms` 属于回测请求层。

## 最小可执行任务顺序

第一个 `P4` 切片应按此顺序实现：

1. 扩展 `execution.profile(...)` 合约文档，增加 `fee_bps` 并保留 `slippage_bps`。
2. 增加回测请求字段 `fee_bps`、`slippage_bps` 和 `latency_ms`。
3. 在后端定义一个共享的已解析执行假设形态。
4. 使图/运行时编译使用该形态。
5. 使 Strategy IR 使用相同形态。
6. 使正式 QuantScript 使用相同形态。
7. 增加请求覆盖与 profile 默认值的显式优先级测试。
8. 增加回测工件/报告可见性，以便用户查看实际应用的假设。

## 最小测试矩阵

第一个实现不得在缺少以下内容时发布：

1. 编译测试，证明所有三个入口点降级到相同的执行假设形态
2. 覆盖顺序测试，锁定请求覆盖 > profile 默认值 > 后端回退
3. 验证测试，拒绝负值 `fee_bps`、负值 `slippage_bps` 和负值 `latency_ms`
4. 回测工件测试，展示在稳定输出位置中应用的假设

## 当前状态

此合约现已部分落地。

已落地：

- 在正式 QuantScript、图运行时编译和 Strategy IR 中跨入口的窄 `execution.profile("paper", fee_bps=..., slippage_bps=...)`
- `fee_bps`、`slippage_bps` 和 `latency_ms` 的请求范围回测覆盖
- 后端解析顺序锁定为 `请求覆盖 > profile 默认值 > 后端回退`
- 回测工件清单现在记录已解析的执行假设
- `latency_ms` 现在作为回测执行时钟滞后
- 该滞后现在在回测输出和投影工件（包括事件日志、交易台账和权益曲线）中移动执行和组合时间戳
- 相同的已解析假设现在也投影到 `metrics.execution_assumptions` 中，因此用户无需打开清单即可查看应用的 `fee_bps`、`slippage_bps` 和 `latency_ms`
- 回测运行/详情响应现在也在顶层以 `execution_assumptions` 暴露相同的假设模块，因此客户端无需深入工件内部获取最小假设视图
- 回测列表响应现在也以压缩的 `filters.execution_assumptions_tag` 视图暴露，包含值标签和来源标签，用于轻量级扫描
- `metrics.execution_assumptions`、运行/详情顶层 `execution_assumptions` 和列表级别 `filters.execution_assumptions_tag` 现在围绕一个共享的假设模块形态组织，而非三个不相关的摘要
- 该摘要现在也携带每个字段的来源标签（`request_override`、`profile_default`、`backend_fallback`），并且这些来源标签必须与嵌入的回测清单保持一致
- 工件/单元测试和 API golden-like 测试现在锁定此最小假设切片的 `metrics.execution_assumptions` 字段集、值来源和清单一致性

尚未落地：

- 超越当前时间戳投影的更丰富的工件/报告可见性

部分落地：

- 一个最小的比较工作流现在存在于恰好两个回测 ID
- 该比较输出现在暴露四个稳定的顶层块：
  - `execution_assumptions`
  - `metrics`
  - `trade_ledger`
  - `report_narrative`
- 该块报告三种状态之一：
  - `same`
  - `different`
  - `missing`
- `execution_assumptions` 块暴露以下字段级别的 diff 状态：
  - `fee_bps`
  - `slippage_bps`
  - `latency_ms`
  - `sources`
- `metrics` 块暴露以下字段级别的 diff 状态：
  - `step_count`
  - `trade_count`
  - `total_return_ratio`
  - `max_drawdown_ratio`
  - `final_equity`
  - `net_profit`
  - `turnover_ratio`
  - `average_trade_notional`
  - `fee_drag_ratio`
- `metrics` 块现在也暴露分组的 `drilldown` 层：
  - `performance`
  - `activity`
  - `costs`
- 每个钻取组仍仅报告字段级别的 `same` / `different` / `missing`，但每个字段现在也携带左右两侧的已解析值，以便比较客户端可以解释差异而无需重新推导
- 此切片仍未添加时间线或逐交易指标比较
- `trade_ledger` 块暴露以下字段级别的 diff 状态：
  - `trade_count`
  - `buy_fill_count`
  - `sell_fill_count`
  - `total_fees_paid`
  - `buy_fees_paid`
  - `sell_fees_paid`
  - `total_filled_notional`
  - `buy_filled_notional`
  - `sell_filled_notional`
  - `average_fill_price`
  - `average_buy_fill_price`
  - `average_sell_fill_price`
  - `average_fee_per_fill`
  - `average_buy_fee`
  - `average_sell_fee`
- `report_narrative` 块现在是一个稳定的报告模块，包含：
  - 标题
  - 简短要点
  - 顶层亮点
  - 友好来源说明
  - 明确的 `执行假设`、`指标摘要` 和 `交易台账摘要` 部分
- 比较响应现在也暴露顶层 `compare_report` 视图，将相同的比较真实数据组织为：
  - 一个共享标题
  - 概览层（`要点` 加 `亮点`）
  - `execution_assumptions`、`metrics`、`trade_ledger` 和 `equity_curve` 的模块视图
- 在保留的 `V1` 面下，`report_narrative` 和 `compare_report` 仍然共存作为外部的比较/报告合约；`V1` 后向 `compare_report` 作为唯一外部报告真实数据的迁移路径在 [Compare Report V1 后迁移检查清单](./guide-compare-report-v1-post-migration-checklist.md) 中跟踪
- 每个报告部分现在也携带一行 `summary`，因此比较客户端可以在深入部分行之前渲染紧凑的叙述层
- 比较/报告现在也携带一个窄时间序列 `equity_curve` 模块：
  - 对 `point_count`、`started_at_ms`、`ended_at_ms`、`first_equity`、`final_equity`、`min_equity` 和 `max_equity` 进行摘要字段比较
  - 对 `start`、`middle` 和 `end` 进行样本钻取
  - `ts_ms`、`equity`、`cash_balance` 和 `net_notional` 的左右样本值
- 报告叙述现在也包含一个明确的 `权益曲线` 部分，以便此时间序列钻取在报告层中与假设、指标和交易台账一样可见
- 这仍然是一个窄时间序列比较合约，不是完整的时间线比较 UI 或自由格式系列分析 DSL
- `交易台账摘要` 部分现在使用一个共享的台账摘要模块，而非独立的仅比较形态，因此比较/报告输出在成交数量、费用拆分、名义价值和平均成交价格方面与工件真实数据保持一致
- 该 `report_narrative` 块现在也携带一个小型来源说明部分，将 `request_override`、`profile_default` 和 `backend_fallback` 转化为面向用户的费用、滑点和延迟标签
- 比较块携带左右假设模块，以便客户端可以比较已解析的值和来源标签而无需重新推导
- 比较仍不涉及更丰富的指标、台账钻取或超出此第一叙述层的更广泛报告部分
- 当前更丰富的指标切片有意保持狭窄且关注成本：
  - `net_profit`
  - `turnover_ratio`
  - `average_trade_notional`
  - `fee_drag_ratio`
- 这些更丰富的指标从投影的权益曲线加上已落地的交易台账摘要推导而来，而非来自单独的仅指标计算路径

## 此切片的非目标

不要将此切片扩展为：

- 执行微观结构建模
- 概率滑点
- 对滑点敏感的成交模型 DSL
- 订单簿模拟
- 交易所特定费用表
- 策略本地延迟脚本

这些属于后续审查，而非第一个 `P4` 可执行合约。
