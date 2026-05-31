# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle_import_pass 第三轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EB-01
> 基线: `384-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EC-01 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EB-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断 | 父叶递归分派 |
| 规范矩阵 | staged explicit import pass / residual queue / stop_split false | 继续细分 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 下一个 lifecycle residual 选择 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` | 选择下一子叶 |

---

## 父叶判断

```text
runtime.mutation.parameter_mutation.transition_lifecycle_import_pass
runtime.mutation.parameter_mutation.transition_lifecycle_import_pass third_parent_residual_judgment
runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false
activation_snapshot_side_effect_import_pass_selected
single_file_activation_snapshot_side_effect_import_pass
remaining_parent_import_bridge_17
remaining_mutation_import_bridge_15
remaining_parameter_mutation_import_bridge_5
remaining_transition_lifecycle_import_bridge_4
old_three_leaf_pause_target_cancelled
```

父叶不能收口。
理由:

1. `transition_lifecycle_import_pass` residual 仍有 4 个文件。
2. parent facade、activation flow、rollback flow 都仍依赖 parent wildcard import。
3. `activation_snapshot_side_effect.rs` 是单 helper 文件，当前仍为 `use super::*`，适合作为下一轮最小 import pocket。
4. activation / rollback public handler 体量更大，应等 side-effect pocket 收敛后再判断。
5. 不启动发布过渡，不引入 sibling horizontal link。

---

## 当前 residual 队列

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
```

当前已完成并停止细拆:

```text
runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass stop_split: true
runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass stop_split: true
runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass stop_split: true
```

---

## 下一子叶选择

下一步只允许建立 BE-001EC-01 等价基线:

```text
runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass
```

选择原因:

1. `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs` 只承载 `auto_snapshot_on_activation`。
2. 该 helper 由 activation flow 通过父级调用，适合验证父子白箱 import 边界。
3. 它含有持久化 side effect、config generation、snapshot memory write，需要先冻结等价基线，不能直接改 Rust。
4. 处理它后再回到父叶判断，决定 activation flow / rollback flow / parent facade 的顺序。

---

## 禁止事项

BE-001EC-01 不得直接改 Rust。
不得触碰:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
src/runtime/mutation/parameter_mutation.rs
release transition
sibling horizontal link
```

ASCII guard:

```text
no_code_movement
no_function_body_change
no_visibility_change
no_parent_facade_rewrite
no_activation_flow_rewrite
no_rollback_flow_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

---

## 下一步边界

下一步只能进入:

```text
BE-001EC-01
runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass
单子叶等价基线
```

BE-001EC-01 只冻结 `activation_snapshot_side_effect.rs` 输入面，不得直接改 Rust。

---

## 验证要求

本批提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_mutation
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001EB-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 父叶保持 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`。
3. 当前 transition_lifecycle residual 为 4 文件。
4. 下一步只能进入 BE-001EC-01 `activation_snapshot_side_effect_import_pass` 单子叶等价基线。
5. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得宣称 transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `385-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第三轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `stop_split: false`。
3. 下一步固定为 BE-001EC-01 单子叶等价基线。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
