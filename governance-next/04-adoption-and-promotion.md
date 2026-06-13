# 接入与 promotion 记录

本文定义新治理如何完成接入、如何正式接管，以及旧治理如何退到兼容档案。

## 1. 接入阶段

### Phase 0: 隔离建档

目标: 建立 `governance-next/`，不破坏旧治理。

完成证据:

1. 新目录存在。
2. 旧治理文件未被删除。
3. 新治理的 QPCursor、治理热度、局部不变量和 promotion 入口形成。

### Phase 1: 只读试判

目标: 用新治理判断任务热度和游标结构，但不改变执行流程。

完成证据:

1. 历史任务可映射到 G0-G5。
2. 历史任务可生成 QPCursor 草案。
3. 冲突点被记录为治理优化素材。

### Phase 2: 旁路执行

目标: 对明确声明试用的新任务启用 QPCursor。

完成证据:

1. 代理能按游标接管。
2. 修改范围没有越过 allowed workset。
3. stop_if 能阻止范围外扩张。
4. evidence 能支撑下一代理继续。

### Phase 3: 双轨对照

目标: 新治理和旧治理同时运行，比较返工率、接管成本、门禁遗漏和文档漂移。

完成证据:

1. 新治理不增加架构错误。
2. 新治理不提高返工率。
3. 新治理能减少上下文依赖。
4. 新治理能更早发现模块边界问题。
5. `WAVE` 已与 `split_decision`、`governance_packaging` 分离。
6. 超大 G4/G5 高风险叶可升为 precision baseline。
7. QPCursor 生成器减少手写字段。
8. 未跟踪活跃文件可被全量树预检发现。

### Phase 4: promote 提案

目标: 形成替换旧治理入口的正式提案。

输出证据:

1. 试运行任务清单。
2. QPCursor 接管证据。
3. 治理热度判定证据。
4. 新旧治理冲突清单。
5. 迁移方案。
6. 回滚方案。

### Phase 5: 默认接管

目标: `governance-next/` 成为默认权威入口。

完成记录:

```text
batch: GOV-GOVERNANCE-NEXT-PROMOTION-01
authority: governance-next
legacy_governance_mode: archived_reference
default_trial_flag_required: false
```

## 2. promote 后禁止条件

出现以下任一情况，必须阻断当前变更并进入治理修复，而不是回退到旧治理默认权威:

1. 新治理导致旧门禁被绕过。
2. 新治理不能解释 G3-G5 高风险任务。
3. QPCursor 无法让新代理接管。
4. 全量树或模块树出现漂移。
5. 旧治理冲突没有明确处理方案。
6. 仍需要手写重复索引才能完成普通递归步骤，且没有生成或降重路线。

## 3. 回滚原则

promote 后不再把“旧治理默认权威”作为普通回滚路径。可回滚的是某条新治理规则、某个 QPCursor 字段、某个门禁自动化；不可回滚的是新治理作为默认入口的方向。

```text
新治理 = 当前权威
旧治理 = 兼容档案
回滚 = 修复具体规则或门禁，不恢复旧默认入口
```

## 4. 旧治理保留范围

旧治理保留以下能力:

1. 兼容门禁继续检查三矩阵核心文件是否存在。
2. 历史里程碑、旧递归协议、旧模块树可作为事实来源。
3. 递归游标仍暂存在 `markdown/00-matrix-governance/recursive-state.json`。
4. 旧文件可被引用，但不得覆盖 QPCursor 结论。
