# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EG-01
> 基线: `395-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第五轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EG-02 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EG-01 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | staged explicit import pass / parent white-box helper / rollback flow contract | 输入面冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` | rollback flow 白箱登记 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` | 建立单子叶基线 |

---

## 基线冻结

```text
runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass
rollback_flow_import_pass baseline_frozen
single_file_rollback_flow_import_pass
remaining_parent_import_bridge_15
remaining_mutation_import_bridge_13
remaining_parameter_mutation_import_bridge_3
remaining_transition_lifecycle_import_bridge_2
old_three_leaf_pause_target_cancelled
```

冻结文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
```

当前 residual:

```rust
use super::*;
```

本批不移动代码、不改函数体、不改可见性、不改 parent facade、不改 activation flow、不改 snapshot side-effect。

---

## 白箱输入输出

目标 handler:

| handler | 当前可见性 | 调用方 | 约束 |
| --- | --- | --- | --- |
| `rollback_runtime_parameter_mutation` | `pub(crate)` | `transition_lifecycle.rs` parent facade re-export | 不改 rollback 状态机、ledger lookup、safe-window、事件写入和持久化 |

函数签名必须保持:

```rust
pub(crate) async fn rollback_runtime_parameter_mutation(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<RollbackRuntimeParameterMutationRequest>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)>
```

显式输入面候选:

```text
super::evaluate_runtime_parameter_mutation_safe_window
super::mutation_lifecycle_entry
super::persist_runtime_parameter_mutation_transition
super::resolve_runtime_parameter_mutation_boundary
super::runtime_parameter_mutation_rollback_record_id
auth
append_parameter_mutation_events_to_run
build_runtime_parameter_mutation_event
current_time_ms
governance_with_parameter_version
io_error
json_bad_request
json_bad_request_with_details
list_runtime_parameter_mutation_records
load_run_record_from_state
load_runtime_parameter_mutation_record
normalize_actor_identity
runtime_parameter_mutation_governance
validate_runtime_capability_guard
AppState
RollbackRuntimeParameterMutationRequest
RuntimeParameterMutationActivationState
RuntimeParameterMutationBoundary
RuntimeParameterMutationRecord
RuntimeParameterMutationStatus
axum::extract::{Path, State}
axum::http::StatusCode
axum::Json
```

预期 BE-001EG-03 import 方向:

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

`super::*` 的替换必须以 `cargo check` 为准；若实际路径需要更小修正，BE-001EG-03 必须记录差异，不得扩大改写范围。

---

## 等价语义

必须保持不变:

1. capability guard 仍通过 `validate_runtime_capability_guard(request.capability_context.as_ref())` 执行。
2. guard 失败仍返回 `json_bad_request_with_details("parameter_mutation_boundary_violation", ...)`。
3. original mutation record 仍通过 `load_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), &proposal_id).await?` 读取。
4. 只允许 `Activated` 状态进入 rollback；其他状态仍返回 `json_bad_request("bad_request", "仅已激活的参数变更可以回滚")`。
5. rollback attempt metric 仍通过 `record_mutation_rollback_attempt` 记录。
6. source run 仍通过 `load_run_record_from_state(&state, &user_id, &original.source_id).await?` 读取。
7. target parameter version 仍优先使用 request override，否则使用 `original.old_parameter_version`。
8. ledger 仍通过 `list_runtime_parameter_mutation_records(&state.mutation_store_dir).await.map_err(io_error)?` 读取。
9. rollback value 仍按 source id、target、old parameter version、proposed parameter version 匹配。
10. 未知目标版本仍返回 `parameter_mutation_rollback_unknown_version`。
11. no-op 目标版本仍返回 `parameter_mutation_rollback_noop`。
12. requested boundary 仍优先使用 request activation boundary，否则使用 `RuntimeParameterMutationBoundary::default`。
13. resolved boundary 仍通过 `resolve_runtime_parameter_mutation_boundary(&requested_boundary, current_sequence_no)?` 取得。
14. `now_ms` 仍来自 `current_time_ms()`。
15. actor 仍通过 `normalize_actor_identity(Some(actor))` 或 original actor 回退。
16. rollback proposal id 仍通过 `runtime_parameter_mutation_rollback_record_id(...)` 生成。
17. governance 仍通过 `runtime_parameter_mutation_governance(...)` 生成。
18. 新 record 仍以 `RuntimeParameterMutationStatus::RollbackScheduled` 初始化，并保留 `rollback_of` 与 `rollback_target_parameter_version`。
19. safe-window denied path 仍设置 `SafeWindowDenied`、写 lifecycle entry、append event、记录 `record_mutation_safe_window_denied`、persist transition，并返回 `parameter_mutation_safe_window_denied`。
20. scheduled path 仍写 `RuntimeParameterMutationActivationState`，状态保持 `RollbackScheduled`，写 schedule lifecycle entry 和 `record_mutation_rollback_scheduled`。
21. `next_cycle_start` path 仍转为 `RolledBack`，设置 activated time、active parameter version、rollback lifecycle、governance event 和 `record_mutation_rollback_applied`。
22. resolved sequence 不在 schedule 后时仍转 `RollbackFailed`，写 failure reason、failure lifecycle 和 `record_mutation_rollback_failed`。
23. events 仍通过 `append_parameter_mutation_events_to_run(...)` 写回 source run。
24. record 仍通过 `persist_runtime_parameter_mutation_transition(&state, &user_id, &record).await?` 持久化。
25. 返回值仍为 `Ok(Json(record))`。
26. 不启动发布过渡，不引入 sibling horizontal link。

ASCII guard:

```text
no_code_movement
no_function_body_change
no_visibility_change
no_parent_facade_rewrite
no_activation_flow_rewrite
no_snapshot_side_effect_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

---

## 影响边界

BE-001EG-01 只冻结 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` 的 import 输入面。

不得触碰:

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
BE-001EG-02
runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass
抽离方案
```

BE-001EG-02 必须固定 BE-001EG-03 的单文件 import rewrite 边界，不得直接改 Rust。

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

AI 声称 BE-001EG-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 冻结文件是 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`。
3. 当前 residual 是 `use super::*`。
4. handler 是 `rollback_runtime_parameter_mutation`。
5. 当前 residual 仍为 total 15 / mutation 13 / parameter_mutation 3 / transition_lifecycle 2。
6. 下一步只能进入 BE-001EG-02 抽离方案。
7. 旧三叶暂停目标保持取消。

不得宣称 rollback_flow import 已改写、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `396-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 冻结 rollback_flow 当前输入面与等价语义。
3. 下一步固定为 BE-001EG-02 抽离方案。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
