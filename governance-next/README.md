# QuantPilot Next Governance

> 状态: 旁路孵化区。
> 生效范围: 只对明确声明接入 `governance-next` 的任务生效。
> 替换关系: 不替换旧治理；旧 `markdown/` 治理体系继续保持当前权威。

这个目录用于隔离新一代 QuantPilot 治理体系。它先作为旁路运行区存在，等总游标、模块树、热度触发和门禁接管跑通后，再决定是否迁移旧治理入口。

## 1. 为什么单独开目录

旧治理已经承载了当前项目运行所需的 General Policy、超级规范化、三矩阵、全量树、模块树、递归协议和 closeout 门禁。新治理还处在重新分层阶段，不能直接覆盖旧体系。

因此本目录遵守三条隔离规则:

1. 不修改旧治理文件。
2. 不移动旧治理文件。
3. 不把本目录声明为默认权威，除非未来有明确 promote 记录。

## 2. 新治理目标

新治理的目标不是继续堆叠规则，而是把日常开发主控迁移到可定位、可接管、可校验的结构坐标上:

```text
日常开发主控 = 总游标 + 全量树 + 模块树
边界约束主控 = 局部不变量 + General Policy 引用
流程升级主控 = 治理热度 + 超级规范化触发矩阵
异常探测主控 = 自由维度诱错循环
发布证明主控 = closeout gates + evidence
```

## 3. 目录内容

| 文件 | 职责 |
| --- | --- |
| `00-operating-principles.md` | 新治理的分层原则和旧治理隔离边界 |
| `01-qpcursor-protocol.md` | QPCursor 总游标协议，定义代理接管坐标 |
| `02-governance-heat-trigger.md` | G0-G5 治理热度和触发式流程升级 |
| `03-local-invariants.md` | 模块、切面、边的局部不变量绑定规则 |
| `04-adoption-and-promotion.md` | 旁路试运行、证据收集、替换旧治理的条件 |
| `trials/` | 明确声明 `governance_next_trial: true` 的旁路试运行样本 |

## 4. 当前使用方式

在新治理跑通前，任何任务如果要试用本目录，必须先声明:

```text
governance_next_trial: true
legacy_governance_authority: preserved
```

如果新治理与旧治理冲突，默认以旧治理为准；新治理只记录差异、问题和改进建议。

## 5. 试运行样本

| Trial | Scope | Status |
| --- | --- | --- |
| `trials/0001-risk-execution-gate-qpcursor.md` | `root.contracts.runtime_support.v4_runtime_support.risk_execution_gate` | handoff_ready |

## 6. promote 前置条件

本目录只有满足以下条件后，才允许讨论替换旧治理入口:

1. 至少完成 3 类任务试运行: 叶子修改、模块重构、能力变更。
2. QPCursor 能让新代理无聊天历史接管任务。
3. 全量树和模块树没有因为旁路治理发生漂移。
4. 热度触发能正确区分日常任务和高风险任务。
5. closeout evidence 能证明新治理没有提高返工率。
