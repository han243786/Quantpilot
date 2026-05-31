# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle_import_pass 第四轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001ED-01
> 基线: `389-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EE-01 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001ED-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第四轮父叶残余判断 | 父叶递归分派 |
| 规范矩阵 | recursive residual judgment / staged explicit import pass | 继续细拆 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 选择下一子叶 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` | 选择下一子叶 |

---

## 当前残余队列

```text
runtime.mutation.parameter_mutation.transition_lifecycle_import_pass
transition_lifecycle_import_pass fourth_parent_residual_judgment
runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false
activation_flow_import_pass_selected
single_file_activation_flow_import_pass
remaining_parent_import_bridge_16
remaining_mutation_import_bridge_14
remaining_parameter_mutation_import_bridge_4
remaining_transition_lifecycle_import_bridge_3
old_three_leaf_pause_target_cancelled
```

当前 transition_lifecycle residual 为 3 文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
```

---

## 选择判断

本轮选择:

```text
runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass
```

选择理由:

1. `activation_flow.rs` 是剩余两个业务 flow 之一，仍有 `use super::*`，且刚刚完成的 `activation_snapshot_side_effect.rs` 是它的直接下游 helper。
2. 先处理 activation flow，可以把 activation scheduled / activated / safe-window denied 的输入面从父 wildcard 中显式化。
3. `rollback_flow.rs` 与 rollback ledger、rollback id、target version resolution 更紧密，适合在 activation flow 后单独处理。
4. `transition_lifecycle.rs` 是父 facade，仍承担 `pub(crate) use` 与父级白箱汇聚职责；在两个 flow 仍依赖父 wildcard 时不宜优先清理。
5. 本轮不宣称父叶完成，只选择下一子叶。

---

## 不选择项

暂不选择 `rollback_flow_import_pass`:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
```

原因: rollback flow 涉及 ledger list、rollback target version、rollback record id 与 rollback-specific lifecycle，适合等 activation flow 收敛后独立冻结。

暂不选择父 facade import rewrite:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
```

原因: 父 facade 仍是 activation / rollback flow 的白箱中转层。提前清理会把两个未收敛子 flow 的输入面同时暴露，单步过宽。

---

## 下一步边界

下一步只允许建立 BE-001EE-01 等价基线:

```text
BE-001EE-01
runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass
```

BE-001EE-01 不得直接改 Rust。

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

AI 声称 BE-001ED-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 父叶仍保持 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`。
3. 当前 transition_lifecycle residual 为 3 文件。
4. 下一步只能进入 BE-001EE-01 `activation_flow_import_pass` 单子叶等价基线。
5. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得宣称 activation_flow import 已改写、rollback_flow import 已改写、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `390-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第四轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `stop_split: false`。
3. 下一步固定为 BE-001EE-01 单子叶等价基线。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
