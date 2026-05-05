# Quant 运行时协议核心 RFC 索引

本目录是当前活跃的 QRPC 协议基线。

仅保留在 `markdown/protocol` 下的 RFC 文件对当前开发、评审和实现工作具有规范性。当前活跃协议面已重新编号为连续的 `RFC-001` 至 `RFC-020` 范围。

已退役的 RFC 已移至 `markdown/archive/protocol-retired`，保留其原有标识符，不得用作活跃设计输入。

## 活跃协议集

QuantPilot 当前使用连续的 20 个 RFC 作为活跃协议面：

1. `RFC-001` 数据请求协议
2. `RFC-002` 规范化市场数据协议
3. `RFC-003` 运行时状态协议
4. `RFC-004` 代理协议
5. `RFC-005` 意图协议
6. `RFC-006` 意图生成器协议
7. `RFC-007` 组合协议
8. `RFC-008` 全局风险控制协议
9. `RFC-009` 风险决策协议
10. `RFC-010` 分配协议
11. `RFC-011` 执行计划协议
12. `RFC-012` 订单协议
13. `RFC-013` 执行反馈协议
14. `RFC-014` 运行模式协议
15. `RFC-015` 运行时事件协议
16. `RFC-016` 能力发现协议
17. `RFC-017` 回测工件协议
18. `RFC-018` 回测输入协议
19. `RFC-019` 回测输出工件协议
20. `RFC-020` 插件清单协议

## 核心链路

当前活跃的运行时链路保持不变：

`数据请求 -> 规范化市场数据 -> 意图 -> 代理决策 -> 全局风险决策 -> 执行计划 -> 运行时事件`

## 活跃 RFC 映射

- [RFC-001](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-001-data-request-protocol.md)：`DataRequest`
- [RFC-002](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-002-normalized-market-data-protocol.md)：`NormalizedMarketData`
- [RFC-003](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-003-runtime-state-protocol.md)：`RuntimeState`
- [RFC-004](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-004-agent-protocol.md)：`Agent`
- [RFC-005](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-005-intent-protocol.md)：`Intent`
- [RFC-006](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-006-intent-generator-protocol.md)：`IntentGenerator`
- [RFC-007](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-007-portfolio-protocol.md)：`Portfolio`
- [RFC-008](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-008-risk-protocol.md)：`GlobalRiskController`
- [RFC-009](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-009-risk-decision-protocol.md)：`RiskDecision`
- [RFC-010](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-010-allocation-protocol.md)：`Allocation`
- [RFC-011](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-011-execution-plan-protocol.md)：`ExecutionPlan`
- [RFC-012](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-012-order-protocol.md)：`Order`
- [RFC-013](D:/rust-js-pr\QuantPilot\quantpilot\markdown\02-protocol\RFC-013-execution-feedback-protocol.md)：`ExecutionFeedback`
- [RFC-014](D:/rust-js-pr\QuantPilot\quantpilot\markdown\02-protocol\RFC-014-runtime-mode-protocol.md)：`RuntimeMode`
- [RFC-015](D:/rust-js-pr\QuantPilot\quantpilot\markdown\02-protocol\RFC-015-runtime-event-protocol.md)：`RuntimeEvent`
- [RFC-016](D:/rust-js-pr\QuantPilot\quantpilot\markdown\02-protocol\RFC-016-capability-discovery-protocol.md)：能力发现及当前 beta 支持边界
- [RFC-017](D:/rust-js-pr\QuantPilot\quantpilot\markdown\02-protocol\RFC-017-backtest-artifact-protocol.md)：编译工件包与回测工件标识
- [RFC-018](D:/rust-js-pr\QuantPilot\quantpilot\markdown\02-protocol\RFC-018-backtest-input-protocol.md)：`RunSpec`、`BacktestSpec` 与输入侧回放模式
- [RFC-019](D:/rust-js-pr\QuantPilot\quantpilot\markdown\02-protocol\RFC-019-backtest-output-artifact-protocol.md)：`EventLogArtifact`、投影工件与可重现性清单
- [RFC-020](D:/rust-js-pr\QuantPilot\quantpilot\markdown\02-protocol\RFC-020-plugin-manifest-protocol.md)：最小插件清单、兼容性边界与扩展点白名单

## 归档边界

已退役的 RFC 仅存放于
[protocol-retired](D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/protocol-retired/README.md)。

当前开发应仅将本目录中的 20 个文件视为唯一的活跃协议编号空间。
