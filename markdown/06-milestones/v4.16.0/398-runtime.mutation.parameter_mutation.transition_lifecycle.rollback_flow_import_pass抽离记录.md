# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EG-03
> 基线: `397-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass`
> 代码动作: actual import rewrite
> 下一步: BE-001EG-04 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EG-03 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 实际抽离记录 | 实施记录 |
| 规范矩阵 | staged explicit import pass / single-file rewrite / parent-child communication | `use super::*` 移除 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` | rollback flow import rewrite 已落地 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` | 等价执行记录 |

---

## 实际变更

```text
runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass
rollback_flow_import_pass extraction_complete
single_file_rollback_flow_import_rewrite
removed_parent_wildcard_import
actual_parent_import_bridge_15_to_14
actual_mutation_import_bridge_13_to_12
actual_parameter_mutation_import_bridge_3_to_2
actual_transition_lifecycle_import_bridge_2_to_1
remaining_parent_import_bridge_14
remaining_mutation_import_bridge_12
remaining_parameter_mutation_import_bridge_2
remaining_transition_lifecycle_import_bridge_1
old_three_leaf_pause_target_cancelled
```

实际改写文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
```

改写前:

```rust
use super::*;
```

改写后:

```rust
use super::super::super::{
    append_parameter_mutation_events_to_run, build_runtime_parameter_mutation_event,
    governance_with_parameter_version, runtime_parameter_mutation_governance,
};
use super::{
    evaluate_runtime_parameter_mutation_safe_window, mutation_lifecycle_entry,
    persist_runtime_parameter_mutation_transition, resolve_runtime_parameter_mutation_boundary,
    runtime_parameter_mutation_rollback_record_id,
};
use crate::{
    auth, current_time_ms, io_error, json_bad_request, json_bad_request_with_details,
    list_runtime_parameter_mutation_records, load_run_record_from_state,
    load_runtime_parameter_mutation_record, normalize_actor_identity,
    validate_runtime_capability_guard, AppState, RollbackRuntimeParameterMutationRequest,
    RuntimeParameterMutationActivationState, RuntimeParameterMutationBoundary,
    RuntimeParameterMutationRecord, RuntimeParameterMutationStatus,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
```

与 BE-001EG-02 方案的差异:

```text
compiler_confirmed_expected_path
no_additional_path_adjustment
append_parameter_mutation_events_to_run
build_runtime_parameter_mutation_event
governance_with_parameter_version
runtime_parameter_mutation_governance
```

本批按 BE-001EG-02 方案落地；共享治理 helper 继续从 runtime 祖先白箱输入面导入。未改 parent facade，未新增 sibling horizontal link，未启动 release transition。

---

## 等价保持

本批只替换 import 输入面，以下内容未改动:

1. `rollback_runtime_parameter_mutation` 名称、签名和 `pub(crate)` 可见性。
2. capability guard 与 `json_bad_request_with_details` 错误语义。
3. original mutation record load、activated-only gate、rollback attempt metric、source run load、ledger lookup、boundary resolution、actor normalization、rollback id、governance projection、safe-window evaluation、lifecycle entry、event append 和 persistence 的调用顺序。
4. 只允许 `Activated` 状态进入 rollback。
5. unknown target version path 仍返回 `parameter_mutation_rollback_unknown_version`。
6. no-op target version path 仍返回 `parameter_mutation_rollback_noop`。
7. safe-window denied path 仍写入 `SafeWindowDenied`、denied event、denied lifecycle、governance、append、metric、persist 和 `parameter_mutation_safe_window_denied` error response。
8. scheduled path 仍写入 `RuntimeParameterMutationActivationState`，状态保持 `RollbackScheduled`。
9. `next_cycle_start` immediate rollback path 仍转为 `RolledBack` 并设置 active parameter version。
10. invalid resolved sequence path 仍转 `RollbackFailed` 并写 failure lifecycle entry。
11. `append_parameter_mutation_events_to_run` 仍先于 `persist_runtime_parameter_mutation_transition`。

精确状态标记:

```text
Activated
SafeWindowDenied
RollbackScheduled
RolledBack
RollbackFailed
parameter_mutation_rollback_unknown_version
parameter_mutation_rollback_noop
parameter_mutation_safe_window_denied
next_cycle_start
```

ASCII guard:

```text
no_function_body_change
no_visibility_change
no_parent_facade_rewrite
no_activation_flow_rewrite
no_snapshot_side_effect_rewrite
no_transition_lifecycle_facade_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

---

## 未触碰范围

本批未触碰:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
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

---

## 下一步边界

下一步只能进入:

```text
BE-001EG-04
runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass
单叶 closeout
```

BE-001EG-04 必须判断本 import pocket 是否值得继续细拆；不得跳过 closeout 直接宣称 transition_lifecycle_import_pass、parameter_mutation_import_pass、mutation_import_pass、backend.runtime 或 Rust 重构完成。

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

AI 声称 BE-001EG-03 完成时，必须说明:

1. 本批实际改写仅限 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` 顶部 import。
2. `use super::*` 已移除并改为显式输入面。
3. 函数体、可见性、parent facade、activation flow、snapshot side effect 和 sibling 均未改。
4. residual 降为 total 14 / mutation 12 / parameter_mutation 2 / transition_lifecycle 1。
5. 共享治理 helper 从 runtime 祖先白箱输入面导入，且编译器确认预期路径成立。
6. 下一步只能进入 BE-001EG-04 单叶 closeout。
7. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得宣称 rollback_flow closeout 已完成、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `398-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` 的 parent wildcard import 被清除。
3. 等价语义与父子通信边界保持不变。
4. 下一步固定为 BE-001EG-04 单叶 closeout。
5. Rust / 治理 / 全量树门禁均通过。
