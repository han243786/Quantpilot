# QuantPilot 编译链合约

## 目的

本文档是编译解释的活跃收口合约。在编译 UI、编译摘要措辞、诊断路由或运行时工件描述发生变化时使用。

它不引入新的编译通道。它仅将当前 beta 链锁定为一种显式解释。

## 固定顺序

QuantPilot 编译必须严格按此顺序解读：

1. `strategy_ir` 语义预检
2. 可选的 `quantscript.formal_source` 降级
3. `/api/runtime/compile` 作为可运行结果的权威

## 解释规则

- `strategy_ir` 可能提前失败，并应诚实暴露诊断信息。
- `strategy_ir` 绝不决定最终的可运行输出。
- `quantscript.formal_source` 在存在时可以提供运行时编译输入。
- 如果正式降级不可用，运行时编译可回退到图生成的 `runtime_config`。
- 无论输入路径如何，最终的可运行结果始终遵循 `/api/runtime/compile`。

## 必需的 UI 措辞

以下含义必须在属性面板、工作区摘要卡片、失败通知、测试和文档中保持一致：

- `Strategy IR 角色` 仅表示预检角色
- `运行时来源` 表示哪个工件馈送了运行时编译
- `可运行真实结果` 表示最终结果仍然遵循 `/api/runtime/compile`

当预检通过但运行时编译失败时，UI 必须解释：

- 预检成功不意味着可运行成功
- 操作者必须修复实际进入运行时编译的工件
- 结构化诊断和运行时真实结果字段应一起阅读
- 该警告文案必须来自共享的前端合约源，而非各面板的内联措辞

## 当前接受的运行时来源标签

- `Formal QuantScript lowering 输入`
- `图生成的 runtime_config 输入`
- `图生成的 runtime_config 回退输入`

## 当前接受的可运行真实结果标签

- `以 /api/runtime/compile 输出为准`

## 当前接受的冲突指导文案

- 冲突警告消息和提示由 `frontend/src/utils/compileContract.js` 拥有
- 属性面板、操作指导和测试必须复用该措辞，而非本地重述
- 操作失败文案可添加操作特定的恢复步骤，但可运行真实结果的措辞仍必须复用 `COMPILE_CONTRACT.runtimeSourceOfTruthLabel`

## 共享文案源清单

- 编译冲突真实结果：`frontend/src/utils/compileContract.js`
- 操作失败后续步骤：`frontend/src/utils/actionFailure.js`
- 能力暴露和支持标签：`frontend/src/capabilities/supportMatrix.js`
- 能力过度声明门禁：`frontend/src/capabilities/capabilityGovernance.js` 和 `tools/check-user-facing-text.ps1`

## 漂移检查

任何编译链变更必须同时更新所有受影响的层：

- 前端编译摘要投影
- 工作区编译上下文卡片
- 编译失败指导文本
- 编译相关测试
- 措辞变更时的支持/治理文档

## 参考

- [支持矩阵](./implementation-support-matrix.md)
- [能力治理](./implementation-capability-governance.md)
- [当前状态与发布状态](../../overview/overview-current-status-and-roadmap.md)
- [已归档功能收口台账](../../archive/planning-retired/implementation-functional-closeout-task-table.md)
