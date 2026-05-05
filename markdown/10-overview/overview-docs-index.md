# QuantPilot 文档索引

本文件是分层 Markdown 结构内简洁的活跃目录。
使用它快速找到当前活跃文档。
对于子树导航，从 [../README.md](../README.md) 开始。

## 从这里开始

1. [当前状态与发布状态](./overview-current-status-and-roadmap.md)
2. [实现规划索引](../implementation/planning/README.md)
3. [实现治理索引](../implementation/governance/README.md)
4. [实现运行时索引](../implementation/runtime/README.md)
5. [指南索引](../guides/README.md)

## 发布和收尾文档

- [首次发布就绪](../implementation/planning/implementation-first-release-readiness.md)
- [v0.2.0 升级工作清单](../implementation/planning/implementation-v0-2-upgrade-worklist.md)
- [已归档功能收尾台账](../archive/planning-retired/implementation-functional-closeout-task-table.md)
- [已归档 P2 收尾清单](../archive/planning-retired/implementation-non-blocking-closeout-list.md)

## 活跃合约文档

- [支持矩阵](../implementation/governance/implementation-support-matrix.md)
- [编译链合约](../implementation/governance/implementation-compile-chain-contract.md)
- [QuantScript 保留界面合约](../implementation/governance/implementation-quantscript-retained-surface-contract.md)
- [运行时治理合约](../implementation/runtime/implementation-runtime-governance-contract.md)
- [运行时证据合约](../implementation/runtime/implementation-runtime-evidence-contract.md)
- [运行时变异合约](../implementation/runtime/implementation-runtime-mutation-contract.md)
- [运行时 AI 批准合约](../implementation/runtime/implementation-runtime-ai-approval-contract.md)
- [运行时/回测解释合约](../implementation/runtime/implementation-runtime-backtest-explanation-contract.md)
- [持久化/回放合约](../implementation/runtime/implementation-persistence-replay-contract.md)
- [测试层期望](../implementation/runtime/implementation-test-layer-expectations.md)
- [交易沙箱实现](../implementation/runtime/implementation-trading-sandbox.md)
- [确定性测试模式](../implementation/runtime/implementation-test-mode.md)

## QuantScript 参考文档

- [QuantScript 支持界面](../../quantscript/QUANTSCRIPT_SUPPORTED_SURFACE.md)
- [QuantScript 技术指南](../../quantscript/QUANTSCRIPT_TECHNICAL_GUIDE.md)
- [QuantScript AI 指南](../../quantscript/QUANTSCRIPT_AI_GUIDE.md)
- [QuantScript 真实策略编写试验](../../quantscript/QUANTSCRIPT_REAL_STRATEGY_AUTHORING_TRIAL.md)
- [QuantScript 主干基线](../guides/quantscript/guide-quantscript-trunk-baseline.md)
- [正式 QuantScript 语法指南](../guides/quantscript/guide-formal-quantscript-syntax.md)
- [V1 冻结/取消范围清单](../guides/quantscript/guide-v1-freeze-descope-checklist.md)

## 支持索引

- [原则索引](../principles/README.md)
- [研究参考索引](../research/README.md)
- [审查参考索引](../reviews/README.md)
- [活跃协议 RFC 索引（`RFC-001` 至 `RFC-020`）](../protocol/README.md)
- [归档索引](../archive/archive-index.md)

## 结构规则

- 每个目录级别的 `README.md` 是该子树的导航入口。
- 概览文档应总结和路由，而非重复实现合约。
- 实现文档定义活跃产品、治理、运行时的和规划的真实数据。
- 研究和审查文档仅作参考，不得覆盖活跃的实现、发布或支持边界文档。
- `archive/` 保存历史材料，不是当前的实现真实数据源。
- 文档文件名保持 ASCII，文件内容应保持 UTF-8。
