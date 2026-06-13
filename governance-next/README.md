# QuantPilot Next Governance

> 状态: 默认权威治理入口。
> 生效范围: 全部变更。
> 替换关系: `governance-next/` 接管日常治理主控；旧 `markdown/00-matrix-governance/` 降级为兼容档案和门禁素材库。

本目录是 QuantPilot 当前治理体系的默认入口。它已经从试运行区升级为权威入口，负责用 QPCursor、治理热度、局部不变量和证据门禁统一驱动开发、重构、文档治理和接力。

旧三矩阵、递归高速协议和历史 closeout 文档仍然保留，但它们不再拥有默认解释权。若新治理与旧治理发生冲突，以本目录的权威运行模型为准；旧文件只提供历史依据、兼容门禁和迁移素材。

## 1. 权威入口顺序

每次变更先按以下顺序读取:

1. `governance-next/README.md` 确认权威入口。
2. `governance-next/05-authoritative-operating-model.md` 判定执行模型、旧治理关系和证据边界。
3. `governance-next/01-qpcursor-protocol.md` 定位当前 QPCursor。
4. `governance-next/02-governance-heat-trigger.md` 判定 G0-G5 治理热度。
5. `governance-next/03-local-invariants.md` 绑定模块、切面、接口和边的局部不变量。
6. `markdown/10-overview/overview-full-feature-tree.md` 与 `markdown/00-matrix-governance/module-tree.md` 校验全量树和模块树事实。
7. `markdown/00-matrix-governance/recursive-state.json` 只作为当前递归游标存储，直到 QPCursor 状态文件完全接管。

## 2. 新治理目标

新治理的目标不是继续堆叠规则，而是把日常开发主控迁移到可定位、可接管、可校验的结构坐标上:

```text
日常开发主控 = QPCursor + 全量树 + 模块树
边界约束主控 = 局部不变量 + General Policy 引用
流程升级主控 = 治理热度 + 超级规范化触发矩阵
异常探测主控 = 自由维度诱错循环
发布证明主控 = closeout gates + evidence
```

## 3. 目录内容

| 文件 | 职责 |
| --- | --- |
| `00-operating-principles.md` | 新治理的分层原则、旧治理归档关系和不可退化规则 |
| `01-qpcursor-protocol.md` | QPCursor 总游标协议，定义代理接管坐标 |
| `02-governance-heat-trigger.md` | G0-G5 治理热度和触发式流程升级 |
| `03-local-invariants.md` | 模块、切面、边的局部不变量绑定规则 |
| `04-adoption-and-promotion.md` | 接入、试运行证据、promotion 记录和回滚条件 |
| `05-authoritative-operating-model.md` | 新治理全面接管后的默认执行模型 |
| `trials/` | 已完成或仍可复用的 QPCursor 样本库 |

## 4. 当前使用方式

默认任务不再声明 `governance_next_trial: true`。除非文档明确标记为历史样本，否则所有新任务都按以下权威字段执行:

```text
governance_next_authority: active
legacy_governance_mode: archived_reference
qpcursor_required: true
```

旧治理仍可被读取，但只能用于三类用途:

1. 兼容门禁仍依赖的文件清单和历史规则。
2. 解释当前递归游标、模块树、全量树的历史来源。
3. 新治理尚未完全机器化前的过渡性证据库。

## 5. promote 记录

| Batch | 结论 | 证据 |
| --- | --- | --- |
| `GOV-GOVERNANCE-NEXT-PROMOTION-01` | `governance-next/` 成为默认权威入口 | 两个 QPCursor 样本完成 handoff；终端叶子控制 v2、QPCursor 生成、未跟踪文件预检和索引降重均已接入；旧治理成本过高的问题已有新执行模型承接 |

## 6. 不可退化规则

1. 不允许再把旧三矩阵声明为默认权威入口。
2. 不允许用旧递归协议覆盖 QPCursor 的 allowed workset、stop_if 和 evidence。
3. 不允许为了速度删除全量树、模块树、局部不变量或 closeout evidence。
4. 不允许 AI 主动提出发布版本过渡；只有开发者明确声明后，才能提出跨子模块直连优化。
5. 不允许把一次性用户问题写入递归循环；临时问题只回答一次，然后从游标中抛掉。
