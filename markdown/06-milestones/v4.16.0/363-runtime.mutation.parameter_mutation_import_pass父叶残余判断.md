# v4.16.0 runtime.mutation.parameter_mutation_import_pass 父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DS-01
> 基准: `362-runtime.mutation.parameter_mutation.record_query_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation.parameter_mutation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DT-01 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DS-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断 | 父叶重新排队 |
| 规范矩阵 | parent bridge residual accounting、单 pocket 递归、旧暂停目标取消 | 规则确认 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation_import_pass` | 父叶残余判断 |
| 模块树 | `runtime.mutation.parameter_mutation_import_pass` | 保持 `stop_split: false` |

---

## 父叶判定

```text
runtime.mutation.parameter_mutation_import_pass stop_split: false
parameter_mutation_parent_residual_judgment_complete
remaining_parent_import_bridge_21
remaining_mutation_import_bridge_19
remaining_parameter_mutation_import_bridge_9
old_three_leaf_pause_target_cancelled
```

父叶不能 closeout，原因:

1. `src/runtime/mutation/parameter_mutation.rs` parent facade 仍有 `use super::*`。
2. `src/runtime/mutation/parameter_mutation/proposal_creation.rs` 仍有单文件 parent wildcard import。
3. `transition_lifecycle` 仍是 7 文件子树，包含 lifecycle facade 与 6 个子叶 residual。
4. `record_query_import_pass` 已收束，不再作为当前队列入口。
5. 旧的“完成三个叶子节点后暂停”指令保持取消；当前递归只由真实父叶残余驱动。

---

## 当前残余分布

runtime parent bridge:

```text
root 1
run 0
backtest 0
mutation 19
test-only 1
total 21
```

`parameter_mutation` 父叶 residual:

```text
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/proposal_creation.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
```

---

## 下一候选选择

选择:

```text
BE-001DT-01
runtime.mutation.parameter_mutation.proposal_creation_import_pass
single_file_proposal_creation_import_pass
```

不选择整批 9 文件 rewrite:

```text
reject_parameter_mutation_bulk_rewrite_9_files
```

不选择 `transition_lifecycle_import_pass` 作为下一步的原因:

1. lifecycle 是 7 文件子树，内部包含 activation、rollback、boundary、persistence 多个职责。
2. lifecycle facade 与子模块之间存在 sibling-to-parent 转运面，风险高于单文件 proposal creation。
3. 先处理单文件 proposal creation 可以继续缩小 parent bridge，并为后续 lifecycle 子树保留清晰基线。

不选择 `parameter_mutation.rs` parent facade 作为下一步的原因:

1. parent facade 仍负责 re-export `proposal_creation`、`record_query` 与 `transition_lifecycle`。
2. parent facade 应在 child pockets 收束后再清理，避免提前扩大 import 面。

---

## 下一步边界

BE-001DT-01 只能建立单子叶等价基线，不得直接改 Rust。

冻结范围:

```text
src/runtime/mutation/parameter_mutation/proposal_creation.rs
create_runtime_parameter_mutation
runtime_parameter_mutation_record_id
```

必须排除:

```text
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/**
src/runtime/mutation/ai_proposal/**
src/runtime/mod.rs
src/runtime/run_guard.rs
release transition
sibling horizontal link
```

---

## 验证要求

本批提交前至少执行:

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

AI 声称 BE-001DS-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. `runtime.mutation.parameter_mutation_import_pass stop_split: false`。
3. 当前 parent bridge 仍为 total 21 / mutation 19。
4. 当前 `parameter_mutation` residual 为 9 文件。
5. 下一步只能进入 BE-001DT-01 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线。
6. 旧三叶暂停目标保持取消。

不得宣称 parameter mutation import 已完成、mutation import 已完成、parent import bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `363-runtime.mutation.parameter_mutation_import_pass父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `runtime.mutation.parameter_mutation_import_pass stop_split: false`。
3. 下一步固定为 BE-001DT-01 单子叶等价基线。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
