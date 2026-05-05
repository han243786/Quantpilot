# 交易沙箱实现

这是 QuantPilot 当前阶段最高优先级的实现指南。

目标不是添加更多不相关的功能。目标是将当前运行时、成交、风险和数据流整合到一个统一的交易沙箱中。

关于 CI、回放和服务级别测试所使用的可重复性边界，请参阅 [implementation-test-mode.md](D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-mode.md)。

## 目标模式

沙箱应随时间支持三种模式：

1. `RealTimeSandbox`（实时沙箱）
2. `FastBacktestSandbox`（快速回测沙箱）
3. `AccurateBacktestSandbox`（精确回测沙箱）

它们都应共享：

- `NormalizedMarketData`
- `ExecutionPlan`
- `RiskChecker`
- `FillEngine`
- `RuntimeEvent`

## 现有基础

仓库已包含大部分原始要素：

- `qrpc_core` 具有核心协议对象
- `qrpc_compiler` 已验证主链
- `qrpc_runtime` 已具有运行时协调、成交处理和组合刷新
- 后端 API 已支持测试运行和事件流式传输
- 前端已编译图并消费运行时事件

当前前端诊断提升也复用了此基础：

- 运行时事件日志加节点 `runtime_state` 现在馈送给 `frontend/src/components/RuntimeDiagnosticsPanel.jsx`
- 选定节点诊断面现在出现在属性面板和工作区诊断模式中
- 运行详情和回测详情响应现在也暴露从相同运行时事件日志/回测事件日志工件推导的结构化 `runtime_diagnostics` 负载
- 这仍然是一个运行时诊断协议，而非并行通道：前端详情视图现在优先使用后端投影的诊断负载，仅在没有详情负载存在时回退到本地事件投影
- 相同的单一诊断合约现在也驱动研究/事件流面：`EventStreamPanel.jsx` 和 `StrategyResearchConsole.jsx` 现在在详情负载提供默认节点时遵循后端投影的默认节点，否则遵循显式选定的节点，否则保持完整事件流可见，而非发明第二套诊断特定过滤器模型
- 在该合约之上的即时强化通道现在也已落地：运行时会话 SSE 测试依赖 `frontend/src/store/graphStoreRuntimeTransport.js` 而非原始全局 `EventSource`，回测详情诊断覆盖现在锚定在稳定的部分/卡片结构上，而非完整的面向用户文案
- 在相同运行时事件边界之上的下一个路线图切片现在也已落地：风险决策、执行计划、成交引擎生命周期事件现在暴露结构化解释字段，如 `reason_text`、`limit_triggered`、`sizing_source`、`order_type_decision_reason`、`lifecycle_stage` 和 `explanation_summary`，事件流面直接渲染这些相同字段，而非发明第二套执行或风险解释协议
- 在相同合约之上的后续详情切片现在也已落地：`runtime_diagnostics.node_details` 现在携带 `explanation_rows`、`risk_detail_rows` 和 `order_detail_rows`，因此属性面板和工作区诊断面可渲染订单详情和风险详情解释，而无需在 `runtime_diagnostics` 之外引入第二套响应系列
- 在相同合约之上的下一个面跟进也已落地：`EventStreamPanel.jsx` 中选定的运行/回测历史卡片现在复用相同的解释行，因此事件历史、订单历史和风险历史都保持在单一诊断/解释负载系列上
- 持久化详情跟进也在相同合约上落地：`BacktestDetailPage.jsx` 现在在显式的解释部分内渲染相同的解释行，因此诊断、历史、详情视图都保持在单一负载系列上，而非派生第二套解释响应形态
- 在相同合约之上的下一个窄运行时跟进现在也已落地：集中度、按交易对净敞口和组合净敞口守卫现在通过 `RiskPolicy`、`RiskDecisionProduced` 和 `runtime_diagnostics.node_details.risk_detail_rows` 流动，因此风险检查器、运行时详情面和前端诊断都保持在单一负载系列上，而非打开第二套组合风险协议
- 该通道上的当前诚实停止线现已显式：每日亏损限制保持延迟，直到沙箱携带可信的会话/日亏损基线，因此此切片不假装当前组合市值计价是有效的日亏损合约
- 在相同沙箱/回测合约之上的第一个窄参数扫掠跟进现在也已落地：运行时路由系列现在支持对 `fee_bps`、`slippage_bps` 和 `latency_ms` 的持久化执行假设扫掠，但每个变种仍通过正常回测路径执行，并通过现有回测详情负载系列浮出，而非引入第二套仅实验的运行时传输

因此，这是一个整合步骤，而非从零重写步骤。

## 建议边界

```rust
trait Sandbox {
    fn start(&mut self) -> anyhow::Result<()>;
    fn stop(&mut self) -> anyhow::Result<()>;
    fn submit_execution_plan(&mut self, plan: ExecutionPlan) -> anyhow::Result<FillResult>;
    fn on_market_data(&mut self, data: NormalizedMarketData) -> anyhow::Result<Vec<RuntimeEvent>>;
    fn snapshot(&self) -> anyhow::Result<SandboxSnapshot>;
}
```

建议职责：

- `Sandbox`：模式边界
- `RiskChecker`：执行前检查
- `FillEngine`：成交和匹配行为
- `MarketDataFeed`：实时或历史输入源
- `EventLog`：结构化事件
- `SnapshotStore`：恢复和回放支持

## 近期任务

### 任务 1：沙箱抽象

目标：

- 停止将 `RuntimeCoordinator` 视为唯一的运行时模式
- 为不同模式引入一个显式的运行时边界

建议位置：

- `qrpc_runtime/src/lib.rs`
- `qrpc_runtime/src/sandbox.rs`

### 任务 2：成交引擎边界

目标：

- 稳定 `ExecutionPlan + MarketState -> FillResult`
- 保持匹配逻辑与更高级的运行时编排分离

当前基础已包括：

- 市价单
- 限价单
- IOC
- FOK
- 挂单延续
- 幂等处理

后续步骤：

- 更显式地定义 `MarketState`
- 分离滑点模型钩子
- 为 L1 留出空间，L2/L3 留到以后
- 使账户更新边界更清晰

### 任务 3：风险检查器边界

目标：

- 将风险检查拉入更清晰的模块边界
- 使所有执行计划通过一个一致的风险门禁

预期的检查包括：

- 仓位限制
- 杠杆限制
- 订单频率限制
- 无效操作拒绝

### 任务 4：统一数据输入

目标：

- 使实时和历史路径都产生相同的规范化市场数据合约
- 保持上层独立于原始源格式

2026-04-23 当前落地切片：

- 实时数据规范化现在将 `DataQualitySnapshot` 附加到规范化的 K 线和报价快照
- 实时数据模块现在在 `DataUpdated`、`RuntimeWarning` 和 `RuntimeError` 上发出 `source_health`、`freshness_ms`、`stale_after_ms`、`gap_count`、`quality_flags` 和 `explanation_summary`
- 前端研究面现在也通过共享的运行时诊断投影消费相同的事实系列，因此研究摘要卡片不会在诊断和事件流视图之外派生独立的数据质量协议
- 图节点卡片现在也通过 `frontend/src/nodes/nodeCardPresentation.js` 消费相同的事实系列，因此源健康、新鲜度与过期阈值比较、源延迟和间隙计数在画布上保持可见，而无需添加第二套节点卡片传输
- 回放/快速回测路径现在复用相同的辅助链，因此历史收集和实时收集暴露相同的质量事实
- 更上层现在通过运行时事件和 `runtime_diagnostics` 消费相同事实，而非通过第二套数据质量特定传输

### 任务 5：日志、快照和恢复

目标：

- 使用稳定的运行时输出支持回放、审计和恢复

最低预期输出：

- 结构化运行时事件
- 账户快照
- 模式快照
- 恢复入口点

2026-04-24 当前落地回放跟进：

- 持久化的运行和回测记录现在通过后端运行时 API 暴露分页回放投影
- 回放排序通过稳定的 `sequence_no` 值和检查点标签显式化，而非隐式的数组位置假设
- 前端事件流侧边栏现在通过窄回放浏览器消费相同的持久化时间线，而非第二套事件传输
- 这仍然是现有运行时输出之上的恢复/审计投影，而非第二套运行时模式或仅回放合约系列

## 交付顺序

### 阶段 1：实时沙箱

- 复用当前事件流式传输路径
- 复用当前成交逻辑
- 复用当前风险逻辑

### 阶段 2：快速回测沙箱

- 馈送 K 线或 L1 数据
- 使用简化匹配逻辑
- 保持运行足够确定以进行重复执行

此阶段的确定性可重复性应来自显式的测试模式合约，而非偶然行为。如果回放或回归工作流需要固定排序、固定时钟行为或种子控制，这些假设应通过测试模式声明，而非隐藏在沙箱内部实现中。

当前第 1 周实现现在通过以下方式暴露此功能：

- `DeterministicTestMode`
- `DeterministicClockMode`
- `DeterministicEventOrdering`
- `DeterministicParallelismPolicy`

并在 `RealTimeSandbox` 和 `FastBacktestSandbox` 上保持选定的配置。

### 阶段 3：精确沙箱

- 添加 L2/L3 数据
- 添加队列位置
- 添加延迟模型
- 添加更高保真度的匹配

## 与插件化的关系

沙箱工作是插件化的先决条件。

在以下边界稳定之前，不要冻结插件合约：

- `Sandbox`
- `RiskChecker`
- `FillEngine`
- `NormalizedMarketData`
- `RuntimeEvent`

## 确定性测试模式边界

沙箱应支持确定性测试模式作为测试和回放辅助手段，而非独立的产品模式。

近期期望：

- 相同的固定输入包应产生稳定的事件排序
- 回放和回测冒烟运行不应依赖挂钟时序变化
- 能力门禁应与正常 beta 配置保持一致
- 仅测试控制应通过显式的运行时配置进入

当前第 1 周实现也从 `qrpc_runtime` 导出 `RuntimeSupportBoundary`，因此后端能力响应和编译门禁可以消费相同的运行时模式和执行模块边界，而非维护第二份副本。

这对于第 2 周服务级别 API 测试和前端 E2E 冒烟覆盖尤其重要。
