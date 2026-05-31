# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EE-03
> 基线: `392-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass`
> 代码动作: actual import rewrite
> 下一步: BE-001EE-04 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EE-03 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 实际抽离记录 | 实施记录 |
| 规范矩阵 | staged explicit import pass / single-file rewrite / parent-child communication | `use super::*` 移除 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` | activation flow import rewrite 已落地 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` | 等价执行记录 |

---

## 实际变更

```text
runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass
activation_flow_import_pass extraction_complete
single_file_activation_flow_import_rewrite
removed_parent_wildcard_import
actual_parent_import_bridge_16_to_15
actual_mutation_import_bridge_14_to_13
actual_parameter_mutation_import_bridge_4_to_3
actual_transition_lifecycle_import_bridge_3_to_2
remaining_parent_import_bridge_15
remaining_mutation_import_bridge_13
remaining_parameter_mutation_import_bridge_3
remaining_transition_lifecycle_import_bridge_2
old_three_leaf_pause_target_cancelled
```

实际改写文件:

```text
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
```

改写前:

```rust
use super::*;
```

改写后:

```rust
use super::super::super::{
    append_parameter_mutation_events_to_run, build_runtime_parameter_mutation_event,
    governance_with_parameter_version,
};
use super::{
    auto_snapshot_on_activation, evaluate_runtime_parameter_mutation_safe_window,
    mutation_lifecycle_entry, persist_runtime_parameter_mutation_transition,
    resolve_runtime_parameter_mutation_boundary,
};
use crate::{
    auth, current_time_ms, json_bad_request, json_bad_request_with_details,
    load_run_record_from_state, load_runtime_parameter_mutation_record, normalize_actor_identity,
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

与 BE-001EE-02 方案的差异:

```text
compiler_driven_minimal_path_adjustment
shared_governance_helpers_from_runtime_ancestor_white_box
append_parameter_mutation_events_to_run
build_runtime_parameter_mutation_event
governance_with_parameter_version
```

`append_parameter_mutation_events_to_run`、`build_runtime_parameter_mutation_event` 与 `governance_with_parameter_version` 未在 crate root 暴露；本批按 `cargo check` 反馈改为从 runtime 祖先白箱输入面导入。未改 parent facade，未新增 sibling horizontal link，未启动 release transition。

---

## 等价保持

本批只替换 import 输入面，以下内容未改动:

1. `activate_runtime_parameter_mutation` 名称、签名和 `pub(crate)` 可见性。
2. capability guard 与 `json_bad_request_with_details` 错误语义。
3. `load_runtime_parameter_mutation_record`、`load_run_record_from_state`、boundary resolution、actor normalization 与 safe-window evaluation 的调用顺序。
4. 只允许 `Proposed` 或 `SafeWindowDenied` 状态进入 activation。
5. safe-window denied path 仍写入 `SafeWindowDenied`、denied event、denied lifecycle、governance、append、metric、persist 和 error response。
6. scheduled path 仍写入 `RuntimeParameterMutationActivationState`，状态转为 `ActivationScheduled`。
7. `next_cycle_start` immediate activation path 仍写入 `activated_at_ms`，状态转为 `Activated`。
8. invalid resolved sequence path 仍转 `ActivationFailed` 并写 failure lifecycle entry。
9. `append_parameter_mutation_events_to_run` 仍先于 `persist_runtime_parameter_mutation_transition`。
10. `auto_snapshot_on_activation(&state, &user_id, &record).await` 仍在 persist 后执行。

精确状态标记:

```text
ActivationScheduled
Activated
ActivationFailed
SafeWindowDenied
parameter_mutation_safe_window_denied
next_cycle_start
```

ASCII guard:

```text
no_function_body_change
no_visibility_change
no_parent_facade_rewrite
no_rollback_flow_rewrite
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

---

## 下一步边界

下一步只能进入:

```text
BE-001EE-04
runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass
单叶 closeout
```

BE-001EE-04 必须判断本 import pocket 是否值得继续细拆；不得跳过 closeout 直接宣称 transition_lifecycle_import_pass、parameter_mutation_import_pass、mutation_import_pass、backend.runtime 或 Rust 重构完成。

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

AI 声称 BE-001EE-03 完成时，必须说明:

1. 本批实际改写仅限 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` 顶部 import。
2. `use super::*` 已移除并改为显式输入面。
3. 函数体、可见性、parent facade、rollback flow、snapshot side effect 和 sibling 均未改。
4. residual 降为 total 15 / mutation 13 / parameter_mutation 3 / transition_lifecycle 2。
5. 三个共享治理 helper 采用 compiler-driven minimal path adjustment，从 runtime 祖先白箱输入面导入。
6. 下一步只能进入 BE-001EE-04 单叶 closeout。
7. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得宣称 activation_flow closeout 已完成、rollback_flow import 已改写、transition_lifecycle_import_pass 已完成、parameter_mutation_import_pass 已完成、mutation_import_pass 已完成、parent_import_bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `393-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` 的 parent wildcard import 被清除。
3. 等价语义与父子通信边界保持不变。
4. 下一步固定为 BE-001EE-04 单叶 closeout。
5. Rust / 治理 / 全量树门禁均通过。
