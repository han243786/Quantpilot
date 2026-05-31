# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EA-02
> 基线: `381-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EA-03 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EA-02 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 抽离方案 | 实施边界冻结 |
| 规范矩阵 | staged explicit import pass / single-file rewrite / parent-child communication | 改写约束冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` | transition persistence import rewrite 指令 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` | 固定下一步实际抽离范围 |

---

## 方案冻结

```text
runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass
transition_record_persistence_import_pass plan_frozen
single_file_transition_record_persistence_import_rewrite
be_001ea_03_only_rewrite_transition_record_persistence_imports
remaining_parent_import_bridge_18
remaining_mutation_import_bridge_16
remaining_parameter_mutation_import_bridge_6
remaining_transition_lifecycle_import_bridge_5
old_three_leaf_pause_target_cancelled
```

BE-001EA-03 只允许改写一个文件的顶部 import:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
```

当前 import:

```rust
use super::*;
```

预期 import:

```rust
use crate::{
    auth, io_error, mutation_event_contract, persist_runtime_parameter_mutation_record, AppState,
    FrontendRuntimeEvent, RuntimeParameterMutationLifecycleEntry, RuntimeParameterMutationRecord,
    RuntimeParameterMutationStatus,
};
use axum::http::StatusCode;
```

---

## 禁止改写范围

BE-001EA-03 不得触碰:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/proposal_creation.rs
src/runtime/mutation/parameter_mutation/record_query.rs
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
no_activation_flow_rewrite
no_rollback_flow_rewrite
no_transition_lifecycle_facade_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

---

## 等价清单

BE-001EA-03 必须保持:

1. `mutation_lifecycle_entry` 名称、签名、`pub(super)` 可见性不变。
2. `mutation_lifecycle_entry` 仍通过 `mutation_event_contract(status)` 取得 reason code。
3. lifecycle entry 的 `status`、`event_id`、`sequence_no`、`occurred_at_ms`、`reason_code`、`message` 映射不变。
4. `event_id` 仍来自 `event.event_id.clone()`。
5. `occurred_at_ms` 仍来自 `event.event_time_ms`。
6. `message` 仍通过 `message.into()` 写入。
7. `persist_runtime_parameter_mutation_transition` 名称、签名、`pub(super)` 可见性不变。
8. 持久化调用仍为 `persist_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), record).await.map_err(io_error)?`。
9. in-memory ledger 写入仍使用 `auth::scoped_key(user_id, &record.proposal_id)` 和 `record.clone()`。
10. 返回类型仍为 `Result<(), (StatusCode, String)>`。
11. `transition_lifecycle.rs` parent facade 与 activation / rollback sibling 调用面不变。
12. 不启动发布过渡，不引入 sibling horizontal link。

---

## 预期残余变化

BE-001EA-03 完成后预期:

```text
actual_parent_import_bridge_18_to_17
actual_mutation_import_bridge_16_to_15
actual_parameter_mutation_import_bridge_6_to_5
actual_transition_lifecycle_import_bridge_5_to_4
remaining_parent_import_bridge_17
remaining_mutation_import_bridge_15
remaining_parameter_mutation_import_bridge_5
remaining_transition_lifecycle_import_bridge_4
```

如果实际统计与预期不一致，BE-001EA-03 必须停在记录阶段说明差异，不得顺手扩大改写范围。

---

## 下一步边界

下一步只能进入:

```text
BE-001EA-03
runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass
实际抽离记录
```

BE-001EA-03 完成后必须回到单叶 closeout，判断本 import pocket 是否值得继续细拆。

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

AI 声称 BE-001EA-02 完成时，必须说明:

1. 本批是 `no code movement`。
2. 下一步只允许改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs` 顶部 import。
3. 不得改函数体、可见性、parent facade、activation flow、rollback flow 或 sibling。
4. 当前 residual 仍为 total 18 / mutation 16 / parameter_mutation 6 / transition_lifecycle 5。
5. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得宣称 transition_record_persistence import 已改写、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `382-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001EA-03 的单文件 import rewrite 边界被固定。
3. 不恢复旧三叶暂停目标。
4. Rust / 治理 / 全量树门禁均通过。
