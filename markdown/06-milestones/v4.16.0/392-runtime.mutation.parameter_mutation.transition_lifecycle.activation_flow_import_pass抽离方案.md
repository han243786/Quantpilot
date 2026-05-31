# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EE-02
> 基线: `391-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EE-03 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EE-02 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 抽离方案 | 实施边界冻结 |
| 规范矩阵 | staged explicit import pass / single-file rewrite / parent-child communication | 改写约束冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` | activation flow import rewrite 指令 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` | 固定下一步实际抽离范围 |

---

## 方案冻结

```text
runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass
activation_flow_import_pass plan_frozen
single_file_activation_flow_import_rewrite
be_001ee_03_only_rewrite_activation_flow_imports
remaining_parent_import_bridge_16
remaining_mutation_import_bridge_14
remaining_parameter_mutation_import_bridge_4
remaining_transition_lifecycle_import_bridge_3
old_three_leaf_pause_target_cancelled
ActivationScheduled
Activated
ActivationFailed
```

BE-001EE-03 只允许改写一个文件的顶部 import:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
```

当前 import:

```rust
use super::*;
```

预期 import:

```rust
use super::{
    auto_snapshot_on_activation, evaluate_runtime_parameter_mutation_safe_window,
    mutation_lifecycle_entry, persist_runtime_parameter_mutation_transition,
    resolve_runtime_parameter_mutation_boundary,
};
use crate::{
    auth, append_parameter_mutation_events_to_run, build_runtime_parameter_mutation_event,
    current_time_ms, governance_with_parameter_version, json_bad_request,
    json_bad_request_with_details, load_run_record_from_state,
    load_runtime_parameter_mutation_record, normalize_actor_identity,
    validate_runtime_capability_guard, ActivateRuntimeParameterMutationRequest, AppState,
    RuntimeParameterMutationActivationState, RuntimeParameterMutationRecord,
    RuntimeParameterMutationStatus,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
```

如果 `cargo check` 暴露私有路径或 parent white-box import 差异，BE-001EE-03 只能最小修正 import 输入面并记录差异，不得改函数体。

---

## 禁止改写范围

BE-001EE-03 不得触碰:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/ai_proposal/**
src/runtime/mod.rs
src/runtime/run_guard.rs
release transition
sibling horizontal link
```

ASCII guard:

```text
no_function_body_change
no_visibility_change
no_parent_facade_rewrite
no_rollback_flow_rewrite
no_snapshot_side_effect_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

---

## 等价清单

BE-001EE-03 必须保持:

1. `activate_runtime_parameter_mutation` 名称、签名、`pub(crate)` 可见性不变。
2. capability guard、bad request details、mutation record load、run load、boundary resolution、actor normalization、safe-window evaluation、lifecycle entry、event append、persistence、snapshot side effect 的调用顺序不变。
3. 只允许 `Proposed` 或 `SafeWindowDenied` 状态进入 activation。
4. safe-window denied path 的 status、updated timestamp、denied event、denied lifecycle、governance、append、metric、persist 和 error response 不变。
5. activation scheduled path 的 activation state、status、updated timestamp、schedule event、lifecycle、metric、governance 和 event buffer 不变。
6. `next_cycle_start` immediate activation path 的 `activated_at_ms`、status、activation event、lifecycle、governance、metric 和 active parameter version 不变。
7. invalid resolved sequence path 的 activation failed semantics 不变。
8. `append_parameter_mutation_events_to_run` 仍先于 `persist_runtime_parameter_mutation_transition`。
9. `auto_snapshot_on_activation` 仍在 persist 后执行。
10. 不启动发布过渡，不引入 sibling horizontal link。

---

## 预期残余变化

BE-001EE-03 完成后预期:

```text
actual_parent_import_bridge_16_to_15
actual_mutation_import_bridge_14_to_13
actual_parameter_mutation_import_bridge_4_to_3
actual_transition_lifecycle_import_bridge_3_to_2
remaining_parent_import_bridge_15
remaining_mutation_import_bridge_13
remaining_parameter_mutation_import_bridge_3
remaining_transition_lifecycle_import_bridge_2
```

如果实际统计与预期不一致，BE-001EE-03 必须停在记录阶段说明差异，不得顺手扩大改写范围。

---

## 下一步边界

下一步只能进入:

```text
BE-001EE-03
runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass
实际抽离记录
```

BE-001EE-03 完成后必须回到单叶 closeout，判断本 import pocket 是否值得继续细拆。

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

AI 声称 BE-001EE-02 完成时，必须说明:

1. 本批是 `no code movement`。
2. 下一步只允许改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` 顶部 import。
3. 不得改函数体、可见性、parent facade、rollback flow、snapshot side effect 或 sibling。
4. 当前 residual 仍为 total 16 / mutation 14 / parameter_mutation 4 / transition_lifecycle 3。
5. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得宣称 activation_flow import 已改写、rollback_flow import 已改写、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `392-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001EE-03 的单文件 import rewrite 边界被固定。
3. 不恢复旧三叶暂停目标。
4. Rust / 治理 / 全量树门禁均通过。
