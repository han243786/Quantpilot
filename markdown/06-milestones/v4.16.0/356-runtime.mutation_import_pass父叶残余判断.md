# v4.16.0 runtime.mutation_import_pass 父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DP-01
> 基准: `355-runtime.mutation.shared_governance_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation_import_pass`
> 判定: `runtime.mutation_import_pass stop_split: false`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DQ-01 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DP-01 `runtime.mutation_import_pass` 父叶残余判断 | 父叶重判 |
| 规范矩阵 | parent import bridge、explicit import pass、minimum batch、release transition guard | 下一 pocket 选择 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation_import_pass` | mutation import 父叶白箱 |
| 模块树 | `runtime.mutation_import_pass` | `stop_split: false` |

---

## 当前事实

BE-001DO 已完成 `runtime.mutation.shared_governance_import_pass`，并设置:

```text
runtime.mutation.shared_governance_import_pass stop_split: true
old_three_leaf_pause_target_cancelled
```

当前 parent bridge 剩余:

```text
root 1
run 0
backtest 0
mutation 20
test-only 1
total 22
```

`runtime.mutation_import_pass` 仍未完成，必须保持:

```text
runtime.mutation_import_pass stop_split: false
```

---

## 剩余 mutation 队列

剩余 20 个 mutation residual 文件为:

```text
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_persistence.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/event_lifecycle.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
src/runtime/mutation/ai_proposal/record_query.rs
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
src/runtime/mutation/ai_proposal/source_governance_identity.rs
src/runtime/mutation/ai_proposal/static_check.rs
src/runtime/mutation/ai_proposal/status_transition.rs
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/proposal_creation.rs
src/runtime/mutation/parameter_mutation/record_query.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
```

两组 pocket:

```text
runtime.mutation.parameter_mutation_import_pass
runtime.mutation.ai_proposal_import_pass
```

---

## 下一候选判定

下一步固定为:

```text
BE-001DQ-01 runtime.mutation.parameter_mutation_import_pass 单子叶等价基线
```

理由:

1. 351 staged order 已约束 shared_governance -> parameter_mutation -> ai_proposal。
2. parameter mutation 与刚完成的 shared governance 直接共享 parameter version、event append 和 governance projection 调用面。
3. parameter mutation residual 为 10 文件，风险集中于 create/list/detail 与 activation/rollback lifecycle，可先建立父 pocket 基线，再决定是否继续拆。
4. AI proposal residual 也为 10 文件，但包含 approval、sandbox、static check 与 source governance 多类风险，保持延后更符合 minimum batch。
5. 不得在父叶判断后直接整批 rewrite 20 文件。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不处理 `src/runtime/mutation/parameter_mutation.rs` 或 `src/runtime/mutation/parameter_mutation/**` import。
- 本批不处理 `src/runtime/mutation/ai_proposal.rs` 或 `src/runtime/mutation/ai_proposal/**` import。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 test-only `src/runtime/run_guard.rs`。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。
- 本批不恢复旧的三叶暂停目标。

---

## 验证要求

本批为 `no code movement` 父叶残余判断，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001DP-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `runtime.mutation_import_pass stop_split: false`。
3. 当前 parent bridge 剩余为 root 1 / run 0 / backtest 0 / mutation 20 / test-only 1 / total 22。
4. 下一步只能进入 BE-001DQ-01 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线。
5. 不得直接整批改写 20 个 mutation residual 文件。
6. `ai_proposal`、`src/runtime/mod.rs` 和 test-only `src/runtime/run_guard.rs` 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。
8. 旧的三叶暂停目标仍为取消状态。

不得宣称 mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `356-runtime.mutation_import_pass父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶设置 `runtime.mutation_import_pass stop_split: false`。
3. 下一步固定为 BE-001DQ-01 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
