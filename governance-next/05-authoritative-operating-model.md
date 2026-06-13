# 新治理权威运行模型

> Batch: `GOV-GOVERNANCE-NEXT-PROMOTION-01`
> Status: active
> Scope: all changes

本文是 `governance-next/` 全面接管后的默认运行模型。它把旧治理从“入口权威”降级为“兼容档案”，并规定代理每次开发、重构和文档治理时如何选用最小但足够的治理强度。

## 1. 默认执行栈

任何变更默认进入以下执行栈:

```text
User intent
  -> QPCursor
  -> governance heat
  -> local invariants
  -> allowed workset
  -> implementation or documentation action
  -> evidence
  -> cursor update
```

旧三矩阵只在两种情况下被读取:

1. 新治理明确引用旧矩阵作为历史事实或兼容门禁素材。
2. 当前任务需要解释旧递归游标、旧模块树或旧里程碑记录。

## 2. 三档治理强度

| 强度 | 触发条件 | 必需产物 |
| --- | --- | --- |
| Light | G0-G1，单文件文档、无行为和无模块树影响 | QPCursor 摘要、影响声明、必要门禁 |
| Standard | G2-G3，普通叶子抽离、模块树更新、接口内重排 | QPCursor、热度判定、局部不变量、全量树或模块树同步、closeout evidence |
| Precision | G4-G5，跨父级、接口冻结、发布风险、超大叶或高不确定性 | 完整 QPCursor、baseline、allowed workset、stop_if、回归门禁、closeout evidence、接管说明 |

治理强度只允许被证据降档。不能因为任务看起来小、用户提示词短、或历史递归已经跑熟而自动降档。

## 3. 旧治理归档边界

旧治理保留，但不再主导:

| 旧内容 | promote 后角色 |
| --- | --- |
| `process-matrix.md` | 流程历史和兼容检查来源 |
| `standard-matrix.md` | GP 投影参考，不覆盖局部不变量 |
| `guidance-matrix.md` | 全量树历史定位参考 |
| `module-tree.md` | 白箱模块事实库，仍需同步 |
| `recursive-speed-protocol.md` | 旧递归优化素材，不能覆盖 QPCursor |
| `recursive-state.json` | 暂存当前递归游标，后续可迁入 QPCursor 状态文件 |

旧治理中的硬规则如果仍然有效，应被迁移或投影到新治理文件，而不是继续要求代理同时执行两套入口。

## 4. 递归重构默认动作

递归重构只执行一个当前 QPCursor 允许的原子动作:

1. 父级 residual judgment。
2. 子叶 baseline。
3. 子叶 extraction。
4. 单叶 closeout。
5. 父级 closeout。

同父级子叶可以并行评估，但只有在 QPCursor 标明 wave 范围、共享门禁、降档条件和逐叶证据后，才允许并行提交。

## 5. 末端叶子判定

每个叶子都必须先回答是否值得继续细分:

1. 单叶 150-600 LOC 且 owner 清晰，默认 stop_split。
2. 小于 100 LOC 的微叶默认 stop_split，除非存在独立风险语义。
3. 100-200 LOC 且治理成本高于拆分收益，默认 same_parent_wave 或 stop_split。
4. 大于 800 LOC、风险高、职责混杂或测试隔离差，进入 split evaluation。
5. G4-G5 高风险叶进入 precision baseline，不被普通 wave 掩盖。

## 6. 门禁策略

门禁按任务热度选择:

1. Light 任务至少运行格式、文档路径和相关脚本门禁。
2. Standard 任务运行相关包测试或 cargo check、文档路径、全量树和治理门禁。
3. Precision 任务运行相关测试、反向引用检查、文档门禁、必要 smoke 或人工核查。

旧门禁仍可执行，但通过旧门禁不代表新治理已经完成；必须有 QPCursor evidence。

## 7. 发布过渡保护

开发态默认禁止子模块横向直连。只有开发者明确声明进入发布版本过渡时，代理才允许提出横向连接、旁路缓存或热路径直连方案。代理不得主动诱导进入发布版本过渡。
