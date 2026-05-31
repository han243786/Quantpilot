# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DW-03
> 方案: `372-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass`
> 代码动作: single file import rewrite
> 下一步: BE-001DW-04 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DW-03 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 实际抽离 | import residual 收敛 |
| 规范矩阵 | staged explicit import pass / 父子通信硬规则 | parent wildcard 移除 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` | 白箱输入显式化 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` | 单子叶实际抽离完成 |

---

## 实际改动

```text
boundary_safety_import_pass extraction_complete
single_file_boundary_safety_import_rewrite_complete
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
use super::* removed
explicit imports installed
json_bad_request
RuntimeParameterMutationBoundary
RuntimeParameterMutationSafeWindowSnapshot
RuntimeParameterMutationSafeWindowState
StatusCode
validate_runtime_parameter_mutation_boundary
resolve_runtime_parameter_mutation_boundary
evaluate_runtime_parameter_mutation_safe_window
no_function_body_change
no_visibility_change
no_transition_lifecycle_facade_rewrite
no_activation_or_rollback_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

原 import:

```rust
use super::*;
```

现 import:

```rust
use crate::{
    json_bad_request, RuntimeParameterMutationBoundary, RuntimeParameterMutationSafeWindowSnapshot,
    RuntimeParameterMutationSafeWindowState,
};
use axum::http::StatusCode;
```

函数体、helper 可见性、facade、activation / rollback sibling 均无改动。

---

## residual 下降

本批实际下降:

```text
actual_parent_import_bridge_20_to_19
actual_mutation_import_bridge_18_to_17
actual_parameter_mutation_import_bridge_8_to_7
actual_transition_lifecycle_import_bridge_7_to_6
```

当前 residual:

```text
remaining_parent_import_bridge_19
remaining_mutation_import_bridge_17
remaining_parameter_mutation_import_bridge_7
remaining_transition_lifecycle_import_bridge_6
```

当前 remaining files:

```text
src/runtime/mod.rs
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
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
src/runtime/run_guard.rs
```

---

## 等价确认

必须保持不变的 boundary / safe window 语义:

1. 空 `requested` 仍返回 `bad_request` 与 `activation_boundary.requested 是必填字段`。
2. `immediate` 仍返回 `parameter_mutation_boundary_violation` 与“不支持立即激活的参数变更”。
3. `next_cycle_start` 与 `manual_pause` 仍直接通过 validation。
4. `sequence_cursor` 仍需要 `resolved_sequence_no`，并从 `sequence_cursor:<u64>` 解析。
5. `next_cycle_start` resolution 仍为 `current_sequence_no + 2`。
6. `manual_pause` resolution 仍为 `resolved_sequence_no: None`。
7. 缺少 sequence cursor 时仍返回 `parameter_mutation_boundary_violation` 与“序列游标激活边界需要 resolved_sequence_no”。
8. safe window reason code 优先级仍包含 `SAFE_WINDOW_RUNTIME_ACTIVE`、`SAFE_WINDOW_OPEN_ORDERS`、`SAFE_WINDOW_RISK_VIOLATION`、`SAFE_WINDOW_STALE_DATA`、`SAFE_WINDOW_EXPOSURE_LIMIT`、`SAFE_WINDOW_COOLDOWN`。
9. safe window `allowed` 仍只在 `SAFE_WINDOW_OPEN` 时为 true，snapshot 原样回填。
10. 未启动 release transition，未新增 sibling horizontal link。

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

AI 声称 BE-001DW-03 完成时，必须说明:

1. 只改了 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` 顶部 import。
2. `use super::*` 已移除，改为 `crate` 显式输入与 `axum::http::StatusCode`。
3. 函数体、helper visibility、facade、activation / rollback sibling、AI proposal、root bridge 与 release transition 均未改。
4. residual 降为 total 19 / mutation 17 / parameter_mutation 7 / transition_lifecycle 6。
5. 下一步只能进入 BE-001DW-04 单叶 closeout。
6. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。

不得宣称 transition lifecycle 已完成、parameter mutation import 已完成、mutation import 已完成、parent import bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `373-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `boundary_safety.rs` parent wildcard import 清零。
3. `api_mutation` 回归通过。
4. residual 数量与文件列表匹配。
5. 下一步固定为 BE-001DW-04 单叶 closeout。
